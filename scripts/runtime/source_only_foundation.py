#!/usr/bin/env python3
# Reject prebuilt Foundation bundles from tracked and development runtime inputs.

from __future__ import annotations

import pathlib
import subprocess
import sys
from collections.abc import Iterable

ROOT = pathlib.Path(__file__).resolve().parents[2]
BUNDLE_NAME = "std.foundation.hbx"

FORBIDDEN_FILES = (
    pathlib.Path("core/rust/src/bin/hara-foundation-artifact.rs"),
    pathlib.Path("core/java/src/main/java/hara/truffle/HbxBundleLibrary.java"),
)
DEVELOPMENT_ROOTS = (
    pathlib.Path("core/rust/assets"),
    pathlib.Path("core/java/src/main/resources"),
    pathlib.Path("core/java/src/test/resources"),
    pathlib.Path("core/java/target/classes"),
    pathlib.Path("core/java/target/test-classes"),
)
FORBIDDEN_MARKERS = {
    pathlib.Path("core/rust/src/lib.rs"): (
        "std.foundation.hbx",
        'include_bytes!("../assets/std.foundation.hbx")',
    ),
    pathlib.Path("core/rust/Cargo.toml"): ("hara-foundation-artifact",),
    pathlib.Path("core/java/pom.xml"): (
        "embed-foundation-hbx-alpha",
        "std.foundation.hbx",
    ),
    pathlib.Path("core/java/src/main/java/hara/truffle/HaraContext.java"): (
        "HbxBundleLibrary",
        "bytecodeLibrary",
        "loadBytecodeNamespace",
    ),
    pathlib.Path(".github/workflows/lang-runtime.yml"): (
        "hara-foundation-artifact",
        "std-foundation-hbx-",
        "core/rust/assets/std.foundation.hbx",
    ),
    pathlib.Path(".github/workflows/main.yml"): (
        "core/rust/assets/std.foundation.hbx",
    ),
}
REQUIRED_MARKERS = {
    pathlib.Path("core/rust/build.rs"): (
        'manifest.join("../lib")',
        'manifest.join("hal-src")',
    ),
    pathlib.Path("core/java/pom.xml"): (
        "${project.basedir}/../lib/src",
        "${project.basedir}/../lib/src-lang",
    ),
}


def tracked_paths(root: pathlib.Path) -> list[pathlib.Path]:
    completed = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z"],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return [
        pathlib.Path(raw.decode("utf-8"))
        for raw in completed.stdout.split(b"\0")
        if raw
    ]


def audit(
    root: pathlib.Path = ROOT,
    tracked: Iterable[pathlib.Path] | None = None,
) -> list[str]:
    failures: list[str] = []
    tracked_entries = list(tracked_paths(root) if tracked is None else tracked)

    for relative in tracked_entries:
        if relative.name == BUNDLE_NAME:
            failures.append(f"tracked Foundation bundle is forbidden: {relative.as_posix()}")

    for relative in FORBIDDEN_FILES:
        if (root / relative).exists():
            failures.append(f"retired Foundation bundle bootstrap file remains: {relative.as_posix()}")

    for relative in DEVELOPMENT_ROOTS:
        directory = root / relative
        if not directory.exists():
            continue
        for candidate in sorted(directory.rglob(BUNDLE_NAME)):
            failures.append(
                "development runtime/classpath contains a Foundation bundle: "
                + candidate.relative_to(root).as_posix()
            )

    for relative, markers in FORBIDDEN_MARKERS.items():
        path = root / relative
        if not path.is_file():
            failures.append(f"required source-only audit input is missing: {relative.as_posix()}")
            continue
        source = path.read_text(encoding="utf-8")
        for marker in markers:
            if marker in source:
                failures.append(
                    f"{relative.as_posix()}: retired Foundation bundle marker remains: {marker}"
                )

    for relative, markers in REQUIRED_MARKERS.items():
        path = root / relative
        if not path.is_file():
            failures.append(f"required source layout input is missing: {relative.as_posix()}")
            continue
        source = path.read_text(encoding="utf-8")
        for marker in markers:
            if marker not in source:
                failures.append(
                    f"{relative.as_posix()}: canonical source layout marker is missing: {marker}"
                )

    return failures


def main() -> int:
    failures = audit()
    if failures:
        for failure in failures:
            print(f"ERROR: {failure}", file=sys.stderr)
        return 1
    print("source-only Foundation development runtime audit passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
