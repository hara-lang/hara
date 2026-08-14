#!/usr/bin/env python3
"""Write exact metadata for a passing Foundation script conformance tranche."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Sequence

ROOT = Path(__file__).resolve().parents[2] if len(Path(__file__).resolve().parents) >= 3 else Path.cwd()
DEFAULT_POLICY = ROOT / "core/spec/foundation-script-policy.json"
DEFAULT_INVENTORY = ROOT / "core/spec/foundation-script-inventory.json"
RUNTIME_COMMANDS: dict[str, tuple[str, ...]] = {
    "java": ("java", "-version"),
    "lua": ("lua", "-v"),
    "node": ("node", "--version"),
    "python": ("python3", "--version"),
    "rust": ("rustc", "--version"),
}


class ConformanceResultError(RuntimeError):
    pass


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ConformanceResultError(f"cannot read JSON: {path}: {error}") from error


def run(command: Sequence[str], cwd: Path | None = None) -> str:
    result = subprocess.run(
        list(command),
        cwd=cwd,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode:
        rendered = " ".join(command)
        detail = result.stdout.strip()
        raise ConformanceResultError(
            f"command failed ({rendered})" + (f": {detail}" if detail else "")
        )
    return result.stdout.strip()


def git_head(root: Path) -> str:
    return run(("git", "rev-parse", "HEAD"), cwd=root)


def capture_runtime_versions() -> dict[str, list[str]]:
    versions: dict[str, list[str]] = {}
    for name, command in sorted(RUNTIME_COMMANDS.items()):
        output = run(command)
        lines = [line.strip() for line in output.splitlines() if line.strip()]
        if not lines:
            raise ConformanceResultError(f"runtime version is empty: {name}")
        versions[name] = lines
    return versions


def canonical_digest(payload: dict) -> str:
    encoded = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(encoded.encode()).hexdigest()


def build_result(
    policy: dict,
    inventory: dict,
    reference_root: Path,
    target_root: Path,
    runtime_versions: dict[str, list[str]],
) -> dict:
    expected_reference = policy.get("reference", {}).get("commit")
    reference_commit = git_head(reference_root)
    if reference_commit != expected_reference:
        raise ConformanceResultError(
            "Foundation checkout does not match policy: "
            f"expected {expected_reference}, found {reference_commit}"
        )
    if inventory.get("reference", {}).get("commit") != expected_reference:
        raise ConformanceResultError("inventory Foundation commit does not match policy")
    if inventory.get("target", {}).get("base_commit") != policy.get("target", {}).get("base_commit"):
        raise ConformanceResultError("inventory Hara base commit does not match policy")

    inventory_sha = inventory.get("inventory_sha256")
    if not inventory_sha:
        raise ConformanceResultError("inventory has no checksum")
    summary = inventory.get("summary", {})
    tranches = policy.get("tranches", [])
    if not tranches:
        raise ConformanceResultError("policy has no script tranches")

    payload = {
        "schema_version": 1,
        "status": "passed",
        "foundation": {
            "repository": policy["reference"]["repository"],
            "commit": reference_commit,
        },
        "hara": {
            "repository": policy["target"]["repository"],
            "commit": git_head(target_root),
            "inventory_base_commit": policy["target"]["base_commit"],
        },
        "inventory": {
            "sha256": inventory_sha,
            "namespaces": summary.get("namespaces"),
            "statuses": summary.get("statuses", {}),
        },
        "tranches": [
            {
                "id": tranche["id"],
                "namespaces": list(tranche["namespaces"]),
            }
            for tranche in tranches
        ],
        "runtimes": runtime_versions,
    }
    return {**payload, "result_sha256": canonical_digest(payload)}


def write(path: Path, result: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--policy", type=Path, default=DEFAULT_POLICY)
    parser.add_argument("--inventory", type=Path, default=DEFAULT_INVENTORY)
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--target-root", type=Path, default=ROOT)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        result = build_result(
            load_json(args.policy),
            load_json(args.inventory),
            args.reference,
            args.target_root,
            capture_runtime_versions(),
        )
        write(args.output, result)
        return 0
    except ConformanceResultError as error:
        print(f"foundation-script-result: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
