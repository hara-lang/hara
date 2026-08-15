#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import pathlib
import sys
import tempfile
import unittest

MODULE_PATH = pathlib.Path(__file__).with_name("source_only_foundation.py")
SPEC = importlib.util.spec_from_file_location("source_only_foundation", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class SourceOnlyFoundationAuditTest(unittest.TestCase):
    def fixture(self) -> tuple[tempfile.TemporaryDirectory[str], pathlib.Path]:
        temporary = tempfile.TemporaryDirectory()
        root = pathlib.Path(temporary.name)
        sources = {
            "core/rust/src/lib.rs": "include!(concat!(env!(\"OUT_DIR\"), \"/embedded_hal.rs\"));\n",
            "core/rust/Cargo.toml": "[package]\nname = \"fixture\"\n",
            "core/rust/build.rs": (
                'let canonical = manifest.join("../lib");\n'
                'let packaged = manifest.join("hal-src");\n'
            ),
            "core/java/pom.xml": (
                "<project>${project.basedir}/../lib/src "
                "${project.basedir}/../lib/src-lang</project>\n"
            ),
            "core/java/src/main/java/hara/truffle/HaraContext.java": (
                "final class HaraContext { HaraLibraryLoader libraryLoader; }\n"
            ),
            ".github/workflows/lang-runtime.yml": "name: fixture\n",
            ".github/workflows/main.yml": "name: fixture\n",
        }
        for relative, source in sources.items():
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(source, encoding="utf-8")
        return temporary, root

    def test_clean_source_layout_passes(self) -> None:
        temporary, root = self.fixture()
        with temporary:
            self.assertEqual([], MODULE.audit(root, tracked=[]))

    def test_tracked_foundation_bundle_is_rejected(self) -> None:
        temporary, root = self.fixture()
        with temporary:
            bundle = pathlib.Path("target/application/std.foundation.hbx")
            (root / bundle).parent.mkdir(parents=True, exist_ok=True)
            (root / bundle).write_bytes(b"HBX0")
            failures = MODULE.audit(root, tracked=[bundle])
            self.assertTrue(any("tracked Foundation bundle" in failure for failure in failures))

    def test_development_classpath_bundle_is_rejected(self) -> None:
        temporary, root = self.fixture()
        with temporary:
            bundle = root / "core/java/target/classes/std.foundation.hbx"
            bundle.parent.mkdir(parents=True, exist_ok=True)
            bundle.write_bytes(b"HBX0")
            failures = MODULE.audit(root, tracked=[])
            self.assertTrue(
                any("development runtime/classpath" in failure for failure in failures)
            )

    def test_untracked_target_local_application_bundle_is_allowed(self) -> None:
        temporary, root = self.fixture()
        with temporary:
            bundle = root / "target/application/std.foundation.hbx"
            bundle.parent.mkdir(parents=True, exist_ok=True)
            bundle.write_bytes(b"HBX0")
            self.assertEqual([], MODULE.audit(root, tracked=[]))

    def test_stale_runtime_bootstrap_marker_is_rejected(self) -> None:
        temporary, root = self.fixture()
        with temporary:
            path = root / "core/java/src/main/java/hara/truffle/HaraContext.java"
            path.write_text("final class HaraContext { HbxBundleLibrary bundles; }\n")
            failures = MODULE.audit(root, tracked=[])
            self.assertTrue(any("HbxBundleLibrary" in failure for failure in failures))


if __name__ == "__main__":
    unittest.main()
