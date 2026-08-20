package hara.truffle;

import java.util.List;
import java.util.Objects;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicLong;

/**
 * Per-owner runtime binding for one opened filesystem capability.
 *
 * <p>The mount table owns provider lifetime. This binding owns only calls issued by one attached
 * Session or Sandbox: detach closes the binding, cancels its pending contexts, rejects every
 * unsettled result exactly once, and prevents later use without closing a provider shared by other
 * owners.
 */
final class FilesystemRuntimeBinding implements AutoCloseable {
  record Pending<T>(CompletableFuture<T> future, java.util.function.BooleanSupplier cancellation) {
    Pending {
      Objects.requireNonNull(future, "filesystem pending future");
      Objects.requireNonNull(cancellation, "filesystem pending cancellation");
    }

    boolean cancel() {
      return cancellation.getAsBoolean();
    }
  }

  private static final class ActiveCall {
    final IFilesystem.CallContext context;
    final CompletableFuture<?> result;
    final String operation;
    final String path;
    final String target;

    ActiveCall(
        IFilesystem.CallContext context,
        CompletableFuture<?> result,
        String operation,
        String path,
        String target) {
      this.context = context;
      this.result = result;
      this.operation = operation;
      this.path = path;
      this.target = target;
    }
  }

  @FunctionalInterface
  private interface Invocation<T> {
    CompletionStage<T> invoke(IFilesystem.CallContext context);
  }

  private final IFilesystem filesystem;
  private final IFilesystem.Descriptor descriptor;
  private final Set<ActiveCall> active = ConcurrentHashMap.newKeySet();
  private final AtomicBoolean closed = new AtomicBoolean();
  private final AtomicLong sequence = new AtomicLong(1);
  private final Object lifecycle = new Object();

  FilesystemRuntimeBinding(IFilesystem filesystem) {
    this.filesystem = Objects.requireNonNull(filesystem, "filesystem");
    this.descriptor =
        Objects.requireNonNull(filesystem.descriptor(), "filesystem descriptor");
  }

  IFilesystem.Descriptor descriptor() {
    return descriptor;
  }

  IFilesystem filesystem() {
    return filesystem;
  }

  boolean closed() {
    return closed.get();
  }

  int pendingCount() {
    return active.size();
  }

  Pending<IFilesystem.Entry> stat(String path) {
    String logical = HaraLogicalPath.normalise(path);
    return submit(
        "stat",
        logical,
        null,
        IFilesystem.Capability.READ,
        context -> filesystem.stat(context, logical));
  }

  Pending<byte[]> read(String path) {
    String logical = HaraLogicalPath.normalise(path);
    return submit(
        "read",
        logical,
        null,
        IFilesystem.Capability.READ,
        context -> filesystem.read(context, logical));
  }

  Pending<IFilesystem.Mutation> write(
      String path,
      byte[] bytes,
      IFilesystem.WriteOptions options,
      IFilesystem.MutationContext mutation) {
    String logical = HaraLogicalPath.normalise(path);
    byte[] frozen = Objects.requireNonNull(bytes, "filesystem bytes").clone();
    IFilesystem.WriteOptions validated = Objects.requireNonNull(options, "write options");
    IFilesystem.MutationContext expected =
        mutation == null ? IFilesystem.MutationContext.none() : mutation;
    IFilesystem.Capability capability =
        validated.mode() == IFilesystem.WriteMode.APPEND
            ? IFilesystem.Capability.APPEND
            : IFilesystem.Capability.WRITE;
    return submit(
        "write",
        logical,
        null,
        capability,
        context -> filesystem.write(context, logical, frozen, validated, expected));
  }

  Pending<Boolean> exists(String path) {
    String logical = HaraLogicalPath.normalise(path);
    return submit(
        "stat",
        logical,
        null,
        IFilesystem.Capability.READ,
        context -> FilesystemEffects.exists(filesystem, context, logical));
  }

  Pending<List<IFilesystem.Entry>> entries(String path) {
    String logical = HaraLogicalPath.normalise(path);
    return submit(
        "entries",
        logical,
        null,
        IFilesystem.Capability.ENTRIES,
        context -> FilesystemEffects.entries(filesystem, context, logical));
  }

  Pending<List<String>> list(String path) {
    String logical = HaraLogicalPath.normalise(path);
    return submit(
        "entries",
        logical,
        null,
        IFilesystem.Capability.ENTRIES,
        context -> FilesystemEffects.list(filesystem, context, logical));
  }

  Pending<List<String>> walk(String path) {
    String logical = HaraLogicalPath.normalise(path);
    return submit(
        "walk",
        logical,
        null,
        IFilesystem.Capability.ENTRIES,
        context -> FilesystemEffects.walk(filesystem, context, logical));
  }

  Pending<IFilesystem.Mutation> mkdir(
      String path,
      IFilesystem.MkdirOptions options,
      IFilesystem.MutationContext mutation) {
    String logical = HaraLogicalPath.normalise(path);
    IFilesystem.MkdirOptions validated = Objects.requireNonNull(options, "mkdir options");
    IFilesystem.MutationContext expected =
        mutation == null ? IFilesystem.MutationContext.none() : mutation;
    return submit(
        "mkdir",
        logical,
        null,
        IFilesystem.Capability.MKDIR,
        context -> filesystem.mkdir(context, logical, validated, expected));
  }

  Pending<IFilesystem.Mutation> delete(
      String path,
      IFilesystem.DeleteOptions options,
      IFilesystem.MutationContext mutation) {
    String logical = HaraLogicalPath.normalise(path);
    IFilesystem.DeleteOptions validated = Objects.requireNonNull(options, "delete options");
    IFilesystem.MutationContext expected =
        mutation == null ? IFilesystem.MutationContext.none() : mutation;
    return submit(
        "delete",
        logical,
        null,
        IFilesystem.Capability.DELETE,
        context -> filesystem.delete(context, logical, validated, expected));
  }

  Pending<IFilesystem.Mutation> copy(
      String source,
      String target,
      IFilesystem.CopyOptions options,
      IFilesystem.MutationContext mutation) {
    String logicalSource = HaraLogicalPath.normalise(source);
    String logicalTarget = HaraLogicalPath.normalise(target);
    IFilesystem.CopyOptions validated = Objects.requireNonNull(options, "copy options");
    IFilesystem.MutationContext expected =
        mutation == null ? IFilesystem.MutationContext.none() : mutation;
    if (validated.preserveModified()) {
      Pending<IFilesystem.Mutation> unsupported =
          rejectedCapability(
              IFilesystem.Capability.PRESERVE_MODIFIED,
              "copy",
              logicalSource,
              logicalTarget);
      if (unsupported != null) return unsupported;
    }
    return submit(
        "copy",
        logicalSource,
        logicalTarget,
        IFilesystem.Capability.COPY,
        context ->
            filesystem.copy(
                context,
                logicalSource,
                logicalTarget,
                validated,
                expected));
  }

  Pending<IFilesystem.Mutation> move(
      String source,
      String target,
      IFilesystem.MoveOptions options,
      IFilesystem.MutationContext mutation) {
    String logicalSource = HaraLogicalPath.normalise(source);
    String logicalTarget = HaraLogicalPath.normalise(target);
    IFilesystem.MoveOptions validated = Objects.requireNonNull(options, "move options");
    IFilesystem.MutationContext expected =
        mutation == null ? IFilesystem.MutationContext.none() : mutation;
    if (validated.atomic()) {
      Pending<IFilesystem.Mutation> unsupported =
          rejectedCapability(
              IFilesystem.Capability.ATOMIC_MOVE,
              "move",
              logicalSource,
              logicalTarget);
      if (unsupported != null) return unsupported;
    }
    return submit(
        "move",
        logicalSource,
        logicalTarget,
        IFilesystem.Capability.MOVE,
        context ->
            filesystem.move(
                context,
                logicalSource,
                logicalTarget,
                validated,
                expected));
  }

  Pending<String> tempFile(String parent, String prefix, String suffix) {
    String logical = HaraLogicalPath.normalise(parent);
    return submit(
        "temp-file",
        logical,
        null,
        IFilesystem.Capability.WRITE,
        context ->
            FilesystemEffects.tempFile(filesystem, context, logical, prefix, suffix));
  }

  Pending<String> tempDirectory(String parent, String prefix) {
    String logical = HaraLogicalPath.normalise(parent);
    return submit(
        "temp-directory",
        logical,
        null,
        IFilesystem.Capability.MKDIR,
        context ->
            FilesystemEffects.tempDirectory(filesystem, context, logical, prefix));
  }

  private <T> Pending<T> submit(
      String operation,
      String path,
      String target,
      IFilesystem.Capability capability,
      Invocation<T> invocation) {
    Objects.requireNonNull(invocation, "filesystem invocation");
    CompletableFuture<T> result = new CompletableFuture<>();
    try {
      requireCapability(capability, operation, path, target);
    } catch (Throwable error) {
      result.completeExceptionally(error);
      return new Pending<>(result, () -> false);
    }

    IFilesystem.CallContext context =
        IFilesystem.CallContext.create()
            .withTraceId(
                "filesystem/"
                    + descriptor.kind()
                    + "/"
                    + operation
                    + "/"
                    + sequence.getAndIncrement());
    ActiveCall call = new ActiveCall(context, result, operation, path, target);
    synchronized (lifecycle) {
      if (closed.get()) {
        result.completeExceptionally(closedFailure(operation, path, target));
        return new Pending<>(result, () -> false);
      }
      active.add(call);
    }
    result.whenComplete((value, error) -> active.remove(call));

    try {
      CompletionStage<T> stage =
          Objects.requireNonNull(invocation.invoke(context), "filesystem operation stage");
      stage.whenComplete(
          (value, error) -> {
            if (error == null) {
              result.complete(value);
            } else {
              result.completeExceptionally(mapFailure(error, operation, path, target));
            }
          });
    } catch (Throwable error) {
      result.completeExceptionally(mapFailure(error, operation, path, target));
    }
    return new Pending<>(result, () -> cancel(call));
  }

  private <T> Pending<T> rejectedCapability(
      IFilesystem.Capability capability,
      String operation,
      String path,
      String target) {
    try {
      requireCapability(capability, operation, path, target);
      return null;
    } catch (Throwable error) {
      CompletableFuture<T> rejected = new CompletableFuture<>();
      rejected.completeExceptionally(error);
      return new Pending<>(rejected, () -> false);
    }
  }

  private boolean cancel(ActiveCall call) {
    if (call.result.isDone()) return false;
    boolean requested = call.context.cancel();
    boolean settled =
        call.result.completeExceptionally(
            FilesystemException.cancelled(
                descriptor.kind(), call.operation, call.path, call.target));
    return requested || settled;
  }

  @Override
  public void close() {
    List<ActiveCall> calls;
    synchronized (lifecycle) {
      if (!closed.compareAndSet(false, true)) return;
      calls = List.copyOf(active);
      active.clear();
    }
    for (ActiveCall call : calls) {
      call.context.cancel();
      call.result.completeExceptionally(
          closedFailure(call.operation, call.path, call.target));
    }
  }

  private void requireCapability(
      IFilesystem.Capability capability,
      String operation,
      String path,
      String target) {
    if (closed.get()) throw closedFailure(operation, path, target);
    if (capability == null || descriptor.capabilities().contains(capability)) return;
    throw new FilesystemException(
        "unsupported",
        "filesystem provider does not advertise " + capability.keyword(),
        descriptor.kind(),
        operation,
        path,
        target,
        "capability-unavailable:" + capability.keyword(),
        false,
        null);
  }

  private FilesystemException closedFailure(String operation, String path, String target) {
    return FilesystemException.providerClosed(descriptor.kind(), operation, path, target);
  }

  private FilesystemException mapFailure(
      Throwable error, String operation, String path, String target) {
    Throwable cause = unwrap(error);
    if (cause instanceof FilesystemException filesystemFailure) return filesystemFailure;
    return new FilesystemException(
        "io",
        "filesystem operation failed",
        descriptor.kind(),
        operation,
        path,
        target,
        cause.getClass().getSimpleName(),
        false,
        cause);
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
