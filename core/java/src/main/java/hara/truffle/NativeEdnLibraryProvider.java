package hara.truffle;

/** Eager native implementation of {@code std.native.Edn}. */
public final class NativeEdnLibraryProvider implements HaraLibraryProvider {
  @Override
  public String namespace() {
    return "std.native.Edn";
  }

  @Override
  public int order() {
    return 20;
  }

  @Override
  public boolean eager() {
    return true;
  }

  @Override
  public void install(HaraContext context) {}
}
