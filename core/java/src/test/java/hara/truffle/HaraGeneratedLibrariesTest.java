package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.PolyglotException;
import org.graalvm.polyglot.io.IOAccess;
import org.junit.Test;

public class HaraGeneratedLibrariesTest {
  @Test
  public void nestedLookupDoesNotConsumeItsPath() {
    try (Context context = context()) {
      assertEquals(
          ",",
          context
              .eval(HaraLanguage.ID, "(get-in {:default {:common {:sep \",\"}}} [:default :common :sep])")
              .asString());
    }
  }

  @Test
  public void emitterTypePredicatesAreAvailable() {
    try (Context context = context()) {
      assertTrue(context.eval(HaraLanguage.ID, "(char? \\a)").asBoolean());
      assertTrue(context.eval(HaraLanguage.ID, "(list? '(a b))").asBoolean());
      assertTrue(context.eval(HaraLanguage.ID, "(not (list? '[a b]))").asBoolean());
      assertTrue(context.eval(HaraLanguage.ID, "(map-entry? (first {:a 1}))").asBoolean());
      assertTrue(context.eval(HaraLanguage.ID, "(not (uuid? :a))").asBoolean());
      assertTrue(context.eval(HaraLanguage.ID, "(not (regexp? :a))").asBoolean());
      assertTrue(context.eval(HaraLanguage.ID, "(fn? (fn [value] value))").asBoolean());
      assertTrue(context.eval(HaraLanguage.ID, "(fn? :a)").asBoolean());
    }
  }

  @Test
  public void protocolPredicatesAndMapEntriesUseCanonicalCapabilities() {
    try (Context context = context()) {
      assertEquals(
          "[true true true true]",
          context
              .eval(
                  HaraLanguage.ID,
                  "[(coll? {}) (counted? []) (pair? (first {:a 1})) "
                      + "(map-entry? (first {:a 1}))]")
              .toString());
      assertEquals(
          ":std.native.Tuple",
          context.eval(HaraLanguage.ID, "(type (first {:a 1}))").toString());
      assertEquals(
          "[false true]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(defprotocol Ready (ready [self])) "
                      + "(defstruct Box [value]) "
                      + "(def before (satisfies? Ready (Box 1))) "
                      + "(extend-type Box Ready (ready [self] (:value self))) "
                      + "[before (satisfies? Ready (Box 1))]")
              .toString());
      assertErrorContains(context, "(collection? [])", "Unbound symbol");
    }
  }

  @Test
  public void portableTypeReturnsCanonicalAndNamedKeywords() {
    try (Context context = context()) {
      assertEquals(
          "[:std.native.Nil :std.native.Integer :std.native.Decimal :std.native.String :std.native.Keyword "
              + ":std.native.Symbol :std.native.Tuple :std.native.Vector :std.native.HashMap "
              + ":std.native.OrderedSet :std.native.Pointer :std.native.Function :std.native.Atom :std.native.Tuple "
              + ":std.native.Tuple :std.native.Vector :std.native.RegExp]",
          context
              .eval(
                  HaraLanguage.ID,
                  "[(type nil) (type 1) (type 1.5) (type \"x\") (type :x) "
                      + "(type 'x) (type []) (type (vector)) (type {}) "
                      + "(type #{}) (type #ptr {:context :kernel}) (type (fn [x] x)) "
                      + "(type (atom 0)) (std.foundation/type []) "
                      + "(type [1 2 3 4 5 6 7 8]) (type [1 2 3 4 5 6 7 8 9]) "
                      + "(type #\"x\")]")
              .toString());
      assertEquals(
          "[:geometry.Point :geometry.Cursor :std.native.StructType :std.native.MutableType]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns geometry) (defstruct Point [x y]) (defmutable Cursor [x y]) "
                      + "[(type (Point 1 2)) (type (Cursor 1 2)) (type Point) (type Cursor)]")
              .toString());
      assertEquals(
          "[true true true true false true false false]",
          context
              .eval(
                  HaraLanguage.ID,
                  "[(vector? []) (tuple? []) (pair? [1 2]) "
                      + "(tuple? [1 2 3 4 5 6 7 8]) (tuple? [1 2 3 4 5 6 7 8 9]) "
                      + "(vector? [1 2 3 4 5 6 7 8 9]) (pair? (vector 1 2)) "
                      + "(pair? (list 1 2))]")
              .toString());
      assertEquals(
          "[true 2 :missing]",
          context
              .eval(
                  HaraLanguage.ID,
                  "[(lookupable? [1 2]) (get [1 2] 1) (get [] 0 :missing)]")
              .toString());
    }
  }

  @Test
  public void typedSchemaValuesSeparateDataOriginsAndVarContracts() {
    try (Context context = context()) {
      assertEquals(
          "[:std.native.SchemaType true true :primitive true true true true true true true true true]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns schema.runtime) "
                      + "(def description [:int]) "
                      + "(defn ^{:schema #'description} customer-name [customer] (:name customer)) "
                      + "(def snapshot-description [:int]) "
                      + "(defn ^{:schema #'snapshot-description} snapshot-name [customer] (:name customer)) "
                      + "(def snapshot-description [:string]) "
                      + "(let [from-var (schema #'description) from-value (schema description) "
                      + "direct (schema [:int])] "
                      + "[(type direct) (= from-var from-value direct) "
                      + "(Schema/instance? direct) (Schema/kind direct) "
                      + "(= #'description (Schema/origin from-var)) "
                      + "(= from-var (schema-of #'customer-name)) "
                      + "(= direct (schema-of #'snapshot-name)) "
                      + "(= direct (schema {:kind :primitive :children [:int]})) "
                      + "(= [:int] (Schema/form direct)) (map? (Schema/ast direct)) "
                      + "(= direct (schema direct)) (= direct (schema :int)) "
                      + "(nil? (schema-of #'description))])")
              .toString());
      assertErrorContains(context, "(schema #'customer-name)", "schema expects schema data");
      assertErrorContains(context, "(schema customer-name)", "schema expects schema data");
      assertErrorContains(context, "(schema-of customer-name)", "schema-of expects a Var");
    }
  }

  @Test
  public void nativeTestCatalogUsesRuntimeRunnerAndTestContext() {
    try (Context context = context()) {
      assertEquals(
          "[true [:code.test :native] :code.test :test "
              + "[:test/run-started :test/fact-started :test/fact-completed :test/run-completed]]",
          context
              .eval(
                  HaraLanguage.ID,
                  "[(= Test std.native.Test) (get (Test/catalog) :runners) "
                      + "(get (Test/catalog) :default) (get (Test/catalog) :context) "
                      + "(Test/events)]")
              .toString());
      assertEquals(
          "[:code.test :fast :test :test :code.test]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(let [config (Test/config {:focus :fast}) "
                      + "context (Test/context config)] "
                      + "[(get config :runner) (get (get config :options) :focus) "
                      + "(IPointer/ptr-context context) (get context :id) "
                      + "(get (get context :config) :runner)])")
              .toString());
      assertErrorContains(
          context, "(Test/config {:runner :native})", "runner is owned by the runtime");
      assertEquals(
          "[{:pass true :actual {:a [1 2]} :expected {:a [1 2]} :name \"equal\"} true false]",
          context
              .eval(
                  HaraLanguage.ID,
                  "[(Test/result \"equal\" {:a [1 2]} {:a [1 2]}) "
                      + "(Test/passed? (Test/result \"equal\" 7 7)) "
                      + "(Test/passed? (Test/result \"different\" 7 8))]")
              .toString());
      assertErrorContains(context, "(Test/passed? {:status :error})", "test result map");
    }
    try (Context context =
        Context.newBuilder(HaraLanguage.ID).option("hara.TestRunner", "native").build()) {
      assertEquals(
          "[:native :native]",
          context
              .eval(
                  HaraLanguage.ID,
                  "[(get (Test/catalog) :runner) (get (Test/config) :runner)]")
              .toString());
    }
  }

  @Test
  public void nativeTestRunAccumulatesCasesAndKeepsErrorsLocal() {
    try (Context context = context()) {
      String first = context.eval(HaraLanguage.ID,
          "(Test/run [{:name \"one\" :test (fn [] (+ 1 1)) :expected 2}])").toString();
      assertTrue(first, first.contains(":name \"one\""));
      assertTrue(first, first.contains(":pass true"));
      String cumulative = context.eval(HaraLanguage.ID,
          "(Test/run [{:name \"two\" :test (fn [] (throw \"boom\")) :expected 2}])")
          .toString();
      assertTrue(cumulative, cumulative.contains(":name \"one\""));
      assertTrue(cumulative, cumulative.contains(":name \"two\""));
      assertTrue(cumulative, cumulative.contains(":status :error"));
      assertEquals(cumulative, context.eval(HaraLanguage.ID, "(Test/run [])").toString());
      String malformed = context.eval(HaraLanguage.ID, "(Test/run [{} 1])").toString();
      assertTrue(malformed, malformed.contains("case requires :test"));
      assertTrue(malformed, malformed.contains("case must be a map"));
    }
  }

  @Test
  public void nativeTestRunAcceptsAFunctionAwareChecker() {
    try (Context context = context()) {
      String checked = context.eval(HaraLanguage.ID,
          "(Test/run [{:name \"checked\" :meta {:refer (quote demo/value)} "
              + ":test (fn [] 7) :expected odd?}] "
              + "(fn [thunk expected] (let [actual (thunk)] "
              + "{:pass (expected actual) :actual actual :expected :predicate})))")
          .toString();
      assertTrue(checked, checked.contains(":name \"checked\""));
      assertTrue(checked, checked.contains(":pass true"));
      assertTrue(checked, checked.contains(":meta {:refer demo/value}"));

      String failures = context.eval(HaraLanguage.ID,
          "(Test/run [{:name \"throws\" :test (fn [] 1) :expected 1} "
              + "{:name \"continues\" :test (fn [] 2) :expected 2}] "
              + "(fn [thunk expected] (throw \"checker boom\")))")
          .toString();
      assertTrue(failures, failures.contains(":name \"throws\""));
      assertTrue(failures, failures.contains(":name \"continues\""));
      assertEquals(2, failures.split(":status :error", -1).length - 1);

      String malformed = context.eval(HaraLanguage.ID,
          "(Test/run [{:name \"malformed\" :test (fn [] 1) :expected 1}] "
              + "(fn [thunk expected] true))")
          .toString();
      assertTrue(malformed, malformed.contains("check function must return a result map"));
    }
  }

  @Test
  public void intrinsicsCanExcludeAndRenameGeneratedAliases() {
    try (Context context = context()) {
      assertEquals(
          "HARA",
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns app (:config {:intrinsics {:exclude [bytes] :alias {string text}}})) "
                      + "(text/upper \"hara\")")
              .asString());
      PolyglotException missing =
          assertThrows(
              PolyglotException.class,
              () -> context.eval(HaraLanguage.ID, "(bytes/count (bytes 1))"));
      assertTrue(missing.getMessage().contains("Unbound symbol: bytes/count"));
      assertEquals("x", context.eval(HaraLanguage.ID, "(str \"x\")").asString());
    }
  }

  @Test
  public void generatedLibrariesAlsoSupportRequireAsAndRefer() {
    try (Context context = context()) {
      assertEquals(
          "x",
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns app (:config {:intrinsics {:exclude [string]}}) "
                      + "(:require [std.foundation.string :as text :refer [trim]])) "
                      + "(trim (text/trim \" x \"))")
              .asString());
    }
  }

  @Test
  public void onlyPortableFoundationShorthandsAreAutomatic() {
    try (Context context = context()) {
      assertEquals(
          42,
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns sample.kernel) (def value 42) "
                      + "(ns app (:require [sample.kernel :as kernel])) kernel/value")
              .asLong());
      assertEquals("x", context.eval(HaraLanguage.ID, "(str \"x\")").asString());
      assertErrorContains(context, "(json/read \"null\")", "Unbound symbol: json/read");
    }
  }

  @Test
  public void definitionsShadowReferredVarsWithoutMutatingTheirOwners() {
    try (Context context = context()) {
      assertEquals(
          "[99 3]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns app (:config {:override [+]})) "
                      + "(defn + [a b] 99) [(+ 1 2) (std.foundation/+ 1 2)]")
              .toString());

      assertEquals(
          "[99 3]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns protected (:config {:blank true}) "
                      + "(:require [std.foundation :refer [+]])) "
                      + "(defn + [a b] 99) [(+ 1 2) (std.foundation/+ 1 2)]")
              .toString());
      assertEquals(
          "[42 0]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns protected-declare (:config {:blank true}) "
                      + "(:require [std.foundation :refer [identity]])) "
                      + "(declare identity) (defn identity [value] 42) "
                      + "[(identity 0) (std.foundation/identity 0)]")
              .toString());
    }
  }

  @Test
  public void configOverridesOmitSelectedFoundationVars() {
    try (Context context = context()) {
      assertEquals(
          42,
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns app.runtime (:config {:override [Runtime]})) "
                      + "(defstruct Runtime [value]) "
                      + "(:value (app.runtime/Runtime 42))")
              .asLong());
      assertErrorContains(
          context,
          "(ns legacy (:refer-clojure :exclude [Runtime]))",
          "Unsupported ns clause: :refer-clojure");
      assertErrorContains(
          context,
          "(ns contradictory (:config {:blank true :override [Runtime]}))",
          "cannot be combined with :override");
    }
  }

  @Test
  public void configExposeSelectsOnlyNamedFoundationVars() {
    try (Context context = context()) {
      assertEquals(
          42,
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns exposed (:config {:expose [identity]})) (identity 42)")
              .asLong());
      assertErrorContains(context, "(count [1 2])", "Unbound symbol: count");
      assertErrorContains(
          context,
          "(ns mixed (:config {:override [map] :expose [inc]}))",
          "cannot be combined with :expose");
    }
  }

  @Test
  public void requireExclusionsSurviveLoadingLaterSourceNamespaces() {
    try (Context context =
        Context.newBuilder(HaraLanguage.ID).allowIO(IOAccess.ALL).build()) {
      assertEquals(
          42,
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns app.require-order "
                      + "(:require [std.foundation :refer :all :exclude [filter]] "
                      + "          [std.work.protocol :as protocol])) "
                      + "(defn filter [value] 42) (filter :value)")
              .asLong());
    }
  }

  @Test
  public void lexicalBindingsShadowCallableFoundationVars() {
    try (Context context = context()) {
      assertEquals(
          "[99 77]",
          context
              .eval(
                  HaraLanguage.ID,
                  "[(let [+ (fn [a b] 99)] (+ 1 2)) "
                      + " (let [count (fn [value] 77)] (count [1 2 3]))]")
              .toString());
    }
  }

  @Test
  public void namespaceUseLoadsAndRefersVarsAndMacros() {
    try (Context context = context()) {
      assertEquals(
          84,
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns demo.use-lib) "
                      + "(def answer 42) "
                      + "(defmacro twice [form] `(+ ~form ~form)) "
                      + "(ns demo.use-app (:use demo.use-lib)) "
                      + "(twice answer)")
              .asLong());
      assertErrorContains(
          context,
          "(ns demo.bad-use (:use [demo.use-lib]))",
          ":use expects unqualified namespace symbols");
    }
  }

  @Test
  public void foundationNamespaceCombinesJavaAndHalSymbols() {
    try (Context context = context()) {
      assertEquals(
          -1,
          context
              .eval(
                  HaraLanguage.ID, "(ns app (:require [std.foundation :as core])) (core/bit-not 0)")
              .asLong());
      assertEquals(
          1,
          context.eval(HaraLanguage.ID, "(std.foundation/count [1])").asLong());
      assertEquals(
          42,
          context.eval(HaraLanguage.ID, "((std.foundation/comp inc inc) 40)").asLong());
    }
  }

  @Test
  public void intrinsicsRejectUnknownConflictingAndDuplicateConfiguration() {
    try (Context context = context()) {
      assertErrorContains(
          context,
          "(ns a (:config {:intrinsics {:exclude [unknown]}}))",
          "Unknown intrinsic library");
      assertErrorContains(
          context,
          "(ns b (:config {:intrinsics {:exclude [bytes] :alias {bytes data}}}))",
          "both excluded and aliased");
      assertErrorContains(
          context,
          "(ns c (:config {:intrinsics {:alias {string data bytes data}}}))",
          "Duplicate intrinsic alias target");
      assertErrorContains(
          context, "(ns d (:config {}) (:config {}))", "only one :config clause");
      assertErrorContains(
          context,
          "(ns e (:config {:intrinsics {:unexpected true}}))",
          "Unsupported :config :intrinsics option");
    }
  }

  @Test
  public void completionIncludesGeneratedAliasesAndMarkerMethods() {
    try (Context context = context()) {
      assertTrue(
          context
              .eval(HaraLanguage.ID, "(Iter/iter-any? (fn [x] (= x \"str/trim\")) (current-symbols))")
              .asBoolean());
      assertTrue(
          context
              .eval(HaraLanguage.ID, "(not (Iter/iter-any? (fn [x] (= x \"str/len\")) (current-symbols)))")
              .asBoolean());
      assertTrue(
          context
              .eval(HaraLanguage.ID, "(Iter/iter-any? (fn [x] (= x \"co/resume\")) (current-symbols))")
              .asBoolean());
      assertTrue(
          context
              .eval(HaraLanguage.ID, "(Iter/iter-any? (fn [x] (= x \"push-last\")) (current-symbols))")
              .asBoolean());
    }
  }

  @Test
  public void completionOnlyQualifiesSymbolsOwnedByRequiredAliases() {
    try (Context context = context()) {
      context.eval(HaraLanguage.ID, "(ns sample.walk) (def own-symbol 1)");
      context.eval(HaraLanguage.ID, "(ns user (:require [sample.walk :as walk]))");
      assertTrue(
          context
              .eval(
                  HaraLanguage.ID,
                  "(and (Iter/iter-any? (fn [x] (= x \"walk/own-symbol\")) (current-symbols)) "
                      + "     (not (Iter/iter-any? (fn [x] (= x \"walk/+\")) (current-symbols))) "
                      + "     (not (Iter/iter-any? (fn [x] (= x \"walk/ILookup\")) (current-symbols))))")
              .asBoolean());
    }
  }

  @Test
  public void dotCallsAreRestrictedToMarkedArraysAndObjects() {
    try (Context context = context()) {
      assertEquals(
          6,
          context
              .eval(HaraLanguage.ID, "(Arr/fold-left (array 1 2 3) (fn [out x] (+ out x)) 0)")
              .asLong());
      assertEquals(
          3,
          context
              .eval(
                  HaraLanguage.ID,
                  "(let [a (array 1 2 3 4)] (Arr/get (Arr/filter a (fn [x] (> x 2))) 0))")
              .asLong());
      assertEquals(
          42,
          context
              .eval(
                  HaraLanguage.ID,
                  "(let [o (object \"answer\" 41)] (Obj/set o \"answer\" 42) "
                      + "(Obj/get o \"answer\"))")
              .asLong());
      PolyglotException denied =
          assertThrows(
              PolyglotException.class, () -> context.eval(HaraLanguage.ID, "(. [1 2] (get 0))"));
      assertTrue(
          denied.getMessage().contains("only supported on values created by array or object"));
    }
  }

  @Test
  public void bitOperationsUseSignedThirtyTwoBitSemantics() {
    try (Context context = context()) {
      assertEquals(2, context.eval(HaraLanguage.ID, "(bit-and 6 3)").asLong());
      assertErrorContains(context, "(bit-and 7 3 1)", "expects two integers");
      assertErrorContains(context, "(bit-or 1 2 4)", "expects two integers");
      assertErrorContains(context, "(bit-xor 1 2 4)", "expects two integers");
      assertEquals(-1, context.eval(HaraLanguage.ID, "(bit-not 0)").asLong());
      assertEquals(-2, context.eval(HaraLanguage.ID, "(bit-shift-right -4 1)").asLong());
      assertEquals(-2147483648L, context.eval(HaraLanguage.ID, "(bit-shift-left 1 31)").asLong());
      assertEquals(1, context.eval(HaraLanguage.ID, "(bit-shift-left 1 0)").asLong());
      assertErrorContains(context, "(bit-shift-left 1 -1)", "distance must be in the range 0..31");
      assertErrorContains(context, "(bit-shift-right 1 32)", "distance must be in the range 0..31");
    }
  }

  private static Context context() {
    return Context.newBuilder(HaraLanguage.ID).build();
  }

  private static void assertErrorContains(Context context, String source, String message) {
    PolyglotException error =
        assertThrows(PolyglotException.class, () -> context.eval(HaraLanguage.ID, source));
    assertTrue(error.getMessage().contains(message));
  }
}
