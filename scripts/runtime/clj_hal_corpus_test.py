import copy
import json
import subprocess
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

    def test_scanner_lexes_character_literals_without_changing_form_depth(self):
        surface = corpus.source_surface(r'''
        (ns std.example)
        (def close-paren \))
        (def open-paren \()
        (def semicolon \;)
        (def quote-char \")
        (defn actual [] :ok)
        ''')
        self.assertEqual(
            ["actual", "close-paren", "open-paren", "quote-char", "semicolon"],
            surface["public_symbols"],
        )

    def test_scanner_recognises_intern_in_and_renamed_exports(self):
        surface = corpus.source_surface('''
        (ns std.aggregate
          (:require [std.block.base :as base]
                    [std.lib.foundation :as f]))
        (def original 1)
        (f/intern-in base/block? [type base/block-type] [renamed original])
        ''')
        self.assertEqual(
            ["block?", "original", "renamed", "type"],
            surface["public_symbols"],
        )

    def test_scanner_does_not_claim_exports_in_an_explicit_destination(self):
        surface = corpus.source_surface('''
        (ns std.aggregate
          (:require [std.block.base :as base]
                    [std.lib.foundation :as f]))
        (f/intern-in another.namespace base/block?)
        ''')
        self.assertEqual([], surface["public_symbols"])

    def test_scanner_claims_exports_for_the_current_explicit_destination(self):
        surface = corpus.source_surface('''
        (ns std.aggregate
          (:require [std.block.base :as base]
                    [std.lib.foundation :as f]))
        (f/intern-in std.aggregate base/block? [type base/block-type])
        ''')
        self.assertEqual(["block?", "type"], surface["public_symbols"])

    def test_scanner_recognises_intern_all_exports(self):
        surface = corpus.source_surface(
            '''
            (ns std.aggregate
              (:require [std.shared :as shared]
                        [std.lib.foundation :as f]))
            (f/intern-all shared)
            ''',
            lambda namespace: ["alpha", "beta"] if namespace == "std.shared" else [],
        )
        self.assertEqual(["alpha", "beta"], surface["public_symbols"])

    def test_scanner_resolves_intern_all_to_a_fixed_point(self):
        surfaces = corpus.resolve_source_surfaces({
            "src/demo/base.clj": '''
              (ns demo.base)
              (defn alpha [] true)
              (def beta 2)
            ''',
            "src/demo/middle.clj": '''
              (ns demo.middle
                (:require [demo.base]
                          [std.lib.foundation :as f]))
              (f/intern-all demo.base)
            ''',
            "src/demo/facade.clj": '''
              (ns demo.facade
                (:require [demo.middle]
                          [std.lib.foundation :as f]))
              (f/intern-all demo.middle)
            ''',
        })
        self.assertEqual(
            ["alpha", "beta"],
            surfaces["src/demo/facade.clj"]["public_symbols"],
        )

    def test_repository_catalog_resolves_cycles_and_transitive_exports(self):
        with tempfile.TemporaryDirectory() as directory:
            reference = Path(directory)
            sources = {
                "src/demo/a.clj": '''
                  (ns demo.a
                    (:require [demo.b]
                              [demo.extra]
                              [std.lib.foundation :as f]))
                  (defn alpha [] true)
                  (f/intern-all demo.b demo.extra)
                ''',
                "src/demo/b.clj": '''
                  (ns demo.b
                    (:require [demo.a]
                              [std.lib.foundation :as f]))
                  (defn beta [] true)
                  (f/intern-all demo.a)
                ''',
                "src/demo/extra.clj": '''
                  (ns demo.extra)
                  (defn gamma [] true)
                ''',
            }
            for name, content in sources.items():
                path = reference / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content, encoding="utf-8")
            subprocess.run(["git", "init", "-q", str(reference)], check=True)
            subprocess.run(
                ["git", "-C", str(reference), "config", "user.name", "Corpus Test"],
                check=True,
            )
            subprocess.run(
                [
                    "git", "-C", str(reference), "config", "user.email",
                    "test@example.invalid",
                ],
                check=True,
            )
            subprocess.run(["git", "-C", str(reference), "add", "-A"], check=True)
            subprocess.run(
                ["git", "-C", str(reference), "commit", "-q", "-m", "fixture"],
                check=True,
            )
            commit = subprocess.check_output(
                ["git", "-C", str(reference), "rev-parse", "HEAD"], text=True
            ).strip()
            catalog = corpus.repository_source_catalog(reference, commit)
            self.assertEqual(
                ["alpha", "beta", "gamma"],
                catalog["src/demo/b.clj"]["public_symbols"],
            )

    def test_pinned_block_family_inventory_is_exact(self):
        reference = Path(".foundation-reference")
        if not reference.is_dir():
            self.skipTest("pinned Foundation checkout is not present")
        paths = [
            reference / "src/std/block.clj",
            *sorted((reference / "src/std/block").rglob("*.clj")),
            reference / "src/std/protocol/block.clj",
        ]
        sources = {
            path.relative_to(reference).as_posix(): path.read_text(encoding="utf-8")
            for path in paths
        }
        surfaces = corpus.resolve_source_surfaces(sources)
        self.assertEqual(21, len(surfaces))
        self.assertEqual(
            412,
            sum(len(surface["public_symbols"]) for surface in surfaces.values()),
        )

        routes = json.loads(corpus.DEFAULT_ROUTES.read_text(encoding="utf-8"))
        declarations = corpus.route_declarations(
            routes,
            reference,
            routes["reference"]["commit"],
            surfaces,
        )
        declarations = [
            declaration
            for declaration in declarations
            if declaration["namespace"].startswith("std.block")
            or declaration["namespace"] == "std.protocol.block"
        ]
        self.assertEqual(21, len(declarations))

        by_namespace = {
            declaration["namespace"]: declaration
            for declaration in declarations
        }
        state_counts = {}
        for surface in surfaces.values():
            declaration = by_namespace[surface["namespace"]]
            for symbol in surface["public_symbols"]:
                state = corpus.route_for(symbol, declaration)["state"]
                state_counts[state] = state_counts.get(state, 0) + 1
        self.assertEqual(
            {
                "host-only": 1,
                "implemented": 1,
                "missing": 402,
                "obsolete": 8,
            },
            state_counts,
        )


class FamilyDiscoveryTest(unittest.TestCase):
    def test_discovers_family_roots_outside_src_std_lib(self):
        with tempfile.TemporaryDirectory() as directory:
            reference = Path(directory)
            source = reference / "src/std/block/base.clj"
            source.parent.mkdir(parents=True)
            source.write_text(
                "(ns std.block.base)\n(defn block? [value] value)\n",
                encoding="utf-8",
            )
            root = reference / "src/std/block.clj"
            root.write_text(
                "(ns std.block)\n(defn block [value] value)\n",
                encoding="utf-8",
            )
            adjacent = reference / "src/std/blockish.clj"
            adjacent.write_text(
                "(ns std.blockish)\n(defn unrelated [] true)\n",
                encoding="utf-8",
            )
            subprocess.run(["git", "init", "-q", str(reference)], check=True)
            subprocess.run(
                ["git", "-C", str(reference), "config", "user.name", "Corpus Test"],
                check=True,
            )
            subprocess.run(
                [
                    "git",
                    "-C",
                    str(reference),
                    "config",
                    "user.email",
                    "test@example.invalid",
                ],
                check=True,
            )
            subprocess.run(["git", "-C", str(reference), "add", "-A"], check=True)
            subprocess.run(
                ["git", "-C", str(reference), "commit", "-q", "-m", "fixture"],
                check=True,
            )
            commit = subprocess.check_output(
                ["git", "-C", str(reference), "rev-parse", "HEAD"], text=True
            ).strip()
            declarations = corpus.route_declarations(
                {
                    "namespaces": [],
                    "families": [
                        {
                            "source_prefix": "src/std/block",
                            "namespace_prefix": "std.block",
                            "target_prefix": "std.block",
                            "route_kind": "preserved",
                            "default": {
                                "state": "missing",
                                "safety": "review",
                                "message": "Port block family.",
                            },
                        }
                    ],
                },
                reference,
                commit,
            )
            self.assertEqual(
                ["std.block", "std.block.base"],
                [entry["namespace"] for entry in declarations],
            )


class TargetSnapshotGenerationTest(unittest.TestCase):
    def initialise_repo(self, root, files):
        for name, content in files.items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        subprocess.run(["git", "init", "-q", str(root)], check=True)
        subprocess.run(
            ["git", "-C", str(root), "config", "user.name", "Corpus Test"],
            check=True,
        )
        subprocess.run(
            [
                "git",
                "-C",
                str(root),
                "config",
                "user.email",
                "test@example.invalid",
            ],
            check=True,
        )
        subprocess.run(["git", "-C", str(root), "add", "-A"], check=True)
        subprocess.run(
            ["git", "-C", str(root), "commit", "-q", "-m", "fixture"],
            check=True,
        )
        return subprocess.check_output(
            ["git", "-C", str(root), "rev-parse", "HEAD"], text=True
        ).strip()

    def route_fixture(self, reference_commit, target_commit, state):
        return {
            "reference": {
                "repository": "example/foundation",
                "commit": reference_commit,
            },
            "target": {
                "repository": "example/hara",
                "base_commit": target_commit,
            },
            "scope": {"id": "fixture", "description": "fixture"},
            "route_policy": {
                "states": ["implemented", "missing"],
                "safety": ["review"],
            },
            "families": [],
            "namespaces": [
                {
                    "namespace": "demo.api",
                    "source_path": "src/demo/api.clj",
                    "route_kind": "same",
                    "default": {
                        "state": state,
                        "safety": "review",
                        "message": "Port demo API.",
                        "target_namespace": "demo.api",
                        "target_path": "core/lib/src/demo/api.hal",
                    },
                }
            ],
        }

    def test_implemented_targets_are_read_from_the_pinned_commit(self):
        with tempfile.TemporaryDirectory() as reference_dir, tempfile.TemporaryDirectory() as target_dir:
            reference = Path(reference_dir)
            target = Path(target_dir)
            reference_commit = self.initialise_repo(
                reference,
                {"src/demo/api.clj": "(ns demo.api)\n(defn run [] true)\n"},
            )
            target_commit = self.initialise_repo(
                target,
                {"core/lib/src/demo/api.hal": "(ns demo.api)\n(defn run [] true)\n"},
            )
            (target / "core/lib/src/demo/api.hal").write_text(
                "(ns demo.api)\n(defn changed [] true)\n",
                encoding="utf-8",
            )
            generated = corpus.generate(
                self.route_fixture(reference_commit, target_commit, "implemented"),
                reference,
                target,
            )
            target_entry = generated["namespaces"][0]["targets"][0]
            self.assertEqual(
                subprocess.check_output(
                    [
                        "git",
                        "-C",
                        str(target),
                        "rev-parse",
                        f"{target_commit}:core/lib/src/demo/api.hal",
                    ],
                    text=True,
                ).strip(),
                target_entry["blob"],
            )

    def test_missing_targets_do_not_capture_uncommitted_blobs(self):
        with tempfile.TemporaryDirectory() as reference_dir, tempfile.TemporaryDirectory() as target_dir:
            reference = Path(reference_dir)
            target = Path(target_dir)
            reference_commit = self.initialise_repo(
                reference,
                {"src/demo/api.clj": "(ns demo.api)\n(defn run [] true)\n"},
            )
            target_commit = self.initialise_repo(
                target,
                {"core/lib/src/demo/api.hal": "(ns demo.api)\n(defn run [] true)\n"},
            )
            generated = corpus.generate(
                self.route_fixture(reference_commit, target_commit, "missing"),
                reference,
                target,
            )
            self.assertEqual(
                {
                    "namespace": "demo.api",
                    "path": "core/lib/src/demo/api.hal",
                },
                generated["namespaces"][0]["targets"][0],
            )

    def test_verify_reads_implemented_targets_from_the_pinned_commit(self):
        with tempfile.TemporaryDirectory() as reference_dir, tempfile.TemporaryDirectory() as target_dir:
            reference = Path(reference_dir)
            target = Path(target_dir)
            reference_commit = self.initialise_repo(
                reference,
                {"src/demo/api.clj": "(ns demo.api)\n(defn run [] true)\n"},
            )
            target_commit = self.initialise_repo(
                target,
                {"core/lib/src/demo/api.hal": "(ns demo.api)\n(defn run [] true)\n"},
            )
            generated = corpus.generate(
                self.route_fixture(reference_commit, target_commit, "implemented"),
                reference,
                target,
            )
            (target / "core/lib/src/demo/api.hal").write_text(
                "(ns demo.api)\n(defn changed [] true)\n",
                encoding="utf-8",
            )
            corpus.verify(generated, reference, target)


    def test_mirrors_are_read_from_the_pinned_target_commit(self):
        with tempfile.TemporaryDirectory() as reference_dir, tempfile.TemporaryDirectory() as target_dir:
            reference = Path(reference_dir)
            target = Path(target_dir)
            reference_commit = self.initialise_repo(
                reference,
                {"src/demo/api.clj": "(ns demo.api)\n(defn run [] true)\n"},
            )
            target_commit = self.initialise_repo(
                target,
                {
                    "core/lib/src/demo/api.hal": "(ns demo.api)\n(defn run [] true)\n",
                    "core/lib/rust-hal-src/demo/api.hal": "(ns demo.api)\n(defn run [] true)\n",
                },
            )
            routes = self.route_fixture(
                reference_commit,
                target_commit,
                "implemented",
            )
            routes["namespaces"][0]["mirrors"] = [
                "core/lib/rust-hal-src/demo/api.hal"
            ]
            (target / "core/lib/rust-hal-src/demo/api.hal").write_text(
                "(ns demo.api)\n(defn working-tree-drift [] true)\n",
                encoding="utf-8",
            )
            generated = corpus.generate(routes, reference, target)
            self.assertEqual(target_commit, generated["target"]["base_commit"])


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
        commit = policy["reference"]["commit"]
        available = subprocess.run(
            ["git", "-C", str(reference), "cat-file", "-e", f"{commit}^{{commit}}"],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        if available.returncode:
            self.skipTest("pinned Foundation source export has no Git history")
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
