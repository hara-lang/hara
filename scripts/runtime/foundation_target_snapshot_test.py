#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path

import foundation_target_snapshot as snapshot


def run(*args: str, cwd: Path) -> str:
    result = subprocess.run(
        args,
        cwd=cwd,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return result.stdout.strip()


class FoundationTargetSnapshotTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        files = {
            "core/lib/src-lang/xt/example.hal": "(ns xt.example)\n",
            "core/lib/test-lang/xt/example_test.hal": "(ns xt.example-test)\n",
            "core/lib/src/std/foundation.hal": "(ns std.foundation)\n",
            "docs/note.txt": "baseline\n",
        }
        for name, content in files.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        run("git", "init", "-q", cwd=self.root)
        run("git", "config", "user.email", "test@example.com", cwd=self.root)
        run("git", "config", "user.name", "Test", cwd=self.root)
        run("git", "add", ".", cwd=self.root)
        run("git", "commit", "-qm", "fixture", cwd=self.root)
        self.commit = run("git", "rev-parse", "HEAD", cwd=self.root)
        self.policy = {"target": {"base_commit": self.commit}}
        self.inventory = {
            "target": {"base_commit": self.commit},
            "namespaces": [
                {
                    "target": {
                        "path": "core/lib/src-lang/xt/example.hal",
                        "blob": "fixture",
                    },
                    "tests": {
                        "target_path": "core/lib/test-lang/xt/example_test.hal",
                        "target_blob": "fixture",
                    },
                }
            ],
        }
        self.routes = {"target": {"base_commit": self.commit}}
        self.corpus = {
            "target": {"base_commit": self.commit},
            "namespaces": [
                {
                    "targets": [
                        {
                            "path": "core/lib/src/std/foundation.hal",
                            "blob": "fixture",
                        }
                    ]
                }
            ],
        }

    def tearDown(self) -> None:
        self.temp.cleanup()

    def test_verifies_both_generated_ledgers(self) -> None:
        result = snapshot.verify(
            "all",
            self.policy,
            self.inventory,
            self.routes,
            self.corpus,
            self.root,
        )
        self.assertEqual(self.commit, result["script"]["base_commit"])
        self.assertEqual(2, result["script"]["paths"])
        self.assertEqual(1, result["corpus"]["paths"])

    def test_rejects_relevant_target_drift(self) -> None:
        path = self.root / "core/lib/src-lang/xt/example.hal"
        path.write_text("(ns xt.changed)\n", encoding="utf-8")
        with self.assertRaisesRegex(snapshot.SnapshotError, "target snapshot drift"):
            snapshot.verify_script_snapshot(self.policy, self.inventory, self.root)

    def test_ignores_changes_outside_recorded_target_paths(self) -> None:
        (self.root / "docs/note.txt").write_text("changed\n", encoding="utf-8")
        result = snapshot.verify_script_snapshot(self.policy, self.inventory, self.root)
        self.assertEqual(2, result["paths"])

    def test_rejects_disagreeing_target_metadata(self) -> None:
        inventory = {
            **self.inventory,
            "target": {"base_commit": "0" * 40},
        }
        with self.assertRaisesRegex(snapshot.SnapshotError, "metadata disagrees"):
            snapshot.verify_script_snapshot(self.policy, inventory, self.root)

    def test_rejects_escaping_evidence_path(self) -> None:
        inventory = {
            **self.inventory,
            "namespaces": [
                {
                    "target": {"path": "../outside.hal", "blob": "fixture"},
                    "tests": {},
                }
            ],
        }
        with self.assertRaisesRegex(snapshot.SnapshotError, "escapes the repository"):
            snapshot.script_paths(inventory)


if __name__ == "__main__":
    unittest.main()
