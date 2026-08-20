package hara.truffle;

/** Native Coroutine substrate used by the source-owned Foundation coroutine library. */
public final class CoroutineLibraryProvider implements HaraLibraryProvider {
  @Override
  public String namespace() {
    return "std.native.Coroutine";
  }

  @Override
  public int order() {
    return 30;
  }

  @Override
  public void install(HaraContext context) {
    context.collectBuiltins("std.foundation.coroutine", () -> {
      StdFoundationCoroutine.install(context, "std.foundation.coroutine");
    });
  }
}
