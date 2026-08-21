#!/usr/bin/env python3
"""Validate and execute the versioned Foundation/Hara behavioral corpus.

Corpus commands use an argv array and emit one JSON object.  The object is a
semantic observation (``outcome``, ``value``, ``type``, ``display``, and
optional lifecycle/state fields), which keeps process wrappers out of the
differential comparison.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any


REPOSITORY = Path(__file__).resolve().parents[2]
CORPUS_PATH = REPOSITORY / "core/spec/code-migrate/foundation-behavioral-corpus.json"
SHA256 = re.compile(r"^[0-9a-f]{64}$")
COMMIT_SHA = re.compile(r"^[0-9a-f]{40}$")
STATUSES = {"portable", "hara-adapted", "target-specific", "deferred", "obsolete"}
NORMALIZERS = {"path", "process-wrapper", "generated-identity"}
SEMANTIC_FIELDS = (
    "outcome",
    "value",
    "type",
    "display",
    "source",
    "diagnostics",
    "lifecycle",
    "namespace",
    "mutation",
    "state",
    "ordering",
)


class CorpusError(ValueError):
    """Raised when a corpus is structurally invalid."""


def canonical_json(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def digest(value: Any) -> str:
    return hashlib.sha256(canonical_json(value).encode("utf-8")).hexdigest()


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise CorpusError(message)


def _validate_hash(record: Any, label: str) -> None:
    _require(isinstance(record, dict), f"{label} must be an object")
    _require(isinstance(record.get("path"), str) and record["path"], f"{label}.path is required")
    _require(
        isinstance(record.get("sha256"), str) and SHA256.fullmatch(record["sha256"]),
        f"{label}.sha256 must be a SHA-256 digest",
    )


def _validate_case(case: Any, index: int) -> None:
    label = f"cases[{index}]"
    _require(isinstance(case, dict), f"{label} must be an object")
    _require(isinstance(case.get("id"), str) and case["id"], f"{label}.id is required")
    _require(case.get("status") in STATUSES, f"{label}.status is invalid")
    if case["status"] in {"deferred", "obsolete"}:
        _require(
            isinstance(case.get("disposition_reason"), str) and case["disposition_reason"],
            f"{label}.disposition_reason is required for {case['status']} cases",
        )

    provenance = case.get("provenance")
    _require(isinstance(provenance, dict), f"{label}.provenance is required")
    for runtime in ("foundation", "hara"):
        runtime_record = provenance.get(runtime)
        _require(isinstance(runtime_record, dict), f"{label}.provenance.{runtime} is required")
        _validate_hash(runtime_record, f"{label}.provenance.{runtime}")
        _validate_hash(runtime_record.get("source"), f"{label}.provenance.{runtime}.source")
        _validate_hash(runtime_record.get("test"), f"{label}.provenance.{runtime}.test")

    coordinates = case.get("coordinates")
    _require(isinstance(coordinates, dict), f"{label}.coordinates is required")
    for field in ("namespace", "symbol", "grammar", "runtime"):
        _require(isinstance(coordinates.get(field), str), f"{label}.coordinates.{field} is required")

    input_data = case.get("input")
    _require(isinstance(input_data, dict), f"{label}.input is required")
    _require(
        isinstance(input_data.get("form"), str) or isinstance(input_data.get("fixture"), str),
        f"{label}.input requires form or fixture",
    )

    expectation = case.get("expectation")
    _require(isinstance(expectation, dict), f"{label}.expectation is required")
    _require(
        expectation.get("outcome") in {"success", "failure"},
        f"{label}.expectation.outcome must be success or failure",
    )
    observations = case.get("observations")
    _require(isinstance(observations, dict), f"{label}.observations is required")
    requirements = case.get("requirements")
    _require(isinstance(requirements, dict), f"{label}.requirements is required")
    _require(
        isinstance(requirements.get("deterministic"), bool),
        f"{label}.requirements.deterministic is required",
    )
    _require(
        isinstance(requirements.get("ordering"), str) and requirements["ordering"],
        f"{label}.requirements.ordering is required",
    )

    commands = case.get("commands")
    _require(isinstance(commands, dict), f"{label}.commands is required")
    for runtime in ("reference", "hara"):
        command = commands.get(runtime)
        if case["status"] in {"deferred", "obsolete"}:
            _require(command is None or isinstance(command, list), f"{label}.commands.{runtime} is invalid")
        else:
            _require(
                isinstance(command, list)
                and command
                and all(isinstance(argument, str) and argument for argument in command),
                f"{label}.commands.{runtime} must be a non-empty argv array",
            )


def validate_corpus(corpus: Any) -> dict[str, Any]:
    """Validate and return a corpus without changing its source ordering."""
    _require(isinstance(corpus, dict), "corpus must be an object")
    _require(corpus.get("document/type") == "foundation-behavioral-corpus", "invalid corpus type")
    _require(corpus.get("document/version") == 1, "unsupported corpus version")
    references = corpus.get("references")
    _require(isinstance(references, dict), "corpus.references is required")
    foundation = references.get("foundation")
    _require(isinstance(foundation, dict), "corpus.references.foundation is required")
    _require(isinstance(foundation.get("repository"), str), "foundation repository is required")
    _require(
        isinstance(foundation.get("revision"), str) and COMMIT_SHA.fullmatch(foundation["revision"]),
        "foundation revision must be a commit SHA",
    )
    normalizers = corpus.get("normalization")
    _require(isinstance(normalizers, list), "corpus.normalization is required")
    normalizer_ids = []
    for index, normalizer in enumerate(normalizers):
        _require(isinstance(normalizer, dict), f"normalization[{index}] must be an object")
        identifier = normalizer.get("id")
        _require(identifier in NORMALIZERS, f"normalization[{index}].id is invalid")
        _require(
            isinstance(normalizer.get("description"), str) and normalizer["description"],
            f"normalization[{index}].description is required",
        )
        normalizer_ids.append(identifier)
    _require(len(normalizer_ids) == len(set(normalizer_ids)), "normalization ids must be unique")

    cases = corpus.get("cases")
    _require(isinstance(cases, list) and cases, "corpus must contain cases")
    for index, case in enumerate(cases):
        _validate_case(case, index)
    ids = [case["id"] for case in cases]
    _require(len(ids) == len(set(ids)), "case ids must be unique")
    return corpus


def load_corpus(path: Path = CORPUS_PATH) -> dict[str, Any]:
    try:
        corpus = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise CorpusError(f"cannot read corpus {path}: {error}") from error
    return validate_corpus(corpus)


def _normalize_string(value: str, normalizers: set[str]) -> str:
    if "path" in normalizers:
        value = re.sub(r"(?<![\w])(?:[A-Za-z]:)?/[^ \t\r\n\"']+", "<path>", value)
    if "process-wrapper" in normalizers:
        value = re.sub(r"\b(?:Process|Subprocess)\[[^\]]*\]\s*:\s*", "", value)
    if "generated-identity" in normalizers:
        value = re.sub(
            r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-"
            r"[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b",
            "<generated>",
            value,
        )
        value = re.sub(r"\bgenerated-[A-Za-z0-9_.-]+\b", "<generated>", value)
    return value


def normalize(value: Any, normalizers: set[str]) -> Any:
    """Apply only named, documented host normalizations recursively."""
    if isinstance(value, str):
        return _normalize_string(value, normalizers)
    if isinstance(value, list):
        return [normalize(item, normalizers) for item in value]
    if isinstance(value, dict):
        return {key: normalize(item, normalizers) for key, item in value.items()}
    return value


def _decode_process(result: subprocess.CompletedProcess[str]) -> dict[str, Any]:
    stdout = result.stdout.strip()
    try:
        observation = json.loads(stdout) if stdout else {}
    except json.JSONDecodeError:
        observation = {"display": result.stdout.rstrip("\n")}
    if not isinstance(observation, dict):
        observation = {"value": observation}
    observation.setdefault("outcome", "success" if result.returncode == 0 else "failure")
    if result.returncode != 0 and result.stderr:
        observation.setdefault("diagnostics", result.stderr.rstrip("\n"))
    return observation


def execute(command: list[str], cwd: Path | None, timeout: float) -> dict[str, Any]:
    """Run one argv command without a shell and decode its semantic observation."""
    try:
        result = subprocess.run(
            command,
            cwd=str(cwd) if cwd else None,
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired as error:
        return {
            "outcome": "failure",
            "diagnostics": f"command timed out after {timeout:g}s",
        }
    except OSError as error:
        return {"outcome": "failure", "diagnostics": str(error)}
    return _decode_process(result)


def _normalizer_ids(corpus: dict[str, Any]) -> set[str]:
    return {entry["id"] for entry in corpus["normalization"]}


def _semantic(observation: dict[str, Any]) -> dict[str, Any]:
    return {field: observation[field] for field in SEMANTIC_FIELDS if field in observation}


def _expected_mismatches(expected: dict[str, Any], actual: dict[str, Any]) -> list[dict[str, Any]]:
    mismatches = []
    for field, wanted in expected.items():
        if wanted is None:
            continue
        if field in SEMANTIC_FIELDS and actual.get(field) != wanted:
            mismatches.append(
                {"field": field, "expected": wanted, "actual": actual.get(field)}
            )
    return mismatches


def compare_observations(
    case: dict[str, Any],
    reference: dict[str, Any],
    hara: dict[str, Any],
    normalizers: set[str],
) -> dict[str, Any]:
    """Compare the reference and target while retaining deterministic context."""
    reference = normalize(reference, normalizers)
    hara = normalize(hara, normalizers)
    expected = normalize(case["expectation"], normalizers)
    expected_mismatches = _expected_mismatches(expected, reference)
    fields = sorted(set(_semantic(reference)) | set(_semantic(hara)))
    differences = [
        {"field": field, "reference": reference.get(field), "hara": hara.get(field)}
        for field in fields
        if reference.get(field) != hara.get(field)
    ]
    return {
        "case_id": case["id"],
        "status": case["status"],
        "phase": case["coordinates"].get("phase", "runtime"),
        "runtime": case["coordinates"]["runtime"],
        "source_hashes": {
            "foundation": case["provenance"]["foundation"]["source"]["sha256"],
            "hara": case["provenance"]["hara"]["source"]["sha256"],
            "foundation_test": case["provenance"]["foundation"]["test"]["sha256"],
            "hara_test": case["provenance"]["hara"]["test"]["sha256"],
        },
        "reference": _semantic(reference),
        "hara": _semantic(hara),
        "expected": expected,
        "differences": differences,
        "expected_mismatches": expected_mismatches,
        "match": not differences and not expected_mismatches,
    }


def run_case(
    case: dict[str, Any],
    normalizers: set[str],
    reference_root: Path | None = None,
    hara_root: Path | None = None,
    timeout: float = 60,
) -> dict[str, Any]:
    """Execute a supported case, or record an explicit deferred disposition."""
    if case["status"] in {"deferred", "obsolete"}:
        return {
            "case_id": case["id"],
            "status": case["status"],
            "skipped": True,
            "reason": case["disposition_reason"],
            "match": True,
        }
    reference = execute(case["commands"]["reference"], reference_root, timeout)
    hara = execute(case["commands"]["hara"], hara_root, timeout)
    return compare_observations(case, reference, hara, normalizers)


def run_corpus(
    corpus: dict[str, Any],
    reference_root: Path | None = None,
    hara_root: Path | None = None,
    timeout: float = 60,
) -> dict[str, Any]:
    """Run all cases and return a stable report with no wall-clock fields."""
    validate_corpus(corpus)
    normalizers = _normalizer_ids(corpus)
    results = [
        run_case(case, normalizers, reference_root, hara_root, timeout)
        for case in corpus["cases"]
    ]
    mismatches = [result for result in results if not result["match"]]
    return {
        "document/type": "foundation-behavioral-report",
        "document/version": 1,
        "corpus_sha256": digest(corpus),
        "total": len(results),
        "skipped": sum(1 for result in results if result.get("skipped")),
        "mismatched": len(mismatches),
        "conformant": not mismatches,
        "results": results,
    }


def render_report(report: dict[str, Any]) -> str:
    """Render reports deterministically for hashing and CI artifacts."""
    return canonical_json(report) + "\n"


def parser() -> argparse.ArgumentParser:
    command = argparse.ArgumentParser()
    command.add_argument("--corpus", type=Path, default=CORPUS_PATH)
    command.add_argument("--reference-root", type=Path)
    command.add_argument("--hara-root", type=Path)
    command.add_argument("--timeout", type=float, default=60)
    command.add_argument("--run", action="store_true")
    return command


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    try:
        corpus = load_corpus(args.corpus)
        if not args.run:
            print(digest(corpus))
            return 0
        report = run_corpus(corpus, args.reference_root, args.hara_root, args.timeout)
        print(render_report(report), end="")
        return 0 if report["conformant"] else 1
    except CorpusError as error:
        print(f"Foundation behavioral corpus error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
