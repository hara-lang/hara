#!/usr/bin/env python3
"""Compare Rust and Truffle reports for the shared production code.vm corpus."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

SCHEMA = "hal.code-vm-conformance-runtime/0-alpha"


def load(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"unable to read {path}: {error}") from error
    if not isinstance(value, dict):
        raise SystemExit(f"{path} must contain a JSON object")
    if value.get("schema") != SCHEMA:
        raise SystemExit(f"{path} has unsupported schema {value.get('schema')!r}")
    return value


def cases(report: dict[str, Any], label: str) -> dict[str, dict[str, Any]]:
    raw = report.get("cases")
    if not isinstance(raw, list):
        raise SystemExit(f"{label} report is missing cases")
    output: dict[str, dict[str, Any]] = {}
    for value in raw:
        if not isinstance(value, dict) or not isinstance(value.get("id"), str):
            raise SystemExit(f"{label} report contains an invalid case")
        identifier = value["id"]
        if identifier in output:
            raise SystemExit(f"{label} report repeats case {identifier}")
        output[identifier] = value
    return output


def outcome(case: dict[str, Any], stage: str) -> dict[str, Any]:
    try:
        value = case["stages"][stage]["outcome"]
    except (KeyError, TypeError) as error:
        raise SystemExit(f"case {case.get('id')} is missing {stage} outcome") from error
    if not isinstance(value, dict):
        raise SystemExit(f"case {case.get('id')} has invalid {stage} outcome")
    return value


def comparable(value: dict[str, Any]) -> tuple[Any, Any, Any]:
    return value.get("status"), value.get("display"), value.get("category")


def required(case: dict[str, Any]) -> bool:
    try:
        return bool(case["stages"]["interpreter"]["required"])
    except (KeyError, TypeError) as error:
        raise SystemExit(
            f"case {case.get('id')} is missing interpreter required status"
        ) from error


def ensure_checks_pass(case: dict[str, Any], label: str) -> None:
    checks = case.get("checks")
    if not isinstance(checks, list):
        raise SystemExit(f"{label} case {case.get('id')} is missing checks")
    failed = [
        check.get("id")
        for check in checks
        if not isinstance(check, dict) or not check.get("pass")
    ]
    if failed:
        raise SystemExit(f"{label} case {case.get('id')} failed checks: {failed}")


def compare(rust: dict[str, Any], truffle: dict[str, Any]) -> int:
    if rust.get("corpus") != truffle.get("corpus"):
        raise SystemExit("Rust and Truffle reports identify different corpora")

    rust_cases = cases(rust, "Rust")
    truffle_cases = cases(truffle, "Truffle")
    if rust_cases.keys() != truffle_cases.keys():
        missing = sorted(rust_cases.keys() - truffle_cases.keys())
        extra = sorted(truffle_cases.keys() - rust_cases.keys())
        raise SystemExit(f"case identity mismatch: missing={missing}, extra={extra}")

    compared = 0
    for identifier in sorted(rust_cases):
        rust_case = rust_cases[identifier]
        truffle_case = truffle_cases[identifier]
        for field in ("sourceId", "namespace", "resource", "source"):
            if rust_case.get(field) != truffle_case.get(field):
                raise SystemExit(
                    f"{identifier} source identity mismatch for {field}: "
                    f"Rust={rust_case.get(field)!r}, "
                    f"Truffle={truffle_case.get(field)!r}"
                )

        # Every Truffle-local case must satisfy its declared contract. Rust
        # compile-only cases may intentionally expose bytecode/HALC failures
        # that Truffle does not execute, so they are outside this comparison.
        ensure_checks_pass(truffle_case, "Truffle")
        if not required(truffle_case):
            continue

        ensure_checks_pass(rust_case, "Rust")
        rust_outcome = comparable(outcome(rust_case, "interpreter"))
        truffle_outcome = comparable(outcome(truffle_case, "interpreter"))
        if rust_outcome != truffle_outcome:
            raise SystemExit(
                f"{identifier} interpreter outcome mismatch: "
                f"Rust={rust_outcome}, Truffle={truffle_outcome}"
            )
        compared += 1

    matrix = truffle.get("runtimeMatrix")
    if not isinstance(matrix, dict) or not matrix.get("truffle", {}).get("supported"):
        raise SystemExit(
            "Truffle report does not declare production interpreter support"
        )
    print(
        "code.vm Rust/Truffle conformance passed: "
        f"{len(rust_cases)} cases, {compared} interpreter outcomes compared"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("rust_report", type=Path)
    parser.add_argument("truffle_report", type=Path)
    arguments = parser.parse_args()
    return compare(load(arguments.rust_report), load(arguments.truffle_report))


if __name__ == "__main__":
    raise SystemExit(main())
