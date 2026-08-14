#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import importlib.util
import pathlib
import sys
import tempfile
import unittest

from retired_stdlib_hbx import HbxFormatError, retired_module_references

MODULE_PATH = pathlib.Path(__file__).with_name("retired_stdlib_hard_cut.py")
SPEC = importlib.util.spec_from_file_location("retired_stdlib_hard_cut", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class HalMigrationTest(unittest.TestCase):
    def migrate(self, source: str, path: str = "core/lib/test/example_test.hal") -> str:
        return MODULE.migrate_hal(pathlib.Path(path), source)

    def test_migrates_aliases_expected_errors_and_direct_results(self) -> None:
        source = """(ns example-test
  (:require [std.lib.test :as test]
            [std.lib.walk :as walk]
            [std.work :as work]))

(def results
  [(test/check "walk" (walk/prewalk identity '(1 2)) '(1 2))
   (test/check-error "error" (throw "boom"))])

(test/print-results results)
"""
        migrated = self.migrate(source)
        self.assertNotIn("std.lib.test", migrated)
        self.assertNotIn("std.lib.walk", migrated)
        self.assertIn("std.foundation/prewalk", migrated)
        self.assertIn("(try (do (throw \"boom\") false)", migrated)
        self.assertTrue(migrated.rstrip().endswith("results"))

    def test_inline_result_vectors_are_rewritten_before_wrapper_removal(self) -> None:
        source = """(ns example-test (:require [std.lib.test :as t]))
(t/print-results [(t/check "direct" true true)])
"""
        migrated = self.migrate(source)
        self.assertIn("[(test-check \"direct\" true true)]", migrated)
        self.assertNotIn("print-results", migrated)

    def test_focused_fixture_preserves_quoted_legacy_input(self) -> None:
        source = """(ns code.translate-rules-test
  (:require [std.lib.test :as test]))
(def legacy '(ns old (:require [std.lib.test :as test])))
(def results [(test/check "fixture" true true)])
(test/print-results results)
"""
        migrated = self.migrate(source, "core/lib/test/code/translate_rules_test.hal")
        self.assertIn("[std.lib.test :as test]", migrated)
        self.assertIn("test-check", migrated)
        self.assertTrue(migrated.rstrip().endswith("results"))

    def test_unknown_retired_member_aborts(self) -> None:
        source = """(ns example-test (:require [std.lib.walk :as walk]))
(walk/not-a-real-operation 1)
"""
        with self.assertRaises(MODULE.MigrationError):
            self.migrate(source)

    def test_removes_all_runner_exclusions(self) -> None:
        source = """collect_root() {
    find "$1" -name '*.hal' \\
            ! -path 'core/lib/test/std/lib/test.hal' \\
            -print
}
collect_files() {
    find "$1" -name '*.hal' \\
                ! -path 'core/lib/test/std/lib/test.hal' \\
                -print
    elif [ -f "$target" ]; then
        if [ "$target" != 'core/lib/test/std/lib/test.hal' ]; then
            printf '%s\\n' "$target"
        fi
    fi
}
"""
        migrated = MODULE.update_run_lib_tests(source)
        self.assertNotIn("std/lib/test.hal", migrated)
        self.assertEqual(migrated.count("-print"), 2)
        self.assertIn("printf '%s\\n' \"$target\"", migrated)

    def test_historical_foundation_inventory_is_allowlisted(self) -> None:
        self.assertIn(
            "core/spec/foundation-script-inventory.json", MODULE.LEGACY_ALLOWLIST
        )


class HbxModuleTableTest(unittest.TestCase):
    @staticmethod
    def bundle(
        modules: list[tuple[str, str, tuple[str, ...], bytes, bool]]
    ) -> bytes:
        def blob(value: bytes) -> bytes:
            return len(value).to_bytes(4, "little") + value

        payload = bytearray(len(modules).to_bytes(4, "little"))
        for resource, namespace_form, dependencies, artifact, eager in modules:
            payload.extend(blob(resource.encode()))
            payload.extend(blob(namespace_form.encode()))
            payload.extend(bytes(32))
            payload.extend(len(dependencies).to_bytes(4, "little"))
            for dependency in dependencies:
                payload.extend(blob(dependency.encode()))
            payload.append(int(eager))
            payload.extend(blob(artifact))
        encoded_payload = bytes(payload)
        return b"HBX0" + hashlib.sha256(encoded_payload).digest() + encoded_payload

    def references(self, data: bytes) -> list[str]:
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "std.foundation.hbx"
            path.write_bytes(data)
            return retired_module_references(path, MODULE.RETIRED)

    def test_legacy_spelling_inside_bytecode_is_not_a_module_reference(self) -> None:
        data = self.bundle(
            [
                (
                    "code.translate.rules",
                    "(ns code.translate.rules)",
                    (),
                    b"migration literal: std.lib.walk and std.lib.test",
                    False,
                )
            ]
        )
        self.assertEqual([], self.references(data))

    def test_retired_resources_dependencies_and_namespace_forms_are_rejected(self) -> None:
        data = self.bundle(
            [
                (
                    "std.lib.walk",
                    "(ns std.lib.walk)",
                    (),
                    b"",
                    False,
                ),
                (
                    "example.dependency",
                    "(ns example.dependency (:require [std.lib.test :as test]))",
                    ("std.lib.test",),
                    b"",
                    False,
                ),
                (
                    "example.namespace-form",
                    "(ns example.namespace-form (:require [std.lib.walk :as walk]))",
                    (),
                    b"",
                    False,
                ),
            ]
        )
        self.assertEqual(
            [
                "example.dependency dependency std.lib.test",
                "example.namespace-form namespace form std.lib.walk",
                "resource std.lib.walk",
            ],
            self.references(data),
        )

    def test_corrupt_hbx_checksum_is_rejected(self) -> None:
        data = bytearray(
            self.bundle([("std.foundation", "(ns std.foundation)", (), b"", True)])
        )
        data[-1] ^= 1
        with self.assertRaises(HbxFormatError):
            self.references(bytes(data))


if __name__ == "__main__":
    unittest.main()
