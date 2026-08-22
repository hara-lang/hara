#!/usr/bin/env python3
"""Validate the current std.foundation source, test, and migration-reference boundary."""
from __future__ import annotations

import argparse
import json
import re
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any, Iterable

SCHEMA_VERSION = 1
NS_PATTERN = re.compile(r"(?m)^\s*\(ns\s+([^\s\)]+)")
IGNORED_DIRECTORIES = {
    ".git",
    ".idea",
    ".vscode",
    ".worktrees",
    "node_modules",
    "target",
    "dist",
    "build",
    "site",
    "coverage",
    "__pycache__",
    # Generated cargo-publication mirror of core/lib sources (gitignored);
    # retired-name evidence is classified against the canonical sources.
    "hal-src",
}
TEXT_SUFFIXES = {
    ".clj",
    ".edn",
    ".hal",
    ".java",
    ".js",
    ".json",
    ".jsonc",
    ".md",
    ".mjs",
    ".py",
    ".rs",
    ".sh",
    ".toml",
    ".ts",
    ".tsx",
    ".txt",
    ".yaml",
    ".yml",
}
ROOT_TEST_FIXTURES = {
    "alias_test.hal": "std.foundation.alias-test",
    "deps_test.hal": "std.foundation.deps-test",
}
# Development-only namespaces are registered in the inventory but are not
# ordinary production children: no child test contract and no root-header
# mention. Mirrors scripts/generate_foundation_api_manifest.py, which cites
# core/rust/bootstrap.namespaces (exactly six production std.foundation
# namespaces).
DEVELOPMENT_ONLY_NAMESPACES = ("std.foundation.bootstrap",)
REQUIRED_HEADER_DIRECTIONS = (
    "std.fs.path",
    "std.lib.format.*",
    "std.lib.component",
    "std.crypto.*",
)
FORBIDDEN_HEADER_TEXT = (
    "std.lib.foundation",
    "std.foundation/* libraries",
    "str/file/context/etc.",
)


class SurfaceError(ValueError):
    """Raised when the Foundation surface contract is inconsistent."""


def read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise SurfaceError(f"Expected a JSON object: {path}")
    return value


def read_inventory(path: Path) -> list[str]:
    return sorted(
        {
            line.strip()
            for line in path.read_text().splitlines()
            if line.strip() and not line.lstrip().startswith("#")
        }
    )


def namespace_from_source(path: Path) -> str:
    match = NS_PATTERN.search(path.read_text())
    if not match:
        raise SurfaceError(f"No namespace declaration in {path}")
    return match.group(1)


def current_foundation_namespaces(inventory: Iterable[str]) -> list[str]:
    return sorted(
        name
        for name in inventory
        if name == "std.foundation" or name.startswith("std.foundation.")
    )


def source_foundation_namespaces(root: Path) -> list[str]:
    source_root = root / "core/lib/src/std"
    root_source = source_root / "foundation.hal"
    child_root = source_root / "foundation"
    paths = [root_source, *sorted(child_root.glob("*.hal"))]
    missing = [path for path in paths if not path.is_file()]
    if missing:
        raise SurfaceError("Missing Foundation source: " + ", ".join(map(str, missing)))
    return sorted(namespace_from_source(path) for path in paths)


def foundation_test_contract(root: Path) -> tuple[list[str], list[str]]:
    test_root = root / "core/lib/test/std/foundation"
    if not test_root.is_dir():
        raise SurfaceError(f"Missing Foundation test directory: {test_root}")

    child_targets: list[str] = []
    root_fixtures: list[str] = []
    for path in sorted(test_root.glob("*_test.hal")):
        namespace = namespace_from_source(path)
        expected_fixture = ROOT_TEST_FIXTURES.get(path.name)
        if expected_fixture:
            if namespace != expected_fixture:
                raise SurfaceError(
                    f"Foundation root fixture {path} declares {namespace}; expected {expected_fixture}"
                )
            root_fixtures.append(namespace)
            continue
        if not namespace.startswith("std.foundation.") or not namespace.endswith("-test"):
            raise SurfaceError(f"Ordinary Foundation test has invalid namespace: {path}: {namespace}")
        child_targets.append(namespace.removesuffix("-test"))
    return sorted(child_targets), sorted(root_fixtures)


def migration_names(ledger: dict[str, Any]) -> list[str]:
    migrations = ledger.get("migrations")
    if not isinstance(migrations, list):
        raise SurfaceError("Foundation migration ledger requires a migrations array")
    names: list[str] = []
    for row in migrations:
        former = row.get("formerName") if isinstance(row, dict) else None
        if not isinstance(former, str) or not former.startswith("std.foundation."):
            raise SurfaceError(f"Invalid Foundation migration row: {row!r}")
        names.append(former)
    if len(names) != len(set(names)):
        raise SurfaceError("Foundation migration ledger contains duplicate former names")
    return sorted(names)


def load_classifications(path: Path, former_names: set[str]) -> dict[str, dict[str, dict[str, str]]]:
    document = read_json(path)
    if document.get("schemaVersion") != SCHEMA_VERSION:
        raise SurfaceError(
            f"Unsupported Foundation reference classification schema: {document.get('schemaVersion')!r}"
        )
    rows = document.get("references")
    if not isinstance(rows, list):
        raise SurfaceError("Foundation reference classifications require a references array")

    result: dict[str, dict[str, dict[str, str]]] = defaultdict(dict)
    for row in rows:
        if not isinstance(row, dict):
            raise SurfaceError(f"Invalid Foundation reference classification: {row!r}")
        former = row.get("formerName")
        relative = row.get("path")
        classification = row.get("classification")
        note = row.get("note")
        if former not in former_names:
            raise SurfaceError(f"Classification names a non-ledger Foundation surface: {former!r}")
        if not isinstance(relative, str) or not relative or relative.startswith("/") or ".." in Path(relative).parts:
            raise SurfaceError(f"Invalid classification path for {former}: {relative!r}")
        if not isinstance(classification, str) or not classification:
            raise SurfaceError(f"Missing classification for {former} in {relative}")
        if not isinstance(note, str) or not note:
            raise SurfaceError(f"Missing classification note for {former} in {relative}")
        if relative in result[former]:
            raise SurfaceError(f"Duplicate classification for {former} in {relative}")
        result[former][relative] = {
            "classification": classification,
            "note": note,
        }
    return result


def text_files(root: Path) -> Iterable[Path]:
    for path in root.rglob("*"):
        if not path.is_file() or path.suffix.lower() not in TEXT_SUFFIXES:
            continue
        relative = path.relative_to(root)
        if any(part in IGNORED_DIRECTORIES for part in relative.parts):
            continue
        yield path


def retired_reference_paths(
    root: Path,
    former_names: Iterable[str],
    *,
    automatic_paths: set[str],
) -> dict[str, set[str]]:
    references: dict[str, set[str]] = {name: set() for name in former_names}
    names = tuple(former_names)
    for path in text_files(root):
        relative = path.relative_to(root).as_posix()
        if relative in automatic_paths:
            continue
        try:
            source = path.read_text()
        except UnicodeDecodeError:
            continue
        for former in names:
            if former in source:
                references[former].add(relative)
    return references


def header_errors(root: Path, current: list[str]) -> list[str]:
    path = root / "core/lib/src/std/foundation.hal"
    source = path.read_text()
    marker = "(ns std.foundation)"
    if marker not in source:
        return [f"Root Foundation source does not declare {marker}: {path}"]
    header = source.split(marker, 1)[0]
    errors = [
        f"Root Foundation header retains stale text {text!r}"
        for text in FORBIDDEN_HEADER_TEXT
        if text in header
    ]
    for namespace in current:
        if namespace != "std.foundation" and namespace not in header:
            errors.append(f"Root Foundation header does not name current child {namespace}")
    for direction in REQUIRED_HEADER_DIRECTIONS:
        if direction not in header:
            errors.append(f"Root Foundation header does not identify current owner {direction}")
    return errors


def build_report(root: Path) -> dict[str, Any]:
    inventory_path = root / "core/rust/standard-library.namespaces"
    ledger_path = root / "core/spec/std/foundation-migrations.json"
    classifications_path = root / "core/spec/std/foundation-reference-classifications.json"

    current = current_foundation_namespaces(read_inventory(inventory_path))
    source = source_foundation_namespaces(root)
    child_tests, root_fixtures = foundation_test_contract(root)
    ledger = read_json(ledger_path)
    former = migration_names(ledger)
    classifications = load_classifications(classifications_path, set(former))
    automatic_paths = {
        ledger_path.relative_to(root).as_posix(),
        classifications_path.relative_to(root).as_posix(),
    }
    references = retired_reference_paths(root, former, automatic_paths=automatic_paths)

    errors: list[str] = []
    if current != source:
        errors.append(f"Registered/source Foundation mismatch: registered={current} source={source}")

    production = [name for name in current if name not in DEVELOPMENT_ONLY_NAMESPACES]
    current_children = sorted(name for name in production if name != "std.foundation")
    if child_tests != current_children:
        errors.append(
            f"Current child/test mismatch: children={current_children} ordinary-tests={child_tests}"
        )

    current_set = set(current)
    retired_current = sorted(current_set.intersection(former))
    if retired_current:
        errors.append("Migration names remain current: " + ", ".join(retired_current))

    errors.extend(header_errors(root, production))

    reference_rows: dict[str, list[dict[str, str]]] = {}
    for name in former:
        actual = references[name]
        expected = set(classifications.get(name, {}))
        unclassified = sorted(actual - expected)
        stale = sorted(expected - actual)
        if unclassified:
            errors.append(f"Unclassified references for {name}: {', '.join(unclassified)}")
        if stale:
            errors.append(f"Stale reference classifications for {name}: {', '.join(stale)}")
        reference_rows[name] = [
            {
                "path": path,
                **classifications[name][path],
            }
            for path in sorted(actual.intersection(expected))
        ]

    return {
        "schemaVersion": SCHEMA_VERSION,
        "currentNamespaces": current,
        "sourceNamespaces": source,
        "ordinaryChildTests": child_tests,
        "rootTestFixtures": root_fixtures,
        "migrationReferences": reference_rows,
        "errors": errors,
    }


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser()
    result.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    result.add_argument("--report", type=Path)
    return result


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    root = args.root.resolve()
    try:
        report = build_report(root)
    except (OSError, json.JSONDecodeError, SurfaceError) as error:
        print(f"Foundation surface conformance failed: {error}", file=sys.stderr)
        return 1

    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")

    if report["errors"]:
        print("Foundation surface conformance failed:", file=sys.stderr)
        for error in report["errors"]:
            print(f"- {error}", file=sys.stderr)
        return 1

    print(
        "Foundation surface conformance passed: "
        f"{len(report['currentNamespaces'])} current namespaces, "
        f"{len(report['ordinaryChildTests'])} child tests, "
        f"{sum(len(rows) for rows in report['migrationReferences'].values())} classified references"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
