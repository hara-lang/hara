package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;
import static org.junit.Assert.fail;

import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.PolyglotException;
import org.junit.Test;

public class StdTypedCatalogTest {
  private static final String GOLDEN_VALUE_HASH =
      "sha256:3fc60b1736332b9f2e9f9e0a7dee75cc19c6287cc4e066970ef97b23a75fd34a";

  private static void installCatalogFixtures(Context context) {
    context.eval(
        HaraLanguage.ID,
        """
        (ns typed-catalog-truffle-probe
          (:require [std.typed.catalog :as catalog]
                    [std.typed.catalog.codec :as codec]))
        (def base
          (catalog/catalog
           [{:schema/id :model/id
             :schema/version 1
             :schema/form :int}
            {:schema/id :model/status
             :schema/version 1
             :schema/form '[:enum :active :disabled]}]
           {:namespace 'model}))
        (def application
          (catalog/catalog
           [{:schema/id :app/user
             :schema/version 1
             :schema/form
             '[:map
               [:id (var id)]
               [:status (var m/status)]]}
            {:schema/id :app/user
             :schema/version 2
             :schema/form
             '[:map
               {:title "User record" :owner :accounts}
               [:id (var id)]
               [:status (var m/status)]
               [:email {:optional true} :str]]}]
           {:namespace 'app
            :aliases {'m 'model}
            :refers {'id 'model/id}
            :parents [base]}))
        (def recursive
          (catalog/catalog
           [{:schema/id :tree/node
             :schema/version 1
             :schema/form
             '[:map
               [:value :int]
               [:children [:vector (var node)]]]}]
           {:namespace 'tree}))
        """);
  }

  @Test
  public void portableCatalogHashLookupResolutionAndGraphAreCanonical() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      installCatalogFixtures(context);
      assertEquals(
          "[diagnostic]",
          context
              .eval(
                  HaraLanguage.ID,
                  "[(codec/content-hash :demo/value 1 :int) "
                      + " (codec/content-hash :demo/value 1 [:int]) "
                      + " (codec/normalize-content :int (catalog/schema-registry base)) "
                      + " (codec/normalize-content [:int] (catalog/schema-registry base)) "
                      + " (:schema/ast (catalog/lookup application :app/user 2)) "
                      + " (std.typed.catalog.ast/reference-names "
                      + "  (:schema/ast (catalog/lookup application :app/user 2))) "
                      + " (:schema/ast (catalog/lookup recursive :tree/node)) "
                      + " (std.typed.catalog.ast/reference-names "
                      + "  (:schema/ast (catalog/lookup recursive :tree/node)))]")
              .toString());
    }
  }

  @Test
  public void portableCatalogRejectsAliasOnlyCycles() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      installCatalogFixtures(context);
      try {
        context.eval(
            HaraLanguage.ID,
            "(catalog/catalog "
                + " [{:schema/id :cycle/a :schema/version 1 :schema/form 'b} "
                + "  {:schema/id :cycle/b :schema/version 1 :schema/form 'a}] "
                + " {:namespace 'cycle})");
        fail("alias-only catalog cycle was accepted");
      } catch (PolyglotException error) {
        assertTrue(
            "unexpected catalog error: " + error.getMessage(),
            error.getMessage().contains("alias-only cycle"));
      }
    }
  }
}
