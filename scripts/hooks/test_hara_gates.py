#!/usr/bin/env python3

import json
import importlib.util
from importlib.machinery import SourceFileLoader
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import textwrap
import unittest


HERE = Path(__file__).resolve().parent
SOURCE_GATE = HERE / "hara-source-gate"
SHELL_GATE = HERE / "hara-shell-edit-gate"
SYNC_GATE = HERE / "sync-source-gates"


def load_source_gate():
    spec = importlib.util.spec_from_loader(
        "hara_source_gate", SourceFileLoader("hara_source_gate", str(SOURCE_GATE))
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class GateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.runtime = self.root / "hara"
        self.runtime.write_text(textwrap.dedent("""\
            #!/bin/sh
            mode=""
            path=""
            for arg in "$@"; do
              if [ "$mode" = run ]; then path="$arg"; mode=done; fi
              if [ "$arg" = run ]; then mode=run; fi
            done
            if [ "$mode" = done ]; then input=$(sed -n '1,999p' "$path"); else input=$(sed -n '1,999p'); fi
            case "$input" in *FAIL_NATIVE*) echo native-failure >&2; exit 7;; esac
            exit 0
        """))
        self.runtime.chmod(0o755)

    def tearDown(self):
        self.temp.cleanup()

    def source(self, payload, mode="codex"):
        env = os.environ.copy()
        env["HARA_BIN"] = str(self.runtime)
        if mode == "kimi":
            env["HARA_HOOK_MODE"] = "kimi"
        else:
            env.pop("HARA_HOOK_MODE", None)
        return subprocess.run(
            [sys.executable, str(SOURCE_GATE)],
            input=json.dumps(payload), text=True, capture_output=True, env=env,
        )

    def shell(self, command, mode="codex"):
        env = os.environ.copy()
        if mode == "kimi":
            env["HARA_HOOK_MODE"] = "kimi"
        else:
            env.pop("HARA_HOOK_MODE", None)
        return subprocess.run(
            [sys.executable, str(SHELL_GATE)],
            input=json.dumps({"tool_input": {"command": command}}),
            text=True, capture_output=True, env=env,
        )

    def payload(self, tool, tool_input, event="PreToolUse"):
        return {
            "hook_event_name": event,
            "tool_name": tool,
            "tool_input": tool_input,
            "cwd": str(self.root),
        }

    def test_write_valid_candidate_is_allowed(self):
        result = self.source(self.payload("Write", {
            "file_path": "valid.hal", "content": "(do 1)\n",
        }))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "")

    def test_runtime_failure_is_denied(self):
        result = self.source(self.payload("Write", {
            "file_path": "invalid.hal", "content": "FAIL_NATIVE\n",
        }))
        body = json.loads(result.stdout)
        self.assertEqual(body["hookSpecificOutput"]["permissionDecision"], "deny")
        self.assertIn("native-failure", body["hookSpecificOutput"]["permissionDecisionReason"])

    def test_policy_checks_complete_edit_candidate(self):
        path = self.root / "module.hal"
        path.write_text("(ns sample)\n(do 1)\n")
        result = self.source(self.payload("Edit", {
            "file_path": str(path), "old_string": "(do 1)",
            "new_string": "(clojure.bad/value)",
        }))
        self.assertIn("hara.source/no-clojure-namespace", result.stdout)

    def test_apply_patch_reconstructs_updated_file(self):
        path = self.root / "module.hal"
        path.write_text("(ns sample)\n(do 1)\n")
        patch = """*** Begin Patch
*** Update File: module.hal
@@
 (ns sample)
-(do 1)
+(do 2)
*** End Patch"""
        result = self.source(self.payload("apply_patch", {"patch": patch}))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "")

    def test_apply_patch_handles_multiple_files(self):
        first = self.root / "first.hal"
        second = self.root / "notes.txt"
        first.write_text("(do 1)\n")
        second.write_text("old\n")
        patch = """*** Begin Patch
*** Update File: first.hal
@@
-(do 1)
+(do 2)
*** Update File: notes.txt
@@
-old
+new
*** End Patch"""
        result = self.source(self.payload("apply_patch", {"patch": patch}))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "")

    def test_edit_replace_all_checks_complete_candidate(self):
        path = self.root / "module.hal"
        path.write_text("(do old old)\n")
        result = self.source(self.payload("Edit", {
            "file_path": str(path), "old_string": "old",
            "new_string": "new", "replace_all": True,
        }))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "")

    def test_apply_patch_rejects_nonmatching_hunk(self):
        (self.root / "module.hal").write_text("(do 1)\n")
        patch = """*** Begin Patch
*** Update File: module.hal
@@
-(do 9)
+(do 2)
*** End Patch"""
        result = self.source(self.payload("apply_patch", {"patch": patch}))
        self.assertIn("does not match", result.stdout)

    def test_delete_does_not_evaluate_missing_source(self):
        path = self.root / "module.hal"
        path.write_text("(do 1)\n")
        patch = "*** Begin Patch\n*** Delete File: module.hal\n*** End Patch"
        pre = self.source(self.payload("apply_patch", {"patch": patch}))
        self.assertEqual(pre.stdout, "")
        path.unlink()
        post = self.source(self.payload("apply_patch", {"patch": patch}, "PostToolUse"))
        self.assertEqual(post.stdout, "")

    def test_post_write_evaluates_actual_file(self):
        path = self.root / "module.hal"
        path.write_text("FAIL_NATIVE\n")
        result = self.source(self.payload(
            "Write", {"file_path": str(path), "content": "ignored"}, "PostToolUse"
        ))
        body = json.loads(result.stdout)
        self.assertEqual(body["decision"], "block")
        self.assertIn("native-failure", body["reason"])

    def test_tool_project_candidate_does_not_bootstrap_through_project(self):
        source = self.root / "core" / "lib" / "src" / "tool" / "project.hal"
        source.parent.mkdir(parents=True)
        source.write_text("(ns tool.project)\n")
        (self.root / "core" / "project.edn").write_text("{}\n")
        gate = load_source_gate()
        self.assertIsNone(gate.project_root(source, self.root))
        sibling = source.parent / "other.hal"
        self.assertEqual(gate.project_root(sibling, self.root), self.root / "core")

    def test_kimi_denial_uses_exit_two(self):
        result = self.source(self.payload("Write", {
            "file_path": "bad.hal", "content": "(requiring-resolve x)\n",
        }), mode="kimi")
        self.assertEqual(result.returncode, 2)
        self.assertIn("no-requiring-resolve", result.stderr)

    def test_shell_gate_catches_quoted_redirect(self):
        self.assertIn("permissionDecision", self.shell('echo x > "demo.hal"').stdout)

    def test_shell_gate_catches_inline_interpreter(self):
        result = self.shell("python3 -c 'open(\"demo.hal\",\"w\").write(\"x\")'")
        self.assertIn("permissionDecision", result.stdout)

    def test_shell_gate_allows_native_run(self):
        self.assertEqual(self.shell("hara --offline run demo.hal").stdout, "")

    def test_policy_snapshot_matches_registry(self):
        result = subprocess.run(
            [sys.executable, str(SYNC_GATE), "--check"],
            text=True, capture_output=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()
