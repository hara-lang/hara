#!/usr/bin/env python3
import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "foundation_target_snapshot",
    ROOT / "scripts/runtime/foundation_target_snapshot.py",
)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


class FoundationTargetSnapshotTest(unittest.TestCase):
    def test_script_inventory_paths_are_non_empty_and_contained(self):
        inventory = {"namespaces": [{"target": {
            "blob": "a" * 40,
            "path": "core/lib/src/demo.hal",
        }}]}
        self.assertEqual(
            MODULE.script_paths(inventory),
            ["core/lib/src/demo.hal"],
        )

    def test_evidence_path_rejects_parent_escape(self):
        with self.assertRaises(MODULE.SnapshotError):
            MODULE.evidence_path("../outside")


if __name__ == "__main__":
    unittest.main()
