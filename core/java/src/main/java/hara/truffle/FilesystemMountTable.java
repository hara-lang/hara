package hara.truffle;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicLong;

/**
 * Kernel-ready ownership table for opened provider-neutral filesystem capabilities.
 *
 * <p>The table keeps provider construction, redacted descriptors, attachment accounting, and
 * provider close in one place. A synchronous Graal filesystem is an optional local-only adapter;
 * remote providers never need to implement {@code java.nio.file.Path} or Graal's FileSystem API.
 */
final class FilesystemMountTable implements AutoCloseable {
  @FunctionalInterface
  interface GraalAdapterFactory {
    HaraMountedFileSystem create(Map<String, ?> configuration);
  }

  record Info(
      SessionModel.SessionMountId id,
      IFilesystem.Descriptor descriptor,
      int attachments,
      boolean sourceLoadable) {
    Info {
      Objects.requireNonNull(id, "filesystem mount id");
      Objects.requireNonNull(descriptor, "filesystem descriptor");
      if (attachments < 0) throw new IllegalArgumentException("negative filesystem attachments");
    }
  }

  private static final class Mount {
    final IFilesystem filesystem;
    final IFilesystem.Descriptor descriptor;
    final HaraMountedFileSystem graalFilesystem;
    int attachments;

    Mount(
        IFilesystem filesystem,
        IFilesystem.Descriptor descriptor,
        HaraMountedFileSystem graalFilesystem) {
      this.filesystem = filesystem;
      this.descriptor = descriptor;
      this.graalFilesystem = graalFilesystem;
    }
  }

  private final FilesystemProviderRegistry providers;
  private final IFilesystemFactory.OpenContext openContext;
  private final ConcurrentHashMap<Long, Mount> mounts = new ConcurrentHashMap<>();
  private final AtomicLong nextId = new AtomicLong(1);
  private final AtomicBoolean closed = new AtomicBoolean();

  FilesystemMountTable(IFilesystemFactory.OpenContext openContext) {
    this(
        new FilesystemProviderRegistry().register(new NativeFilesystem.Factory()),
        openContext);
  }

  FilesystemMountTable(
      FilesystemProviderRegistry providers,
      IFilesystemFactory.OpenContext openContext) {
    this.providers = Objects.requireNonNull(providers, "filesystem provider registry");
    this.openContext = Objects.requireNonNull(openContext, "filesystem open context");
  }

  FilesystemMountTable register(IFilesystemFactory factory) {
    requireOpen();
    providers.register(factory);
    return this;
  }

  boolean supports(String kind) {
    return providers.contains(kind);
  }

  CompletionStage<SessionModel.SessionMountId> open(
      String kind, Map<String, ?> configuration) {
    return open(kind, configuration, null);
  }

  CompletionStage<SessionModel.SessionMountId> openNative(Path root) {
    Objects.requireNonNull(root, "native filesystem root");
    Path normalized = root.toAbsolutePath().normalize();
    if (!Files.isDirectory(normalized)) {
      return CompletableFuture.failedFuture(
          new IllegalArgumentException("FILESYSTEM_NOT_FOUND " + normalized));
    }
    Map<String, ?> configuration = Map.of("root", normalized.toString());
    return open(
        "native",
        configuration,
        ignored -> new HaraMountedFileSystem(normalized));
  }

  CompletionStage<SessionModel.SessionMountId> openSourceLoadable(
      String kind,
      Map<String, ?> configuration,
      GraalAdapterFactory graalAdapterFactory) {
    Objects.requireNonNull(graalAdapterFactory, "Graal filesystem adapter factory");
    return open(kind, configuration, graalAdapterFactory);
  }

  private CompletionStage<SessionModel.SessionMountId> open(
      String kind,
      Map<String, ?> configuration,
      GraalAdapterFactory graalAdapterFactory) {
    try {
      requireOpen();
    } catch (Throwable error) {
      return CompletableFuture.failedFuture(error);
    }
    Map<String, ?> frozen = Map.copyOf(Objects.requireNonNull(configuration, "configuration"));
    return providers
        .open(kind, openContext, frozen)
        .thenCompose(
            filesystem ->
                publish(
                    Objects.requireNonNull(filesystem, "opened filesystem"),
                    kind,
                    frozen,
                    graalAdapterFactory));
  }

  private CompletionStage<SessionModel.SessionMountId> publish(
      IFilesystem filesystem,
      String requestedKind,
      Map<String, ?> configuration,
      GraalAdapterFactory graalAdapterFactory) {
    try {
      IFilesystem.Descriptor descriptor =
          Objects.requireNonNull(filesystem.descriptor(), "filesystem descriptor");
      if (!requestedKind.equals(descriptor.kind())) {
        throw new IllegalStateException(
            "FILESYSTEM_PROVIDER_KIND_MISMATCH "
                + requestedKind
                + " "
                + descriptor.kind());
      }
      HaraMountedFileSystem graalFilesystem =
          graalAdapterFactory == null ? null : graalAdapterFactory.create(configuration);
      synchronized (this) {
        requireOpen();
        long value = nextId.getAndIncrement();
        if (value <= 0) throw new IllegalStateException("FILESYSTEM_IDS_EXHAUSTED");
        SessionModel.SessionMountId id = SessionModel.SessionMountId.of(value);
        mounts.put(value, new Mount(filesystem, descriptor, graalFilesystem));
        return CompletableFuture.completedFuture(id);
      }
    } catch (Throwable error) {
      return rejectAfterClose(filesystem, error);
    }
  }

  IFilesystem filesystem(SessionModel.SessionMountId id) {
    return requireMount(id).filesystem;
  }

  HaraMountedFileSystem graalFilesystem(SessionModel.SessionMountId id) {
    return requireMount(id).graalFilesystem;
  }

  synchronized Info info(SessionModel.SessionMountId id) {
    Mount mount = requireMount(id);
    return new Info(id, mount.descriptor, mount.attachments, mount.graalFilesystem != null);
  }

  synchronized void retain(SessionModel.SessionMountId id) {
    Mount mount = requireMount(id);
    if (mount.attachments == Integer.MAX_VALUE) {
      throw new IllegalStateException("FILESYSTEM_ATTACHMENTS_EXHAUSTED " + id);
    }
    mount.attachments++;
  }

  synchronized void release(SessionModel.SessionMountId id) {
    Mount mount = requireMount(id);
    if (mount.attachments == 0) {
      throw new IllegalStateException("FILESYSTEM_ATTACHMENT_UNDERFLOW " + id);
    }
    mount.attachments--;
  }

  CompletionStage<Void> close(SessionModel.SessionMountId id) {
    final Mount mount;
    synchronized (this) {
      mount = requireMount(id);
      if (mount.attachments != 0) {
        return CompletableFuture.failedFuture(
            new IllegalArgumentException("FILESYSTEM_ATTACHED " + id));
      }
      mounts.remove(id.value());
    }
    return closeProvider(mount.filesystem);
  }

  synchronized int size() {
    return mounts.size();
  }

  CompletionStage<Void> closeAll() {
    List<Mount> owned;
    synchronized (this) {
      if (!closed.compareAndSet(false, true)) {
        return CompletableFuture.completedFuture(null);
      }
      owned = new ArrayList<>(mounts.values());
      mounts.clear();
    }
    CompletableFuture<?>[] closing =
        owned.stream()
            .map(mount -> closeProvider(mount.filesystem).toCompletableFuture())
            .toArray(CompletableFuture[]::new);
    return CompletableFuture.allOf(closing);
  }

  @Override
  public void close() {
    try {
      closeAll().toCompletableFuture().join();
    } catch (CompletionException error) {
      Throwable cause = unwrap(error);
      if (cause instanceof RuntimeException runtime) throw runtime;
      throw error;
    }
  }

  private synchronized Mount requireMount(SessionModel.SessionMountId id) {
    Objects.requireNonNull(id, "filesystem mount id");
    Mount mount = mounts.get(id.value());
    if (mount == null) throw new IllegalArgumentException("NO_FILESYSTEM " + id);
    return mount;
  }

  private void requireOpen() {
    if (closed.get()) throw new IllegalStateException("FILESYSTEM_TABLE_CLOSED");
  }

  private static CompletionStage<Void> closeProvider(IFilesystem filesystem) {
    try {
      CompletionStage<Void> closing =
          filesystem.close(IFilesystem.CallContext.create());
      return Objects.requireNonNull(closing, "filesystem close stage");
    } catch (Throwable error) {
      return CompletableFuture.failedFuture(error);
    }
  }

  private static CompletionStage<SessionModel.SessionMountId> rejectAfterClose(
      IFilesystem filesystem, Throwable original) {
    CompletableFuture<SessionModel.SessionMountId> rejected = new CompletableFuture<>();
    closeProvider(filesystem)
        .whenComplete(
            (ignored, closeError) -> {
              if (closeError != null) original.addSuppressed(unwrap(closeError));
              rejected.completeExceptionally(original);
            });
    return rejected;
  }

  private static Throwable unwrap(Throwable error) {
    Throwable current = error;
    while ((current instanceof CompletionException
            || current instanceof java.util.concurrent.ExecutionException)
        && current.getCause() != null) {
      current = current.getCause();
    }
    return current;
  }
}
