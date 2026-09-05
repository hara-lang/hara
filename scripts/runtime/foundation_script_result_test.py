#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path

import foundation_script_result as result


def run(*args: str, cwd: Path) -> str:
    completed = subprocess.run(
        args,
        cwd=cwd,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    )
    return completed.stdout.strip()


def init_repo(root: Path, name: str) -> str:
    (root / "README.md").write_text(f"{name}\n", encoding="utf-8")
    run("git", "init", "-q", cwd=root)
    run("git", "config", "user.email", "test@example.com", cwd=root)
    run("git", "config", "user.name", "Test", cwd=root)
    run("git", "add", ".", cwd=root)
    run("git", "commit", "-qm", "fixture", cwd=root)
    return run("git", "rev-parse", "HEAD", cwd=root)


class ConformanceResultTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        root = Path(self.temp.name)
        self.reference = root / "foundation"
        self.target = root / "hara"
        self.reference.mkdir()
        self.target.mkdir()
        self.reference_commit = init_repo(self.reference, "foundation")
        self.target_commit = init_repo(self.target, "hara")
        self.policy = {
            "reference": {
                "repository": "example/foundation",
                "commit": self.reference_commit,
            },
            "target": {
                "repository": "example/hara",
                "base_commit": self.target_commit,
            },
            "tranches": [
                {
                    "id": "xt-lang-core-1",
                    "namespaces": ["xt.lang.spec-base", "xt.lang.common-lib"],
                }
            ],
        }
        self.inventory = {
            "reference": self.policy["reference"],
            "target": self.policy["target"],
            "inventory_sha256": "abc123",
            "summary": {
                "namespaces": 2,
                "statuses": {"ported-with-tests": 2},
            },
        }
        self.versions = {
            "java": ["openjdk version fixture"],
            "lua": ["Lua fixture"],
            "node": ["v22.fixture"],
            "python": ["Python fixture"],
            "rust": ["rustc fixture"],
        }

    def tearDown(self) -> None:
        self.temp.cleanup()

    def test_result_is_keyed_to_exact_commits_and_tranche(self) -> None:
        generated = result.build_result(
            self.policy,
            self.inventory,
            self.reference,
            self.target,
            self.versions,
        )
        self.assertEqual("passed", generated["status"])
        self.assertEqual(self.reference_commit, generated["foundation"]["commit"])
        self.assertEqual(self.target_commit, generated["hara"]["commit"])
        self.assertEqual("abc123", generated["inventory"]["sha256"])
        self.assertEqual("xt-lang-core-1", generated["tranches"][0]["id"])
        payload = dict(generated)
        digest = payload.pop("result_sha256")
        self.assertEqual(result.canonical_digest(payload), digest)

    def test_result_is_deterministic(self) -> None:
        first = result.build_result(
            self.policy,
            self.inventory,
            self.reference,
            self.target,
            self.versions,
        )
        second = result.build_result(
            self.policy,
            self.inventory,
            self.reference,
            self.target,
            self.versions,
        )
        self.assertEqual(first, second)

    def test_rejects_reference_checkout_mismatch(self) -> None:
        policy = {**self.policy, "reference": {**self.policy["reference"], "commit": "0" * 40}}
        with self.assertRaisesRegex(result.ConformanceResultError, "does not match policy"):
            result.build_result(
                policy,
                self.inventory,
                self.reference,
                self.target,
                self.versions,
            )


if __name__ == "__main__":
    unittest.main()
