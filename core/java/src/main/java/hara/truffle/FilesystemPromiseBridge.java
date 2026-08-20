package hara.truffle;

import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.function.Function;

/** Converts one provider-runtime call into the existing native Hara Promise implementation. */
final class FilesystemPromiseBridge {
  @FunctionalInterface
  interface Factory {
    Object create(CompletableFuture<Object> future, Runnable cancellation);
  }

  private FilesystemPromiseBridge() {}

  static <T> Object bind(
      HaraContext context,
      FilesystemRuntimeBinding.Pending<T> pending,
      Function<? super T, ?> transform) {
    Objects.requireNonNull(context, "Hara context");
    return bind(context::cancellablePromise, pending, transform);
  }

  static <T> Object bind(
      Factory factory,
      FilesystemRuntimeBinding.Pending<T> pending,
      Function<? super T, ?> transform) {
    Objects.requireNonNull(factory, "filesystem promise factory");
    Objects.requireNonNull(pending, "filesystem pending call");
    Objects.requireNonNull(transform, "filesystem result transform");
    CompletableFuture<Object> future =
        pending.future().thenApply(value -> transform.apply(value));
    return factory.create(future, () -> pending.cancel());
  }

  static Object bind(
      HaraContext context, FilesystemRuntimeBinding.Pending<?> pending) {
    return bind(context, pending, value -> value);
  }
}
