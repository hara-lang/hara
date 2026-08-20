package hara.truffle;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.WeakHashMap;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.atomic.AtomicLong;

/** Kernel-scoped ownership for provider-neutral filesystem factories and mounts. */
final class KernelFilesystemRuntime {
  record MountId(long value) {
    MountId {
      if (value <= 0) throw new IllegalArgumentException("filesystem mount id must be positive");
    }
  }

  record FilesystemInfo(IFilesystem.Descriptor descriptor, int attachments) {
    FilesystemInfo {
      descriptor = Objects.requireNonNull(descriptor, "filesystem descriptor");
      if (attachments < 0) throw new IllegalArgumentException("negative attachment count");
    }
  }

  private static final Map<SessionKernel, KernelFilesystemRuntime> KERNELS =
      new WeakHashMap<>();

  private static final class Mount {
    final IFilesystem filesystem;
    final IFilesystem.Descriptor descriptor;
    int attachments;
    boolean closing;

    Mount(IFilesystem filesystem) {
      this.filesystem = Objects.requireNonNull(filesystem, "filesystem");
      this.descriptor = Objects.requireNonNull(filesystem.descriptor(), "filesystem descriptor");
    }
  }

  static synchronized KernelFilesystemRuntime install(
      SessionKernel kernel, IFilesystemFactory.OpenContext context) {
    Objects.requireNonNull(kernel, "session kernel");
    Objects.requireNonNull(context, "filesystem open context");
    return KERNELS.computeIfAbsent(
        kernel, ignored -> new KernelFilesystemRuntime(kernel, context));
  }

  static synchronized KernelFilesystemRuntime require(SessionKernel kernel) {
    KernelFilesystemRuntime runtime =
        KERNELS.get(Objects.requireNonNull(kernel, "session kernel"));
    if (runtime == null) throw new IllegalStateException("FILESYSTEM_RUNTIME_NOT_INSTALLED");
    return runtime;
  }

  static CompletionStage<Void> release(SessionKernel kernel) {
    KernelFilesystemRuntime runtime;
    synchronized (KernelFilesystemRuntime.class) {
      runtime = KERNELS.remove(Objects.requireNonNull(kernel, "session kernel"));
    }
    return runtime == null ? CompletableFuture.completedFuture(null) : runtime.closeAll();
  }

  private final SessionKernel kernel;
  private final IFilesystemFactory.OpenContext context;
  private final FilesystemProviderRegistry providers = new FilesystemProviderRegistry();
  private final AtomicLong nextMountId = new AtomicLong(1);
  private final LinkedHashMap<Long, Mount> mounts = new LinkedHashMap<>();
  private final LinkedHashMap<String, Long> sessionAttachments = new LinkedHashMap<>();
  private boolean closed;

  private KernelFilesystemRuntime(
      SessionKernel kernel, IFilesystemFactory.OpenContext context) {
    this.kernel = kernel;
    this.context = context;
    providers.register(new NativeFilesystem.Factory());
  }

  synchronized KernelFilesystemRuntime register(IFilesystemFactory factory) {
    requireOpen();
    providers.register(factory);
    return this;
  }

  synchronized boolean contains(String kind) {
    return providers.contains(kind);
  }

  CompletionStage<MountId> open(Map<String, ?> specification) {
    Objects.requireNonNull(specification, "filesystem specification");
    String kind = requireKind(specification.get("kind"));
    LinkedHashMap<String, Object> configuration = new LinkedHashMap<>();
    for (Map.Entry<String, ?> entry : specification.entrySet()) {
      if (!"kind".equals(entry.getKey())) configuration.put(entry.getKey(), entry.getValue());
    }
    synchronized (this) {
      requireOpen();
    }
    return providers
        .open(kind, context, Map.copyOf(configuration))
        .thenCompose(this::publishMount);
  }

  synchronized void attach(SessionModel.SessionId sessionId, MountId mountId) {
    requireOpen();
    SessionKernel.Session session = kernel.require(sessionId);
    Mount mount = requireMount(mountId);
    Long current = sessionAttachments.get(session.id().value());
    if (current != null && current == mountId.value()) return;
    if (current != null) decrement(current);
    mount.attachments++;
    sessionAttachments.put(session.id().value(), mountId.value());
  }

  synchronized void detach(SessionModel.SessionId sessionId) {
    SessionKernel.Session session = kernel.require(sessionId);
    Long current = sessionAttachments.remove(session.id().value());
    if (current != null) decrement(current);
  }

  synchronized MountId filesystem(SessionModel.SessionId sessionId) {
    SessionKernel.Session session = kernel.require(sessionId);
    Long mountId = sessionAttachments.get(session.id().value());
    return mountId == null ? null : new MountId(mountId);
  }

  synchronized IFilesystem requireFilesystem(SessionModel.SessionId sessionId) {
    MountId mountId = filesystem(sessionId);
    if (mountId == null) throw new IllegalStateException("FILESYSTEM_UNATTACHED " + sessionId);
    return requireMount(mountId).filesystem;
  }

  synchronized FilesystemInfo info(MountId mountId) {
    Mount mount = requireMount(mountId);
    return new FilesystemInfo(mount.descriptor, mount.attachments);
  }

  CompletionStage<Void> closeMount(MountId mountId) {
    Mount mount;
    synchronized (this) {
      mount = requireMount(mountId);
      if (mount.attachments != 0) {
        return CompletableFuture.failedFuture(
            new IllegalArgumentException("FILESYSTEM_ATTACHED " + mountId.value()));
      }
      if (mount.closing) {
        return CompletableFuture.failedFuture(
            new IllegalStateException("FILESYSTEM_CLOSING " + mountId.value()));
      }
      mount.closing = true;
    }
    CompletableFuture<Void> result = new CompletableFuture<>();
    mount
        .filesystem
        .close(IFilesystem.CallContext.create())
        .whenComplete(
            (ignored, error) -> {
              synchronized (KernelFilesystemRuntime.this) {
                if (error == null) mounts.remove(mountId.value(), mount);
                else mount.closing = false;
              }
              if (error == null) result.complete(null);
              else result.completeExceptionally(unwrap(error));
            });
    return result;
  }

  CompletionStage<Void> closeAll() {
    List<Mount> closing;
    synchronized (this) {
      if (closed) return CompletableFuture.completedFuture(null);
      closed = true;
      sessionAttachments.clear();
      closing = new ArrayList<>(mounts.values());
      mounts.clear();
    }
    CompletableFuture<?>[] futures =
        closing.stream()
            .map(mount -> mount.filesystem.close(IFilesystem.CallContext.create()))
            .map(CompletionStage::toCompletableFuture)
            .toArray(CompletableFuture[]::new);
    return CompletableFuture.allOf(futures);
  }

  private CompletionStage<MountId> publishMount(IFilesystem filesystem) {
    synchronized (this) {
      if (!closed) {
        long value = nextMountId.getAndIncrement();
        if (value <= 0) {
          return closeAfterFailure(
              filesystem, new IllegalStateException("FILESYSTEM_IDS_EXHAUSTED"));
        }
        mounts.put(value, new Mount(filesystem));
        return CompletableFuture.completedFuture(new MountId(value));
      }
    }
    return closeAfterFailure(filesystem, new IllegalStateException("FILESYSTEM_RUNTIME_CLOSED"));
  }

  private CompletionStage<MountId> closeAfterFailure(IFilesystem filesystem, Throwable failure) {
    CompletableFuture<MountId> result = new CompletableFuture<>();
    filesystem
        .close(IFilesystem.CallContext.create())
        .whenComplete((ignored, closeError) -> result.completeExceptionally(failure));
    return result;
  }

  private Mount requireMount(MountId mountId) {
    Objects.requireNonNull(mountId, "filesystem mount id");
    Mount mount = mounts.get(mountId.value());
    if (mount == null) throw new IllegalArgumentException("NO_FILESYSTEM " + mountId.value());
    return mount;
  }

  private void decrement(long mountId) {
    Mount mount = mounts.get(mountId);
    if (mount != null && mount.attachments > 0) mount.attachments--;
  }

  private void requireOpen() {
    if (closed) throw new IllegalStateException("FILESYSTEM_RUNTIME_CLOSED");
  }

  private static String requireKind(Object value) {
    if (!(value instanceof String kind) || kind.isBlank()) {
      throw new IllegalArgumentException("filesystem specification kind is required");
    }
    return kind;
  }

  private static Throwable unwrap(Throwable error) {
    Throwable current = error;
    while ((current instanceof java.util.concurrent.CompletionException
            || current instanceof java.util.concurrent.ExecutionException)
        && current.getCause() != null) {
      current = current.getCause();
    }
    return current;
  }
}
