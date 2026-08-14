import copy
import json
import tempfile
import unittest
from pathlib import Path

import clj_hal_corpus as corpus
import foundation_script_inventory as script_inventory


class CorpusParserTest(unittest.TestCase):
    def test_extracts_namespace_dependencies_and_publics(self):
        source = '''
        ;; (ns fake.example)
        (ns+ std.example
          (:require [clojure.string :as str]
                    [std.block.base :as base])
          (:use [std.legacy.helper]))
        (defn public-fn [x] x)
        (defn- private-fn [] nil)
        (def ^{:doc "value"} public-value 1)
        (defmulti dispatch identity)
        (defmethod dispatch :kind [_] nil)
        '''
        self.assertEqual(
            {
                "namespace": "std.example",
                "dependencies": [
                    "clojure.string",
                    "std.block.base",
                    "std.legacy.helper",
                ],
                "public_symbols": ["dispatch", "public-fn", "public-value"],
            },
            corpus.source_surface(source),
        )

    def test_scanner_ignores_comments_and_parentheses_in_strings(self):
        surface = corpus.source_surface('''
        ;; (def fake 1)
        (ns std.example)
        (def text "not a closing form )")
        (defn actual [] "(")
        ''')
        self.assertEqual(["actual", "text"], surface["public_symbols"])

    def test_scanner_accepts_quoted_jvm_type_hints(self):
        surface = corpus.source_surface('''
        (ns std.example)
        (defn ^"[B" decode [value] value)
        ''')
        self.assertEqual(["decode"], surface["public_symbols"])


class SymbolRouteTest(unittest.TestCase):
    def test_override_inherits_reviewed_namespace_defaults(self):
        declaration = {
            "default": {
                "state": "missing",
                "safety": "review",
                "message": "Port the codec.",
            },
            "symbols": {
                "encode": {
                    "target_namespace": "std.codec.base64",
                    "target_symbol": "encode",
                },
            },
        }
        self.assertEqual(
            {
                "state": "missing",
                "safety": "review",
                "message": "Port the codec.",
                "target_namespace": "std.codec.base64",
                "target_symbol": "encode",
                "source_symbol": "encode",
            },
            corpus.route_for("encode", declaration),
        )

    def test_native_route_preserves_symbol_destination_without_a_hal_path(self):
        declaration = {
            "default": {
                "state": "deferred",
                "safety": "manual",
                "message": "Review the helper.",
            },
            "symbols": {
                "var-sym": {
                    "state": "implemented",
                    "safety": "review",
                    "target_kind": "native",
                    "target_namespace": "std.foundation",
                    "target_symbol": "var-sym",
                },
            },
        }
        self.assertEqual(
            {
                "state": "implemented",
                "safety": "review",
                "message": "Review the helper.",
                "target_kind": "native",
                "target_namespace": "std.foundation",
                "target_symbol": "var-sym",
                "source_symbol": "var-sym",
            },
            corpus.route_for("var-sym", declaration),
        )


class DependencyPlanTest(unittest.TestCase):
    def test_orders_roots_before_dependants(self):
        compiled = corpus.compile_entries([
            {"namespace": "demo.top", "dependencies": ["demo.mid"]},
            {"namespace": "demo.root", "dependencies": []},
            {"namespace": "demo.mid", "dependencies": ["demo.root"]},
        ])
        self.assertEqual(
            [("demo.root", 0), ("demo.mid", 1), ("demo.top", 2)],
            [(entry["namespace"], entry["dependency_rank"]) for entry in compiled],
        )

    def test_groups_cycles_explicitly(self):
        compiled = corpus.compile_entries([
            {"namespace": "demo.a", "dependencies": ["demo.b"]},
            {"namespace": "demo.b", "dependencies": ["demo.a"]},
            {"namespace": "demo.consumer", "dependencies": ["demo.a"]},
        ])
        by_name = {entry["namespace"]: entry for entry in compiled}
        self.assertEqual(["demo.a", "demo.b"], by_name["demo.a"]["dependency_component"])
        self.assertTrue(by_name["demo.a"]["dependency_cycle"])
        self.assertEqual(0, by_name["demo.a"]["dependency_rank"])
        self.assertEqual(1, by_name["demo.consumer"]["dependency_rank"])


class CorpusValidationTest(unittest.TestCase):
    def fixture(self):
        entries = corpus.compile_entries([
            {
                "namespace": "demo.root",
                "source_path": "src/demo/root.clj",
                "source_blob": "1" * 40,
                "target_namespace": "demo.root",
                "target_path": "core/lib/src/demo/root.hal",
                "target_blob": "2" * 40,
                "status": "ported",
                "dependencies": [],
            },
            {
                "namespace": "demo.top",
                "source_path": "src/demo/top.clj",
                "source_blob": "3" * 40,
                "target_namespace": "demo.top",
                "target_path": None,
                "target_blob": None,
                "status": "missing",
                "dependencies": ["demo.root"],
            },
        ])
        return {
            "schema_version": 1,
            "reference": {"repository": "example/foundation", "commit": "4" * 40},
            "target": {"repository": "example/hara", "base_commit": "5" * 40},
            "status_policy": {"allowed": ["ported", "missing"]},
            "namespaces": entries,
            "inventory_sha256": corpus.checksum(entries),
        }

    def test_validates_a_compiled_corpus(self):
        fixture = self.fixture()
        self.assertEqual(fixture["namespaces"], corpus.validate(fixture))

    def test_rejects_stale_dependency_ranks(self):
        fixture = self.fixture()
        fixture["namespaces"][1]["dependency_rank"] = 0
        fixture["inventory_sha256"] = corpus.checksum(fixture["namespaces"])
        with self.assertRaisesRegex(corpus.CorpusError, "not deterministic"):
            corpus.validate(fixture)

    def test_rejects_a_stale_checksum(self):
        fixture = self.fixture()
        fixture["inventory_sha256"] = "0" * 64
        with self.assertRaisesRegex(corpus.CorpusError, "checksum is stale"):
            corpus.validate(fixture)

    def test_main_summarises_a_fixture(self):
        fixture = self.fixture()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "corpus.json"
            path.write_text(json.dumps(fixture), encoding="utf-8")
            self.assertEqual(0, corpus.main(["--corpus", str(path)]))

    def test_v2_requires_one_valid_route_per_public_symbol(self):
        entry = corpus.compile_entries([{
            "namespace": "demo.root",
            "source_path": "src/demo/root.clj",
            "source_blob": "1" * 40,
            "status": "reviewed",
            "dependencies": [],
            "public_symbols": ["run"],
            "symbol_routes": [],
        }])[0]
        fixture = {
            "schema_version": 2,
            "reference": {"repository": "example/foundation", "commit": "4" * 40},
            "target": {"repository": "example/hara", "base_commit": "5" * 40},
            "route_policy": {"states": ["missing"], "safety": ["review"]},
            "namespaces": [entry],
            "inventory_sha256": corpus.checksum([entry]),
        }
        with self.assertRaisesRegex(corpus.CorpusError, "coverage is incomplete"):
            corpus.validate(fixture)

    def test_v2_accepts_an_implemented_native_symbol_route(self):
        entry = corpus.compile_entries([{
            "namespace": "demo.root",
            "source_path": "src/demo/root.clj",
            "source_blob": "1" * 40,
            "status": "reviewed",
            "dependencies": [],
            "public_symbols": ["native-call"],
            "symbol_routes": [{
                "source_symbol": "native-call",
                "state": "implemented",
                "safety": "review",
                "target_kind": "native",
                "target_namespace": "std.foundation",
                "target_symbol": "native-call",
            }],
        }])[0]
        fixture = {
            "schema_version": 2,
            "reference": {"repository": "example/foundation", "commit": "4" * 40},
            "target": {"repository": "example/hara", "base_commit": "5" * 40},
            "route_policy": {"states": ["implemented"], "safety": ["review"]},
            "namespaces": [entry],
            "inventory_sha256": corpus.checksum([entry]),
        }
        self.assertEqual([entry], corpus.validate(fixture))


class FoundationScriptInventoryIntegrationTest(unittest.TestCase):
    def test_pinned_script_inventory_is_current(self):
        reference = Path(".foundation-reference")
        if not reference.is_dir():
            self.skipTest("pinned Foundation checkout is not present")
        policy = script_inventory.load(script_inventory.DEFAULT_POLICY)
        generated = script_inventory.generate(
            policy,
            reference,
            script_inventory.ROOT,
        )
        script_inventory.validate(generated)
        expected = json.dumps(generated, indent=2, sort_keys=True) + "\n"
        actual = script_inventory.DEFAULT_OUTPUT.read_text(encoding="utf-8")
        self.assertEqual(expected, actual)


if __name__ == "__main__":
    unittest.main()
