#!/usr/bin/env python3
"""Resolve PR #589 against current main, preserving the retired-stdlib hard cut."""

from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

KNOWN_CONFLICTS = {
    "core/lib/test/std/block_navigation_test.hal",
    "core/lib/test/std/block_test.hal",
    "core/lib/test/std/work_command_test.hal",
    "core/lib/test/std/work_receipt_test.hal",
    "core/lib/test/std/work_report_renderer_test.hal",
    "core/rust/assets/std.foundation.hbx",
    "core/rust/src/bin/hara/cli/project.rs",
    "core/rust/standard-library.namespaces",
}

MAIN_OWNED_TESTS = [
    "core/lib/test/std/block_navigation_test.hal",
    "core/lib/test/std/block_test.hal",
    "core/lib/test/std/work_command_test.hal",
    "core/lib/test/std/work_receipt_test.hal",
    "core/lib/test/std/work_report_renderer_test.hal",
]


def run(*args: str, check: bool = True, capture: bool = False) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=ROOT,
        check=check,
        text=True,
        capture_output=capture,
    )


def output(*args: str) -> str:
    return run(*args, capture=True).stdout.strip()


def conflicted(path: str) -> bool:
    return bool(output("git", "ls-files", "-u", "--", path))


def replace_exact(path: Path, old: str, new: str, label: str) -> None:
    source = path.read_text()
    if old in source:
        if source.count(old) != 1:
            raise SystemExit(f"{label} is ambiguous")
        source = source.replace(old, new)
    elif new not in source:
        raise SystemExit(f"{label} is missing")
    path.write_text(source)


def main() -> None:
    run("git", "config", "user.name", "github-actions[bot]")
    run(
        "git",
        "config",
        "user.email",
        "41898282+github-actions[bot]@users.noreply.github.com",
    )
    run("git", "fetch", "origin", "main")
    merged = run("git", "merge", "--no-commit", "--no-ff", "origin/main", check=False)

    conflicts = {
        line
        for line in output("git", "diff", "--name-only", "--diff-filter=U").splitlines()
        if line
    }
    unexpected = sorted(conflicts - KNOWN_CONFLICTS)
    if unexpected:
        raise SystemExit(f"unexpected merge conflicts: {unexpected}")
    if merged.returncode != 0 and not conflicts:
        raise SystemExit("merge failed without producing resolvable index conflicts")

    for path in MAIN_OWNED_TESTS:
        if conflicted(path):
            run("git", "checkout", "--theirs", "--", path)
            run("git", "add", path)

    project_path = "core/rust/src/bin/hara/cli/project.rs"
    if conflicted(project_path):
        run("git", "checkout", "--ours", "--", project_path)
        run("git", "add", project_path)

    inventory_path = "core/rust/standard-library.namespaces"
    if conflicted(inventory_path):
        run("git", "checkout", "--theirs", "--", inventory_path)
    inventory = ROOT / inventory_path
    retired = {"std.lib.test", "std.lib.walk"}
    lines = [line for line in inventory.read_text().splitlines() if line not in retired]
    inventory.write_text("\n".join(lines) + "\n")
    required = {"code.vm.conformance", "std.substrate.protocol"}
    missing = required - set(lines)
    if missing:
        raise SystemExit(f"missing current-main namespaces: {sorted(missing)}")
    forbidden = retired | {
        "std.lib.substrate",
        "std.lib.substrate.frame",
        "std.lib.substrate.protocol",
        "std.context",
    }
    present = forbidden & set(lines)
    if present:
        raise SystemExit(f"forbidden namespaces remain: {sorted(present)}")
    run("git", "add", inventory_path)

    replace_exact(
        ROOT / "core/lib/test/std/foundation_test_primitives_test.hal",
        "[true :passed false 1 2])",
        "[true nil false 1 2])",
        "portable Test/result status expectation",
    )

    replace_exact(
        ROOT / "core/rust/src/project/production.rs",
        "        namespace_edges: BTreeSet::new(),\n        native_primitives: BTreeSet::new(),",
        "        namespace_edges: BTreeSet::new(),\n        native_roots: Default::default(),\n        native_primitives: BTreeSet::new(),",
        "failed-expansion UnitAnalysis initializer",
    )

    java_path = ROOT / "core/java/src/test/java/hara/truffle/HaraNativeTestRunnerTest.java"
    java_source = java_path.read_text()
    if "import java.nio.file.Files;" not in java_source:
        marker = "import java.nio.file.Path;"
        if java_source.count(marker) != 1:
            raise SystemExit("Java Path import is missing or ambiguous")
        java_source = java_source.replace(
            marker,
            "import java.nio.file.Files;\nimport java.nio.file.Path;",
        )
    method_name = "classifiesDirectFoundationResultVectors"
    if method_name not in java_source:
        method = r'''

  @Test
  public void classifiesDirectFoundationResultVectors() throws Exception {
    Path file = Files.createTempFile("hara-direct-test-result-", ".hal");
    try {
      Files.writeString(file, "[(test-check \"direct result\" true true)]");
      HaraNativeTestRunner.Result direct = HaraNativeTestRunner.runFile(ROOT, file);
      assertTrue(direct.passed());
      assertEquals(1, direct.facts());
      assertEquals(1, direct.checks());
      assertEquals(1, direct.passedChecks());
      assertEquals(0, direct.failedChecks());

      HaraNativeTestRunner.Result encoded =
          HaraNativeTestRunner.parseResult(
              file,
              "\"[{:name \\\"encoded\\\" :actual true :expected true :pass true}]\"");
      assertTrue(encoded.passed());
      assertEquals(1, encoded.facts());
      assertEquals(1, encoded.passedChecks());
      assertEquals(0, encoded.failedChecks());
    } finally {
      Files.deleteIfExists(file);
    }
  }
'''
        closing = java_source.rfind("\n}")
        if closing < 0:
            raise SystemExit("Java test class closing brace was not found")
        java_source = java_source[:closing] + method + java_source[closing:]
    java_path.write_text(java_source)

    hbx_path = "core/rust/assets/std.foundation.hbx"
    if conflicted(hbx_path):
        run("git", "checkout", "--theirs", "--", hbx_path)
        run("git", "add", hbx_path)

    unresolved = output("git", "diff", "--name-only", "--diff-filter=U")
    if unresolved:
        raise SystemExit(f"unresolved conflicts remain:\n{unresolved}")

    for path in MAIN_OWNED_TESTS:
        run("git", "diff", "--exit-code", "origin/main", "--", path)

    for temporary in [
        ROOT / ".github/workflows/_resolve-pr-589.yml",
        ROOT / ".github/workflows/_resolve-pr-589-v2.yml",
        ROOT / "scripts/runtime/resolve_pr_589.py",
    ]:
        temporary.unlink(missing_ok=True)

    run("git", "add", "core/lib/test/std/foundation_test_primitives_test.hal")
    run("git", "add", "core/rust/src/project/production.rs")
    run("git", "add", "core/java/src/test/java/hara/truffle/HaraNativeTestRunnerTest.java")
    run("git", "add", "-A", ".github/workflows", "scripts/runtime/resolve_pr_589.py")

    print(f"resolved conflicts: {sorted(conflicts)}")


if __name__ == "__main__":
    main()
