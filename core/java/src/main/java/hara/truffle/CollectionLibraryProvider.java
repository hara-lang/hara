package hara.truffle;

/** Lazy Java implementation of {@code std.lib.collection}. */
public final class CollectionLibraryProvider implements HaraLibraryProvider {
  @Override
  public String namespace() {
    return "std.lib.collection";
  }

  @Override
  public int order() {
    return 30;
  }

  @Override
  public void install(HaraContext context) {
    context.collectBuiltins(
        namespace(), () -> HaraStaticLibrary.install(context, namespace(), StdLibCollection.class));
  }
}
