#!/usr/bin/env python3
"""Verify that generated Foundation evidence names an exact Hara target snapshot."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Sequence

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_POLICY = ROOT / "core/spec/foundation-script-policy.json"
DEFAULT_INVENTORY = ROOT / "core/spec/foundation-script-inventory.json"
SHA = re.compile(r"^[0-9a-f]{40}$")


class SnapshotError(RuntimeError):
    pass


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise SnapshotError(f"cannot read JSON: {path}: {error}") from error


def git(root: Path, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        ["git", "-C", str(root), *args],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if check and result.returncode:
        detail = result.stderr.strip() or result.stdout.strip() or "git command failed"
        raise SnapshotError(detail)
    return result


def require_commit(root: Path, commit: str, label: str) -> None:
    if not SHA.fullmatch(commit):
        raise SnapshotError(f"{label} target commit is not pinned: {commit!r}")
    resolved = git(root, "rev-parse", f"{commit}^{{commit}}").stdout.strip()
    if resolved != commit:
        raise SnapshotError(
            f"{label} target commit resolves unexpectedly: expected {commit}, found {resolved}"
        )


def evidence_path(value: object) -> str | None:
    if not isinstance(value, str) or not value:
        return None
    path = Path(value)
    if path.is_absolute() or ".." in path.parts:
        raise SnapshotError(f"target evidence path escapes the repository: {value}")
    return path.as_posix()


def script_paths(inventory: dict) -> list[str]:
    paths: set[str] = set()
    for entry in inventory.get("namespaces", []):
        target = entry.get("target", {})
        if target.get("blob"):
            path = evidence_path(target.get("path"))
            if path:
                paths.add(path)
        tests = entry.get("tests", {})
        if tests.get("target_blob"):
            path = evidence_path(tests.get("target_path"))
            if path:
                paths.add(path)
    if not paths:
        raise SnapshotError("Foundation script inventory has no target blob paths")
    return sorted(paths)


def changed_paths(root: Path, commit: str, paths: Sequence[str]) -> list[str]:
    result = git(root, "diff", "--quiet", commit, "--", *paths, check=False)
    if result.returncode == 0:
        return []
    if result.returncode != 1:
        detail = result.stderr.strip() or result.stdout.strip() or "git diff failed"
        raise SnapshotError(detail)
    output = git(root, "diff", "--name-only", commit, "--", *paths).stdout
    return [line for line in output.splitlines() if line]


def verify_ledger(
    root: Path,
    label: str,
    recorded_commit: str,
    generated_commit: str,
    paths: Sequence[str],
) -> dict:
    if generated_commit != recorded_commit:
        raise SnapshotError(
            f"{label} target metadata disagrees: policy records {recorded_commit}, "
            f"generated evidence records {generated_commit}"
        )
    require_commit(root, recorded_commit, label)
    changed = changed_paths(root, recorded_commit, paths)
    if changed:
        rendered = ", ".join(changed[:12])
        suffix = "" if len(changed) <= 12 else f" (+{len(changed) - 12} more)"
        raise SnapshotError(
            f"{label} target snapshot drift from {recorded_commit}: {rendered}{suffix}"
        )
    return {"base_commit": recorded_commit, "paths": len(paths)}


def verify_script_snapshot(
    policy: dict,
    inventory: dict,
    target_root: Path,
) -> dict:
    return verify_ledger(
        target_root,
        "Foundation script inventory",
        policy.get("target", {}).get("base_commit", ""),
        inventory.get("target", {}).get("base_commit", ""),
        script_paths(inventory),
    )


def verify(
    ledger: str,
    policy: dict,
    inventory: dict,
    target_root: Path,
) -> dict:
    result: dict[str, dict] = {}
    if ledger in {"all", "script"}:
        result["script"] = verify_script_snapshot(policy, inventory, target_root)
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ledger", choices=("all", "script"), default="all")
    parser.add_argument("--policy", type=Path, default=DEFAULT_POLICY)
    parser.add_argument("--inventory", type=Path, default=DEFAULT_INVENTORY)
    parser.add_argument("--target-root", type=Path, default=ROOT)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    try:
        result = verify(
            args.ledger,
            load_json(args.policy),
            load_json(args.inventory),
            args.target_root,
        )
    except SnapshotError as error:
        print(f"foundation-target-snapshot: {error}", file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        for name, metadata in sorted(result.items()):
            print(
                f"Foundation target snapshot ({name}): "
                f"{metadata['paths']} paths @ {metadata['base_commit']}"
            )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
