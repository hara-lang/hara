package hara.truffle;

/** Eager native substrate used by the source-owned Foundation root. */
public final class FoundationLibraryProvider implements HaraLibraryProvider {
  @Override
  public String namespace() {
    return "std.native";
  }

  @Override
  public int order() {
    return 5;
  }

  @Override
  public boolean eager() {
    return true;
  }

  @Override
  public void install(HaraContext context) {
    context.collectBuiltins("std.foundation", () -> {
      HaraStaticLibrary.install(context, "std.foundation", StdFoundationCollection.class);
    });
  }
}
