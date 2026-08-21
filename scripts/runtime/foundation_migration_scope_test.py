import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SHA1 = re.compile(r"^[0-9a-f]{40}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")


class FoundationMigrationScopeTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.scope = json.loads(
            (ROOT / "core/spec/foundation-migration-scope.json").read_text()
        )
        cls.parity = json.loads(
            (ROOT / "core/spec/foundation-parity.json").read_text()
        )
        cls.scripts = json.loads(
            (ROOT / "core/spec/foundation-script-policy.json").read_text()
        )

    def test_pr_heads_and_absence_of_merge_commits_are_locked(self):
        for reference in self.scope["references"].values():
            self.assertTrue(SHA1.fullmatch(reference["base_commit"]))
            self.assertTrue(SHA1.fullmatch(reference["head_commit"]))
            self.assertIsNone(reference["merge_commit"])
            self.assertFalse(reference["merged"])
            self.assertTrue(SHA256.fullmatch(reference["diff_sha256"]))
            self.assertTrue(reference["changed_files"])

    def test_parity_and_script_pins_are_explicitly_distinct(self):
        parity = self.scope["pinned_references"]["foundation_parity"]
        scripts = self.scope["pinned_references"]["foundation_scripts"]
        self.assertEqual(self.parity["reference"]["commit"], parity["commit"])
        self.assertEqual(self.scripts["reference"]["commit"], scripts["commit"])
        self.assertNotEqual(parity["commit"], scripts["commit"])
        self.assertEqual(
            ["lang", "postgres", "xt"],
            self.scope["scope"]["source_families"],
        )

    def test_parity_config_points_to_existing_ledgers(self):
        self.assertEqual(
            "core/spec/foundation-migration-scope.json",
            self.parity["scope_lock"],
        )
        self.assertEqual(
            "core/spec/code-migrate/foundation-behavioral-corpus.json",
            self.parity["behavioral_corpus"]["path"],
        )
        self.assertEqual(
            "core/spec/code-migrate/foundation-baa75a.edn",
            self.parity["migration_corpus"]["path"],
        )


if __name__ == "__main__":
    unittest.main()
