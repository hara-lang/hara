#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("check_foundation_surface.py")
SPEC = importlib.util.spec_from_file_location("check_foundation_surface", MODULE_PATH)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class FoundationSurfaceTest(unittest.TestCase):
    def make_root(self) -> Path:
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        (root / "core/rust").mkdir(parents=True)
        (root / "core/lib/src/std/foundation").mkdir(parents=True)
        (root / "core/lib/test/std/foundation").mkdir(parents=True)
        (root / "core/spec/std").mkdir(parents=True)
        (root / "core/java/src/test").mkdir(parents=True)

        (root / "core/rust/standard-library.namespaces").write_text(
            "std.foundation\nstd.foundation.bytes\nstd.foundation.string\nstd.lib.component\n"
        )
        (root / "core/lib/src/std/foundation.hal").write_text(
            ";; std.foundation\n"
            ";; std.foundation.bytes\n"
            ";; std.foundation.string\n"
            ";; std.fs.path std.lib.format.* std.lib.component std.crypto.*\n"
            "(ns std.foundation)\n"
        )
        (root / "core/lib/src/std/foundation/bytes.hal").write_text(
            "(ns std.foundation.bytes)\n"
        )
        (root / "core/lib/src/std/foundation/string.hal").write_text(
            "(ns std.foundation.string)\n"
        )
        (root / "core/lib/test/std/foundation/bytes_test.hal").write_text(
            "(ns std.foundation.bytes-test)\n"
        )
        (root / "core/lib/test/std/foundation/string_test.hal").write_text(
            "(ns std.foundation.string-test)\n"
        )
        (root / "core/lib/test/std/foundation/alias_test.hal").write_text(
            "(ns std.foundation.alias-test)\n"
        )
        (root / "core/lib/test/std/foundation/deps_test.hal").write_text(
            "(ns std.foundation.deps-test)\n"
        )
        (root / "core/spec/std/foundation-migrations.json").write_text(
            json.dumps(
                {
                    "schemaVersion": 1,
                    "migrations": [
                        {
                            "formerName": "std.foundation.old",
                            "status": "retired",
                            "disposition": "removed",
                        }
                    ],
                }
            )
        )
        (root / "core/java/src/test/NegativeTest.java").write_text(
            'String source = "(require \'std.foundation.old)";\n'
        )
        (root / "core/spec/std/foundation-reference-classifications.json").write_text(
            json.dumps(
                {
                    "schemaVersion": 1,
                    "references": [
                        {
                            "formerName": "std.foundation.old",
                            "path": "core/java/src/test/NegativeTest.java",
                            "classification": "rejection-test",
                            "note": "Proves the retired namespace cannot load.",
                        }
                    ],
                }
            )
        )
        return root

    def test_consistent_surface_passes(self) -> None:
        report = MODULE.build_report(self.make_root())
        self.assertEqual([], report["errors"])
        self.assertEqual(
            ["std.foundation", "std.foundation.bytes", "std.foundation.string"],
            report["currentNamespaces"],
        )

    def test_missing_source_is_reported(self) -> None:
        root = self.make_root()
        (root / "core/lib/src/std/foundation/string.hal").unlink()
        report = MODULE.build_report(root)
        self.assertTrue(any("Registered/source" in error for error in report["errors"]))

    def test_orphan_test_is_reported(self) -> None:
        root = self.make_root()
        (root / "core/lib/test/std/foundation/old_test.hal").write_text(
            "(ns std.foundation.old-test)\n"
        )
        report = MODULE.build_report(root)
        self.assertTrue(any("child/test mismatch" in error for error in report["errors"]))

    def test_unclassified_reference_is_reported(self) -> None:
        root = self.make_root()
        (root / "extra.md").write_text("std.foundation.old\n")
        report = MODULE.build_report(root)
        self.assertTrue(any("Unclassified references" in error for error in report["errors"]))

    def test_stale_classification_is_reported(self) -> None:
        root = self.make_root()
        (root / "core/java/src/test/NegativeTest.java").write_text("no legacy reference\n")
        report = MODULE.build_report(root)
        self.assertTrue(any("Stale reference classifications" in error for error in report["errors"]))

    def test_bootstrap_is_development_only(self) -> None:
        root = self.make_root()
        namespaces = root / "core/rust/standard-library.namespaces"
        namespaces.write_text(namespaces.read_text() + "std.foundation.bootstrap\n")
        (root / "core/lib/src/std/foundation/bootstrap.hal").write_text(
            "(ns std.foundation.bootstrap)\n"
        )
        report = MODULE.build_report(root)
        self.assertEqual([], report["errors"])
        self.assertEqual(
            ["std.foundation.bytes", "std.foundation.string"],
            report["ordinaryChildTests"],
        )

    def test_generated_hal_src_mirror_is_not_scanned(self) -> None:
        root = self.make_root()
        mirror = root / "core/rust/hal-src/code/translate"
        mirror.mkdir(parents=True)
        (mirror / "rules.hal").write_text(";; std.foundation.old\n")
        report = MODULE.build_report(root)
        self.assertEqual([], report["errors"])


if __name__ == "__main__":
    unittest.main()
