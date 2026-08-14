package hara.truffle;

import hara.kernel.builtin.BuiltinStruct;
import hara.lang.base.Iter;
import hara.lang.data.Trie;
import java.util.Iterator;
import java.util.Map.Entry;

/** Native constructors owned by {@code std.lib.collection}. */
public final class StdLibCollection {
  private StdLibCollection() {}

  @HaraExport(name = "deque", doc = "Creates a persistent finger-tree deque.", arglists = {"[& values]"})
  public static Object deque(HaraContext context, Object[] values) {
    return BuiltinStruct.deque(values);
  }

  @HaraExport(name = "ordered-map", doc = "Creates an insertion-ordered persistent map.", arglists = {"[& entries]"})
  public static Object orderedMap(HaraContext context, Object[] values) {
    return BuiltinStruct.orderedMap(values);
  }

  @HaraExport(name = "ordered-set", doc = "Creates an insertion-ordered persistent set.", arglists = {"[& values]"})
  public static Object orderedSet(HaraContext context, Object[] values) {
    return BuiltinStruct.orderedSet(values);
  }

  @HaraExport(name = "priority-map", doc = "Creates a stable persistent priority map.", arglists = {"[& entries]"})
  public static Object priorityMap(HaraContext context, Object[] values) {
    return BuiltinStruct.priorityMap(values);
  }

  @HaraExport(name = "queue", doc = "Creates a persistent queue.", arglists = {"[& values]"})
  public static Object queue(HaraContext context, Object[] values) {
    return BuiltinStruct.queue(values);
  }

  @HaraExport(name = "sorted-map", doc = "Creates a key-sorted persistent map.", arglists = {"[& entries]"})
  public static Object sortedMap(HaraContext context, Object[] values) {
    return BuiltinStruct.sortedMap(values);
  }

  @HaraExport(name = "sorted-set", doc = "Creates a value-sorted persistent set.", arglists = {"[& values]"})
  public static Object sortedSet(HaraContext context, Object[] values) {
    return BuiltinStruct.sortedSet(values);
  }

  @HaraExport(name = "trie", doc = "Creates a persistent trie from string key/value entries.", arglists = {"[& entries]"})
  @SuppressWarnings({"rawtypes", "unchecked"})
  public static Object trie(HaraContext context, Object[] values) {
    Trie<Object> trie = new Trie.Standard<>();
    Iterator<Entry> entries = Iter.partitionPair(Iter.iter(values));
    while (entries.hasNext()) {
      Entry entry = entries.next();
      Object key = HaraBox.unwrap(entry.getKey());
      if (!(key instanceof String)) {
        throw new HaraException("trie expects string keys");
      }
      trie = trie.assoc((String) key, HaraBox.unwrap(entry.getValue()));
    }
    return trie;
  }
}
