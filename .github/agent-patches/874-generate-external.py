#!/usr/bin/env python3
"""Generate deterministic #554 evidence from code.migrate.external."""

from __future__ import annotations

import hashlib
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HAL = ROOT / "core/lib/src/code/migrate/external.hal"
CORPUS = ROOT / "core/spec/code-migrate/foundation-baa75a.edn"
OUT = ROOT / "core/spec/code-migrate/external-baa75a.edn"
REPORT = ROOT / "core/spec/code-migrate/external-baa75a.md"
SUMS = ROOT / "core/spec/code-migrate/external-baa75a.sha256"
SELF = ROOT / ".github/agent-patches/874-generate-external.py"
REVISION = "baa75aabd6a879753d7d5cb07271b1448271e7cb"
TREE = "26d494f60c4970df56eba8ac40f92affeee4e159"


def form(source: str, name: str) -> str:
    start = source.index(f"(def {name}")
    stack: list[str] = []
    pairs = {")": "(", "]": "[", "}": "{"}
    quoted = escaped = comment = False
    for i in range(start, len(source)):
        c = source[i]
        if comment:
            if c == "\n":
                comment = False
            continue
        if quoted:
            if escaped:
                escaped = False
            elif c == "\\":
                escaped = True
            elif c == '"':
                quoted = False
            continue
        if c == ";":
            comment = True
        elif c == '"':
            quoted = True
        elif c in "([{":
            stack.append(c)
        elif c in ")]}" and stack:
            if stack[-1] != pairs[c]:
                raise RuntimeError(f"unbalanced catalog at {i}")
            stack.pop()
            if not stack:
                return source[start:i + 1]
    raise RuntimeError(f"unclosed catalog form {name}")


def values(source: str, name: str) -> set[str]:
    return set(re.findall(r'"([^"]+)"', form(source, name)))


def mapping(source: str, name: str) -> dict[str, str]:
    return dict(re.findall(r'"([^"]+)"\s+"([^"]+)"', form(source, name)))


def route_records(source: str) -> list[dict[str, str]]:
    segment = source.split(":external/routes [", 1)[1].split(
        "] :foundation/rank-base", 1
    )[0]
    records: list[dict[str, str]] = []
    for body in re.findall(r"\{([^{}]+)\}", segment):
        if ':external/namespace "' not in body:
            continue
        record: dict[str, str] = {}
        for source_key, target_key in (
            ("external/namespace", "external/name"),
            ("source/namespace", "source/namespace"),
            ("source/path", "source/path"),
            ("source/blob", "source/blob"),
        ):
            match = re.search(
                r":" + re.escape(source_key) + r' "([^"]+)"', body
            )
            if not match:
                raise RuntimeError(f"route missing {source_key}")
            record[target_key] = match.group(1)
        records.append(record)
    return records


def blob(path: Path) -> str:
    data = path.read_bytes()
    return hashlib.sha1(
        f"blob {len(data)}\0".encode("ascii") + data
    ).hexdigest()


def classify(name: str, groups: dict[str, set[str]]) -> tuple[str, str, str | None]:
    if name in groups["parser"]:
        return "referred-symbol", "obsolete", None
    if name in groups["jvm"]:
        return "jvm-class", "host-runtime-adapter", None
    if name == "clojure.string":
        return "namespace", "portable-substitute", "std.foundation.string"
    if name in groups["reader"]:
        return "namespace", "manual-boundary", "std.block"
    if name in groups["host"]:
        return "namespace", "host-runtime-adapter", None
    if name in groups["semantic"]:
        return "namespace", "semantic-replacement", {
            "clojure.data.json": "Json",
            "clojure.set": "std.foundation",
            "clojure.walk": "std.lib.collection",
        }.get(name)
    if name in groups["missing"]:
        return "namespace", "missing", None
    return "namespace", "manual-boundary", None


def edn(value: object, key: str | None = None) -> str:
    keyword_keys = {
        "document/type", "profile/id", "route/kind",
        "review/disposition", "review/status", "rewrite/safety",
    }
    if value is None:
        return "nil"
    if value is True:
        return "true"
    if value is False:
        return "false"
    if isinstance(value, str):
        return ":" + value if key in keyword_keys else json.dumps(value)
    if isinstance(value, int):
        return str(value)
    if isinstance(value, list):
        return "[" + " ".join(edn(item) for item in value) + "]"
    if isinstance(value, dict):
        return "{" + " ".join(
            f":{item_key} {edn(value[item_key], item_key)}"
            for item_key in sorted(value)
        ) + "}"
    raise TypeError(type(value))


def main() -> int:
    hal = HAL.read_text(encoding="utf-8")
    groups = {
        "reviewed": values(hal, "+reviewed-names+"),
        "parser": values(hal, "+parser-candidates+"),
        "jvm": values(hal, "+jvm-classes+"),
        "reader": values(hal, "+reader-boundaries+"),
        "host": values(hal, "+host-namespaces+"),
        "semantic": values(hal, "+semantic-namespaces+"),
        "missing": values(hal, "+missing-namespaces+"),
    }
    symbols = mapping(hal, "+string-symbols+")
    routes = route_records(CORPUS.read_text(encoding="utf-8"))
    names = {record["external/name"] for record in routes}
    if names != groups["reviewed"]:
        raise RuntimeError(
            f"catalog drift missing={sorted(names - groups['reviewed'])} "
            f"stale={sorted(groups['reviewed'] - names)}"
        )

    string_source = ROOT / "core/lib/src/std/foundation/string.hal"
    string_test = ROOT / "core/lib/test/std/foundation/string_test.hal"
    evidence = {
        "hara/path": string_source.relative_to(ROOT).as_posix(),
        "hara/blob": blob(string_source),
        "hara/test-path": string_test.relative_to(ROOT).as_posix(),
        "hara/test-blob": blob(string_test),
    }
    reviewed: list[dict[str, object]] = []
    for route in routes:
        name = route["external/name"]
        kind, disposition, target = classify(name, groups)
        record: dict[str, object] = {
            **route,
            "foundation/revision": REVISION,
            "profile/id": "foundation-baa75a",
            "route/kind": kind,
            "review/disposition": disposition,
            "review/status": "reviewed",
            "rewrite/safety": "review" if name == "clojure.string" else "manual",
            "hara/candidate": target,
        }
        if name == "clojure.string":
            record.update(evidence)
            record["symbol/mappings"] = [
                {"source": source, "target": target_name}
                for source, target_name in sorted(symbols.items())
            ]
        reviewed.append(record)
    reviewed.sort(
        key=lambda record: (
            str(record["external/name"]),
            str(record["source/namespace"]),
            str(record["source/path"]),
        )
    )
    counts = Counter(
        str(record["review/disposition"]) for record in reviewed
    )
    document = {
        "document/type": "code-migrate-external-review",
        "document/version": 1,
        "profile/id": "foundation-baa75a",
        "foundation/revision": REVISION,
        "foundation/tree": TREE,
        "route/count": len(reviewed),
        "name/count": len(names),
        "reviewed/count": len(reviewed),
        "pending/count": 0,
        "by/disposition": dict(sorted(counts.items())),
        "portable/rules": [{
            "source": "clojure.string",
            "target": "std.foundation.string",
            "safety": "review",
            "symbol/mappings": [
                {"source": source, "target": target_name}
                for source, target_name in sorted(symbols.items())
            ],
            **evidence,
        }],
        "routes": reviewed,
    }
    out = edn(document) + "\n"
    OUT.write_text(out, encoding="utf-8")
    lines = [
        "# Foundation external dependency review: `foundation-baa75a`", "",
        f"- Pinned revision: `{REVISION}`",
        f"- Exact route occurrences: {len(reviewed)}",
        f"- Unique external names: {len(names)}",
        f"- Reviewed route occurrences: {len(reviewed)}",
        "- Pending route occurrences: 0", "", "## Dispositions", "",
    ]
    lines += [f"- `{name}`: {count}" for name, count in sorted(counts.items())]
    lines += [
        "", "## Reusable rule admission", "",
        "Only `clojure.string` is admitted as a portable substitute. Its Hara source "
        "and paired-test blobs are recorded in the EDN evidence. Historical `triml`, "
        "`trimr`, and `trim-newline` names use explicit symbol mappings.", "",
        "Reader state, JVM classes/interfaces, publication symbols, and project-specific "
        "integrations remain diagnostic-only. No pending candidate counts as reviewed.", "",
    ]
    report = "\n".join(lines)
    REPORT.write_text(report, encoding="utf-8")
    SUMS.write_text(
        f"{hashlib.sha256(out.encode()).hexdigest()}  {OUT.name}\n"
        f"{hashlib.sha256(report.encode()).hexdigest()}  {REPORT.name}\n",
        encoding="utf-8",
    )
    SELF.unlink()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
