#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXAMPLES = ROOT / "core" / "lib" / "examples"
CATALOG = EXAMPLES / "catalog.json"
EXPECTED_REGISTRY = "64d81ebe5fded2809c6fc4414796a3feddf98a33"


def fail(message: str) -> None:
    raise SystemExit(f"examples/catalog: {message}")


def main() -> int:
    document = json.loads(CATALOG.read_text())
    if document.get("schemaVersion") != 1:
        fail("schemaVersion must be 1")
    authority = document.get("authority") or {}
    if authority.get("repository") != "hara-lang/hara-specs-registry":
        fail("unexpected specification authority")
    if authority.get("commit") != EXPECTED_REGISTRY:
        fail("unexpected specification revision")

    entries = document.get("entries")
    if not isinstance(entries, list) or not entries:
        fail("entries must be a non-empty list")

    actual = sorted(
        path.name for path in EXAMPLES.iterdir()
        if path.name != CATALOG.name
    )
    declared = sorted(entry.get("path") for entry in entries if isinstance(entry, dict))
    if actual != declared:
        fail(f"top-level inventory mismatch: actual={actual!r} declared={declared!r}")

    seen = set()
    allowed_modes = {"native-smoke", "deferred", "inventory-only"}
    for entry in entries:
        path = entry.get("path")
        if not isinstance(path, str) or not path or "/" in path or path in {".", ".."}:
            fail(f"unsafe or invalid entry path: {path!r}")
        if path in seen:
            fail(f"duplicate entry: {path}")
        seen.add(path)
        target = EXAMPLES / path
        if not target.exists():
            fail(f"missing example path: {path}")
        if not entry.get("kind") or not entry.get("status"):
            fail(f"{path}: kind and status are required")
        specs = entry.get("governingSpecs")
        capabilities = entry.get("capabilities")
        if not isinstance(specs, list) or not all(isinstance(item, str) for item in specs):
            fail(f"{path}: governingSpecs must be a string list")
        if not isinstance(capabilities, list) or not all(isinstance(item, str) for item in capabilities):
            fail(f"{path}: capabilities must be a string list")
        validation = entry.get("validation") or {}
        mode = validation.get("mode")
        if mode not in allowed_modes:
            fail(f"{path}: unsupported validation mode {mode!r}")
        if mode == "native-smoke" and not isinstance(validation.get("expectedStdout"), str):
            fail(f"{path}: native-smoke requires expectedStdout")
        if mode in {"deferred", "inventory-only"} and not validation.get("reason"):
            fail(f"{path}: {mode} requires a reason")

    print(
        f"validated {len(entries)} top-level example entries against "
        f"hara-specs-registry@{EXPECTED_REGISTRY}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
