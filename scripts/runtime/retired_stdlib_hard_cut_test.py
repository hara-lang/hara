#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import pathlib
import sys
import unittest

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


if __name__ == "__main__":
    unittest.main()
