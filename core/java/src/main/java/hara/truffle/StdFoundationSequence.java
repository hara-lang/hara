package hara.truffle;

/** Optimized public sequence operations backed by the Truffle iterator runtime. */
public final class StdFoundationSequence {
  private StdFoundationSequence() {}

  public static Object map(HaraContext context, Object[] values) {
    return context.mapValues(values);
  }

  @HaraExport(
      name = "reduce",
      doc = "Reduces a collection with function and an optional initial value.",
      arglists = {"[function value]", "[function initial value]"})
  public static Object reduce(HaraContext context, Object[] values) {
    return context.reduceIterator(values);
  }

  public static Object cycle(HaraContext context, Object[] values) {
    HaraContext.requireMethodArity("cycle", values, 1);
    return context.seqValue(new Object[] {context.iterCycle(values[0])});
  }

  public static Object partition(HaraContext context, Object[] values) {
    return context.partitionValues(values, false);
  }

  public static Object partitionAll(HaraContext context, Object[] values) {
    return context.partitionValues(values, true);
  }

  public static Object filter(HaraContext context, Object[] values) {
    return context.filterValues(values);
  }

  public static Object take(HaraContext context, Object[] values) {
    return context.takeValues(values);
  }

  public static Object drop(HaraContext context, Object[] values) {
    return context.dropValues(values);
  }
}
