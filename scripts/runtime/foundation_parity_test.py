import unittest

import foundation_parity as parity


class FoundationParityParserTest(unittest.TestCase):
    def test_extracts_namespace_and_public_definitions(self):
        source = '''
        (ns+ tahto.example)
        (defn public-fn [x] x)
        (defn- private-fn [] nil)
        (def ^{:doc "value"} public-value 1)
        (defn.pg typed-op [:uuid id] id)
        (defmethod dispatch :kind [_] nil)
        '''

        namespace, public = parity.namespace_and_publics(source)

        self.assertEqual("tahto.example", namespace)
        self.assertEqual(["public-fn", "public-value", "typed-op"], public)

    def test_top_level_scanner_ignores_comments_and_parentheses_in_strings(self):
        source = '''
        ;; (def fake 1)
        (ns xt.example)
        (def text "not a closing form )")
        (defn actual [] "(")
        '''

        namespace, public = parity.namespace_and_publics(source)

        self.assertEqual("xt.example", namespace)
        self.assertEqual(["actual", "text"], public)

    def test_namespace_mapping_changes_only_the_family_prefix(self):
        family = {"source_namespace": "tahto", "target_namespace": "lang"}
        self.assertEqual("lang.model.spec-js", parity.mapped_namespace("tahto.model.spec-js", family))

    def test_extracts_explicit_and_aggregate_intern_surfaces(self):
        source = '''
        (ns postgres.core)
        (f/intern-all postgres.core.builtin postgres.core.impl)
        (f/intern-in impl/t:select [query impl/q] app/app-create)
        '''

        namespace, public, intern_all = parity.namespace_surface(source)

        self.assertEqual("postgres.core", namespace)
        self.assertEqual(["app-create", "query", "t:select"], public)
        self.assertEqual(["postgres.core.builtin", "postgres.core.impl"], intern_all)

    def test_resolves_intern_all_transitively(self):
        entries = [
            {"namespace": "demo.base", "public": ["base"], "intern_all": []},
            {"namespace": "demo.mid", "public": ["mid"], "intern_all": ["demo.base"]},
            {"namespace": "demo.api", "public": [], "intern_all": ["demo.mid"]},
        ]

        parity.resolve_intern_all(entries)

        self.assertEqual(["base", "mid"], entries[2]["public"])

    def test_extracts_macros_and_dependencies_for_complete_inventory(self):
        source = """
        (ns tahto.example
          (:require [tahto.base :as base] [external.util :as util]))
        (defmacro when-ready [value] value)
        (defn run [value] (base/identity value))
        """

        self.assertEqual(["when-ready"], parity.macro_surface(source))
        self.assertEqual(
            ["external.util", "tahto.base"],
            parity.required_namespaces(source),
        )

    def test_dependency_components_and_ranks_are_stable(self):
        graph = {
            "demo.api": ["demo.mid"],
            "demo.mid": ["demo.base"],
            "demo.base": ["demo.mid"],
        }

        components, owners, ranks = parity.dependency_components(graph)

        self.assertEqual([["demo.api"], ["demo.base", "demo.mid"]], components)
        self.assertEqual(0, ranks[owners["demo.base"]])
        self.assertEqual(1, ranks[owners["demo.api"]])

    def test_maps_reference_tests_to_target_tests(self):
        family = {
            "source_root": "src/tahto",
            "reference_test_roots": ["test/tahto"],
            "target_test_root": "core/lib/test-lang/lang",
        }

        self.assertEqual(
            ["test/tahto/model/example_test.clj"],
            parity.reference_test_candidates("src/tahto/model/example.clj", family),
        )
        self.assertEqual(
            "core/lib/test-lang/lang/model/example_test.hal",
            parity.target_test_path("src/tahto/model/example.clj", family),
        )


if __name__ == "__main__":
    unittest.main()
