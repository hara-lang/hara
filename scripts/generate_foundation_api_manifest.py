#!/usr/bin/env python3
"""Build the canonical Foundation API manifest from Hara's registered inventory."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

SCHEMA_VERSION = 2
DEFAULT_EXTRA_NAMESPACES = ("std.lib.collection",)
ALLOWED_MIGRATION_STATUSES = {
    "moved",
    "retired",
    "compatibility-only",
    "planned-replacement",
}


def canonical_json(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()


def digest(value: Any) -> str:
    return f"sha256:{hashlib.sha256(canonical_json(value)).hexdigest()}"


def read_inventory(path: Path) -> list[str]:
    names = {
        line.strip()
        for line in path.read_text().splitlines()
        if line.strip() and not line.lstrip().startswith("#")
    }
    return sorted(names)


def selected_namespaces(inventory: list[str], extras: tuple[str, ...]) -> list[str]:
    registered = set(inventory)
    missing_extras = sorted(set(extras) - registered)
    if missing_extras:
        raise ValueError(f"Configured API namespaces are not registered: {', '.join(missing_extras)}")
    return sorted(
        name
        for name in registered
        if name == "std.foundation" or name.startswith("std.foundation.") or name in extras
    )


def parse_runtime_config(path: Path) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    source = path.read_text()
    libraries_match = re.search(r"const LIBRARIES:.*?=\s*&\[(.*?)\];", source, re.S)
    native_match = re.search(r"const NATIVE_TYPES:.*?=\s*&\[(.*?)\];", source, re.S)
    if not libraries_match or not native_match:
        raise ValueError(f"Unable to parse runtime aliases from {path}")
    aliases = [
        {
            "alias": alias,
            "target": namespace,
            "kind": "namespace-alias",
            "automatic": True,
        }
        for _, namespace, alias in re.findall(
            r'\("([^"]+)",\s*"([^"]+)",\s*"([^"]+)"\)', libraries_match.group(1)
        )
        if namespace.startswith("std.foundation.")
    ]
    native_objects = [
        {
            "name": name,
            "namespace": f"std.native.{name}",
            "automaticAlias": name,
            "kind": "static-object",
        }
        for name in re.findall(r'"([^"]+)"', native_match.group(1))
    ]
    return sorted(aliases, key=lambda item: item["alias"]), sorted(
        native_objects, key=lambda item: item["name"]
    )


def load_migrations(path: Path | None) -> tuple[int, list[dict[str, Any]], str | None]:
    if path is None:
        return 0, [], None
    document = json.loads(path.read_text())
    schema_version = document.get("schemaVersion")
    migrations = document.get("migrations")
    if not isinstance(schema_version, int) or not isinstance(migrations, list):
        raise ValueError("Foundation migration ledger requires schemaVersion and migrations")
    seen: set[str] = set()
    for migration in migrations:
        former = migration.get("formerName")
        status = migration.get("status")
        if not isinstance(former, str) or not former.startswith("std.foundation."):
            raise ValueError(f"Invalid Foundation migration name: {former!r}")
        if status not in ALLOWED_MIGRATION_STATUSES:
            raise ValueError(f"Invalid migration status for {former}: {status!r}")
        if former in seen:
            raise ValueError(f"Duplicate Foundation migration: {former}")
        if not migration.get("replacement") and not migration.get("disposition"):
            raise ValueError(f"Migration requires replacement or disposition: {former}")
        for field in ("requireRewrite", "callRewrite", "evidence"):
            if not migration.get(field):
                raise ValueError(f"Migration requires {field}: {former}")
        seen.add(former)
    return schema_version, sorted(migrations, key=lambda item: item["formerName"]), str(path)


def raw_api(args: argparse.Namespace, root: Path) -> dict[str, Any]:
    if args.api_index:
        return json.loads(args.api_index.read_text())
    command = [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        str(root / "core/rust/Cargo.toml"),
        "--bin",
        "hara-api-doc",
        "--",
        str(root / "core/lib/src"),
        str(root / "core/lib/test"),
    ]
    return json.loads(subprocess.check_output(command, text=True))


def build_manifest(
    api: dict[str, Any],
    inventory: list[str],
    migrations: list[dict[str, Any]],
    migration_schema: int,
    migration_path: str | None,
    aliases: list[dict[str, Any]],
    native_objects: list[dict[str, Any]],
    *,
    repository: str,
    source_ref: str,
    commit: str,
    profiles: list[str],
    inventory_path: str,
    extras: tuple[str, ...] = DEFAULT_EXTRA_NAMESPACES,
) -> dict[str, Any]:
    selected = selected_namespaces(inventory, extras)
    raw_by_name = {namespace["name"]: namespace for namespace in api.get("namespaces", [])}
    missing = sorted(set(selected) - raw_by_name.keys())
    unexpected = sorted(
        name
        for name in raw_by_name
        if (name == "std.foundation" or name.startswith("std.foundation.") or name in extras)
        and name not in selected
    )
    if missing or unexpected:
        raise ValueError(
            f"Registered/source API mismatch: missing={missing or 'none'} unexpected={unexpected or 'none'}"
        )
    current = set(selected)
    conflicting = sorted(
        migration["formerName"] for migration in migrations if migration["formerName"] in current
    )
    if conflicting:
        raise ValueError(f"Migration names are still current API: {', '.join(conflicting)}")

    namespaces = []
    for name in selected:
        namespace = dict(raw_by_name[name])
        namespace["group"] = "foundation" if name == "std.foundation" or name.startswith("std.foundation.") else "library"
        namespace["status"] = "implemented"
        namespace["profiles"] = profiles
        namespaces.append(namespace)

    semantic_surface = {
        "schemaVersion": SCHEMA_VERSION,
        "profiles": profiles,
        "namespaces": [
            {
                "name": namespace["name"],
                "group": namespace["group"],
                "status": namespace["status"],
                "definitions": [
                    {
                        "name": definition["name"],
                        "kind": definition["kind"],
                        "signature": definition.get("signature", ""),
                    }
                    for definition in namespace.get("definitions", [])
                ],
            }
            for namespace in namespaces
        ],
        "aliases": aliases,
        "nativeObjects": native_objects,
    }
    migration_document = {"schemaVersion": migration_schema, "migrations": migrations}
    return {
        "schemaVersion": SCHEMA_VERSION,
        "source": {"repository": repository, "ref": source_ref, "commit": commit},
        "generator": {"name": "generate-foundation-api-manifest", "version": "1"},
        "inventory": {
            "path": inventory_path,
            "authority": "registered-standard-library-namespaces",
        },
        "profiles": profiles,
        "surfaceDigest": digest(semantic_surface),
        "migrationLedger": {
            "schemaVersion": migration_schema,
            "path": migration_path,
            "digest": digest(migration_document),
        },
        "namespaces": namespaces,
        "aliases": aliases,
        "nativeObjects": native_objects,
        "migrations": migrations,
    }


def git_value(root: Path, *arguments: str, fallback: str) -> str:
    try:
        return subprocess.check_output(["git", "-C", str(root), *arguments], text=True).strip() or fallback
    except (OSError, subprocess.CalledProcessError):
        return fallback


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser()
    result.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    result.add_argument("--api-index", type=Path)
    result.add_argument("--inventory", type=Path)
    result.add_argument("--migrations", type=Path)
    result.add_argument("--runtime-config", type=Path)
    result.add_argument("--repository", default="https://github.com/hara-lang/hara")
    result.add_argument("--ref", dest="source_ref")
    result.add_argument("--commit")
    result.add_argument("--profiles", default="rust,jvm,wasm")
    result.add_argument("--output", type=Path)
    return result


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    root = args.root.resolve()
    inventory_path = (args.inventory or root / "core/rust/standard-library.namespaces").resolve()
    migrations_path = (args.migrations or root / "core/spec/std/foundation-migrations.json").resolve()
    runtime_config = (args.runtime_config or root / "core/rust/src/kernel/generated.rs").resolve()
    source_ref = args.source_ref or os.environ.get("HARA_API_REF") or git_value(
        root, "branch", "--show-current", fallback="detached"
    )
    commit = args.commit or os.environ.get("HARA_API_COMMIT") or os.environ.get("GITHUB_SHA") or git_value(
        root, "rev-parse", "HEAD", fallback="unknown"
    )
    profiles = sorted({profile.strip() for profile in args.profiles.split(",") if profile.strip()})
    if not profiles:
        raise ValueError("At least one runtime profile is required")
    aliases, native_objects = parse_runtime_config(runtime_config)
    migration_schema, migrations, migration_source = load_migrations(migrations_path)
    manifest = build_manifest(
        raw_api(args, root),
        read_inventory(inventory_path),
        migrations,
        migration_schema,
        migration_source,
        aliases,
        native_objects,
        repository=args.repository,
        source_ref=source_ref,
        commit=commit,
        profiles=profiles,
        inventory_path=str(inventory_path),
    )
    output = json.dumps(manifest, indent=2, ensure_ascii=False) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(output)
    else:
        sys.stdout.write(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
