package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertThrows;

import org.graalvm.polyglot.Context;
import org.junit.Test;

public class StdLibCollectionTest {
  @Test
  public void ownsSpecialisedPersistentCollectionConstructors() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      context.eval(
          HaraLanguage.ID,
          "(ns collection.java (:require [std.lib.collection :as collection]))");

      assertEquals(
          "[:hara/OrderedMap :hara/OrderedSet :hara/Queue :hara/SortedMap :hara/SortedSet :hara/Trie]",
          context
              .eval(
                  HaraLanguage.ID,
                  "[(type (collection/ordered-map :a 1))"
                      + " (type (collection/ordered-set 1))"
                      + " (type (collection/queue 1))"
                      + " (type (collection/sorted-map :b 2 :a 1))"
                      + " (type (collection/sorted-set 2 1))"
                      + " (type (collection/trie \"alpha\" 7))]")
              .toString());
      assertEquals(
          "[true true true true true true false]",
          context
              .eval(
                  HaraLanguage.ID,
                  "[(collection/ordered-map? (collection/ordered-map))"
                      + " (collection/ordered-set? (collection/ordered-set))"
                      + " (collection/queue? (collection/queue))"
                      + " (collection/sorted-map? (collection/sorted-map))"
                      + " (collection/sorted-set? (collection/sorted-set))"
                      + " (collection/trie? (collection/trie))"
                      + " (collection/trie? {})]")
              .toString());
      assertEquals(
          "[[:b :a] [:a :b] 5 7]",
          context
              .eval(
                  HaraLanguage.ID,
                  "[(keys (collection/ordered-map :b 2 :a 1))"
                      + " (keys (collection/sorted-map :b 2 :a 1))"
                      + " (nth (collection/queue 4 5 6) 1)"
                      + " (get (collection/trie \"alpha\" 7) \"alpha\")]")
              .toString());
      assertEquals(
          "std.lib.collection/ordered-map",
          context
              .eval(
                  HaraLanguage.ID,
                  "(str (var-sym (resolve (quote collection/ordered-map))))")
              .asString());

      assertThrows(RuntimeException.class, () -> context.eval(HaraLanguage.ID, "(ordered-map)"));
      assertThrows(
          RuntimeException.class,
          () -> context.eval(HaraLanguage.ID, "(std.foundation/ordered-map)"));
      assertThrows(
          RuntimeException.class,
          () -> context.eval(HaraLanguage.ID, "(collection/trie :alpha 7)"));
    }
  }
}
