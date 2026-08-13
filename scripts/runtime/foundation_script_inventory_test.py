#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path

import foundation_script_inventory as inventory


def run(*args: str, cwd: Path) -> str:
    result = subprocess.run(args, cwd=cwd, check=True, text=True, stdout=subprocess.PIPE)
    return result.stdout.strip()


def init_repo(root: Path) -> str:
    run("git", "init", "-q", cwd=root)
    run("git", "config", "user.email", "test@example.com", cwd=root)
    run("git", "config", "user.name", "Test", cwd=root)
    run("git", "add", ".", cwd=root)
    run("git", "commit", "-qm", "fixture", cwd=root)
    return run("git", "rev-parse", "HEAD", cwd=root)


class InventoryTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        root = Path(self.temp.name)
        self.reference = root / "foundation"
        self.target = root / "hara"
        self.reference.mkdir()
        self.target.mkdir()

        (self.reference / "src-lang/xt/lang").mkdir(parents=True)
        (self.reference / "test-lang/xt/lang").mkdir(parents=True)
        (self.target / "core/lib/src-lang/xt/lang").mkdir(parents=True)
        (self.target / "core/lib/test-lang/xt/lang").mkdir(parents=True)

        (self.reference / "src-lang/xt/lang/base.clj").write_text(
            """(ns xt.lang.base
  (:require [tahto.core :as l :refer [defspec.xt]]))

(l/script :xtalk)
(defspec.xt value :xt/num)
(defn.xt value [] (return 1))
""",
            encoding="utf-8",
        )
        (self.reference / "test-lang/xt/lang/base_test.clj").write_text(
            """(ns xt.lang.base-test
  (:require [tahto.core :as l]))
(l/script- :js {:runtime :basic})
(!.js (return 1))
""", encoding="utf-8"
        )
        (self.reference / "src-lang/xt/lang/child.clj").write_text(
            """(ns xt.lang.child
  (:require [tahto.core :as l]
            [xt.lang.base :as base]))

(l/script- :js {:runtime :basic})
(defptr.js legacy (+ 1 2))
(defn.js child [] (return (base/value)))
""",
            encoding="utf-8",
        )
        (self.target / "core/lib/src-lang/xt/lang/base.hal").write_text(
            """(ns xt.lang.base
  (:require [lang.core :as l :refer [defspec.xt]]))
(l/script :xtalk)
(defspec.xt value :xt/num)
(defn.xt value [] (return 1))
""",
            encoding="utf-8",
        )
        (self.target / "core/lib/test-lang/xt/lang/base_test.hal").write_text(
            """(ns xt.lang.base-test
  (:require [lang.core :as l]))
(l/script- :js {:runtime :basic})
(!.js (return 1))
""", encoding="utf-8"
        )
        (self.target / "core/lib/src-lang/xt/lang/child.hal").write_text(
            """(ns xt.lang.child
  (:require [lang.core :as l]
            [xt.lang.base :as base]))
(l/script- :js {:runtime :basic})
(defn.js child [] (return (base/value)))
""",
            encoding="utf-8",
        )
        self.reference_commit = init_repo(self.reference)
        self.target_commit = init_repo(self.target)
        self.policy = {
            "reference": {
                "repository": "example/foundation",
                "commit": self.reference_commit,
            },
            "target": {"repository": "example/hara", "base_commit": self.target_commit},
            "scope": {
                "source_root": "src-lang",
                "reference_test_root": "test-lang",
                "target_root": "core/lib/src-lang",
                "target_test_root": "core/lib/test-lang",
            },
            "macro_policy": {
                "defptr.*": {
                    "state": "obsolete",
                    "message": "Free-form pointer declarations are not part of Hara script compatibility.",
                }
            },
            "tranches": [
                {"id": "core", "namespaces": ["xt.lang.base"]}
            ],
        }

    def tearDown(self) -> None:
        self.temp.cleanup()

    def test_macro_surface_classifies_defptr_as_obsolete(self) -> None:
        surface = inventory.macro_surface(
            '(defptr.lua A 1) (defn.lua B [] (return 2)) (!.lua (B))'
        )
        self.assertEqual(["!.lua", "defn.lua"], surface["required_macros"])
        self.assertEqual(["defptr.lua"], surface["obsolete_macros"])
        self.assertEqual(["return"], surface["highlights"])

    def test_generate_orders_dependencies_and_maps_targets(self) -> None:
        generated = inventory.generate(self.policy, self.reference, self.target)
        inventory.validate(generated)
        entries = {entry["namespace"]: entry for entry in generated["namespaces"]}
        base = entries["xt.lang.base"]
        child = entries["xt.lang.child"]
        self.assertEqual(0, base["dependency_rank"])
        self.assertEqual(1, child["dependency_rank"])
        self.assertEqual("ported-with-tests", base["target"]["status"])
        self.assertEqual("ported", child["target"]["status"])
        self.assertEqual("core", base["tranche"])
        self.assertEqual([], base["target"]["missing_source_macros"])
        self.assertEqual(["!.js"], base["tests"]["reference_required_macros"])
        self.assertEqual([], base["tests"]["missing_source_macros"])
        self.assertEqual(["defptr.js"], child["obsolete_macros"])
        self.assertNotIn("defptr.js", child["required_macros"])

    def test_validation_rejects_defptr_in_required_surface(self) -> None:
        generated = inventory.generate(self.policy, self.reference, self.target)
        generated["namespaces"][0]["required_macros"].append("defptr.xt")
        generated["inventory_sha256"] = inventory.checksum(generated["namespaces"])
        with self.assertRaisesRegex(inventory.InventoryError, "defptr"):
            inventory.validate(generated)


if __name__ == "__main__":
    unittest.main()
