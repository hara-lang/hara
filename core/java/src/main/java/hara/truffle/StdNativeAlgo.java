package hara.truffle;

/** Native implementation owner for specialised persistent collection constructors. */
public final class StdNativeAlgo {
  private StdNativeAlgo() {}

  @HaraExport(name = "deque", arglists = {"[& values]"})
  public static Object deque(HaraContext context, Object[] values) {
    return StdLibCollection.deque(context, values);
  }

  @HaraExport(name = "ordered-map", arglists = {"[& entries]"})
  public static Object orderedMap(HaraContext context, Object[] values) {
    return StdLibCollection.orderedMap(context, values);
  }

  @HaraExport(name = "ordered-set", arglists = {"[& values]"})
  public static Object orderedSet(HaraContext context, Object[] values) {
    return StdLibCollection.orderedSet(context, values);
  }

  @HaraExport(name = "priority-map", arglists = {"[& entries]"})
  public static Object priorityMap(HaraContext context, Object[] values) {
    return StdLibCollection.priorityMap(context, values);
  }

  @HaraExport(name = "queue", arglists = {"[& values]"})
  public static Object queue(HaraContext context, Object[] values) {
    return StdLibCollection.queue(context, values);
  }

  @HaraExport(name = "sorted-map", arglists = {"[& entries]"})
  public static Object sortedMap(HaraContext context, Object[] values) {
    return StdLibCollection.sortedMap(context, values);
  }

  @HaraExport(name = "sorted-set", arglists = {"[& values]"})
  public static Object sortedSet(HaraContext context, Object[] values) {
    return StdLibCollection.sortedSet(context, values);
  }

  @HaraExport(name = "trie", arglists = {"[& entries]"})
  public static Object trie(HaraContext context, Object[] values) {
    return StdLibCollection.trie(context, values);
  }
}
