#!/usr/bin/env python3
"""Reject legacy std.foundation.os process calls in canonical HAL sources."""

from __future__ import annotations

import argparse
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Sequence

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_ROOTS = (
    ROOT / "core/lib/src",
    ROOT / "core/lib/integration",
)

LEGACY_CALLS = {
    "os/spawn": "Process/spawn",
    "os/process?": "Process/instance?",
    "os/process-alive?": "Process/alive?",
    "os/process-write": "Process/write",
    "os/process-close-input": "Process/close-input",
    "os/process-stdout": "Process/stdout",
    "os/process-stderr": "Process/stderr",
    "os/process-wait": "Process/wait",
    "os/process-kill": "Process/kill",
}


@dataclass(frozen=True)
class Finding:
    path: Path
    line: int
    column: int
    legacy: str
    replacement: str


def hal_files(roots: Sequence[Path]) -> Iterable[Path]:
    for root in roots:
        if not root.exists():
            continue
        if root.is_file():
            if root.suffix == ".hal":
                yield root
            continue
        yield from sorted(root.rglob("*.hal"))


def audit_file(path: Path) -> list[Finding]:
    findings: list[Finding] = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        for legacy, replacement in LEGACY_CALLS.items():
            start = 0
            while True:
                index = line.find(legacy, start)
                if index < 0:
                    break
                findings.append(
                    Finding(path, line_number, index + 1, legacy, replacement)
                )
                start = index + len(legacy)
    return findings


def audit(roots: Sequence[Path]) -> list[Finding]:
    findings: list[Finding] = []
    for path in hal_files(roots):
        findings.extend(audit_file(path))
    return findings


def display_path(path: Path) -> str:
    try:
        return path.resolve().relative_to(ROOT.resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "roots",
        nargs="*",
        type=Path,
        default=list(DEFAULT_ROOTS),
        help="canonical HAL files or directories to scan",
    )
    args = parser.parse_args(argv)
    findings = audit(args.roots)
    if findings:
        for finding in findings:
            print(
                f"{display_path(finding.path)}:{finding.line}:{finding.column}: "
                f"legacy {finding.legacy}; use {finding.replacement}",
                file=sys.stderr,
            )
        print(
            f"process-static-object-audit: {len(findings)} legacy call(s)",
            file=sys.stderr,
        )
        return 2
    print("process-static-object-audit: canonical Process/* surface only")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
