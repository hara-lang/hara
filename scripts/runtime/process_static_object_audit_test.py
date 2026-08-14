#!/usr/bin/env python3
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import process_static_object_audit as audit


class ProcessStaticObjectAuditTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def write(self, name: str, content: str) -> Path:
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return path

    def test_accepts_canonical_process_static_object_calls(self) -> None:
        self.write(
            "runtime.hal",
            "(let [child (Process/spawn [\"true\"])] "
            "(Process/wait child))\n",
        )
        self.assertEqual([], audit.audit([self.root]))

    def test_reports_each_legacy_call_with_replacement(self) -> None:
        path = self.write(
            "runtime.hal",
            "(os/spawn [\"true\"])\n(os/process-wait child)\n",
        )
        findings = audit.audit([self.root])
        self.assertEqual(2, len(findings))
        self.assertEqual(path, findings[0].path)
        self.assertEqual(1, findings[0].line)
        self.assertEqual("Process/spawn", findings[0].replacement)
        self.assertEqual("Process/wait", findings[1].replacement)

    def test_scans_only_hal_files(self) -> None:
        self.write("notes.txt", "os/process-kill\n")
        self.write("clean.hal", "(Process/kill child)\n")
        self.assertEqual([], audit.audit([self.root]))

    def test_all_legacy_spellings_have_canonical_replacements(self) -> None:
        source = "\n".join(f"({legacy} child)" for legacy in audit.LEGACY_CALLS)
        self.write("all.hal", source)
        findings = audit.audit([self.root])
        self.assertEqual(set(audit.LEGACY_CALLS), {item.legacy for item in findings})
        self.assertEqual(
            set(audit.LEGACY_CALLS.values()),
            {item.replacement for item in findings},
        )


if __name__ == "__main__":
    unittest.main()
