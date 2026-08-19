package hara.truffle;

import hara.lang.data.Symbol;

/** Optimized public collection operations shared by core and HAL libraries. */
public final class StdFoundationCollection {
  private StdFoundationCollection() {}

  public static Object remove(HaraContext context, Object[] values) {
    return context.removeValues(values);
  }

  @HaraExport(
      name = "var-sym",
      doc = "converts a var to a symbol",
      arglists = {"[var]"})
  public static Object varSymbol(HaraContext context, Object[] values) {
    if (values.length != 1 || !(HaraBox.unwrap(values[0]) instanceof HaraVar variable)) {
      throw new HaraException("var-sym expects a var");
    }
    return Symbol.create(variable.namespaceName(), variable.symbolName());
  }
}
