#!/usr/bin/env python3
"""Generate reviewed #554 evidence from the HAL-owned external catalog."""

from __future__ import annotations

import hashlib
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CATALOG = ROOT / "core/lib/src/code/migrate/external.hal"
CORPUS = ROOT / "core/spec/code-migrate/foundation-baa75a.edn"
OUTPUT = ROOT / "core/spec/code-migrate/external-baa75a.edn"
REPORT = ROOT / "core/spec/code-migrate/external-baa75a.md"
MANIFEST = ROOT / "core/spec/code-migrate/external-baa75a.sha256"
SELF = ROOT / ".github/agent-patches/874-generate-external.py"

PROFILE = "foundation-baa75a"
REVISION = "baa75aabd6a879753d7d5cb07271b1448271e7cb"
TREE = "26d494f60c4970df56eba8ac40f92affeee4e159"


class EvidenceError(RuntimeError):
    pass


def form_body(source: str, name: str) -> str:
    marker = f"(def {name}"
    start = source.find(marker)
    if start < 0:
        raise EvidenceError(f"missing HAL catalog form: {name}")
    stack: list[str] = []
    pairs = {")": "(", "]": "[", "}": "{"}
    in_string = False
    escaped = False
    in_comment = False
    for index in range(start, len(source)):
        char = source[index]
        if in_comment:
            if char == "\n":
                in_comment = False
            continue
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            continue
        if char == ";":
            in_comment = True
        elif char == '"':
            in_string = True
        elif char in "([{":
            stack.append(char)
        elif char in ")]}" and stack:
            if stack[-1] != pairs[char]:
                raise EvidenceError(f"unbalanced HAL catalog near {index}")
            stack.pop()
            if not stack:
                return source[start:index + 1]
    raise EvidenceError(f"unclosed HAL catalog form: {name}")


def string_set(source: str, name: str) -> set[str]:
    return set(re.findall(r'"([^"]+)"', form_body(source, name)))


def string_map(source: str, name: str) -> dict[str, str]:
    body = form_body(source, name)
    pairs = re.findall(r'"([^"]+)"\s+"([^"]+)"', body)
    return dict(pairs)


def field(body: str, key: str) -> str:
    match = re.search(r":" + re.escape(key) + r' "([^"]*)"', body)
    if not match:
        raise EvidenceError(f"external route is missing :{key}: {body[:180]}")
    return match.group(1)


def routes(source: str) -> list[dict[str, object]]:
    try:
        segment = source.split(":external/routes [", 1)[1].split(
            "] :foundation/rank-base", 1
        )[0]
    except IndexError as error:
        raise EvidenceError("cannot locate corpus external route vector") from error
    records = []
    for body in re.findall(r"\{([^{}]+)\}", segment):
        if ':external/namespace "' not in body:
            continue
        records.append(
            {
                "external/name": field(body, "external/namespace"),
                "source/namespace": field(body, "source/namespace"),
                "source/path": field(body, "source/path"),
                "source/blob": field(body, "source/blob"),
            }
        )
    if not records:
        raise EvidenceError("external route vector is empty")
    return records


def git_blob(path: Path) -> str:
    data = path.read_bytes()
    header = f"blob {len(data)}\0".encode("ascii")
    return hashlib.sha1(header + data).hexdigest()


def classify(
    name: str,
    *,
    reviewed: set[str],
    parser: set[str],
    jvm: set[str],
    readers: set[str],
    hosts: set[str],
    semantic: set[str],
    missing: set[str],
) -> dict[str, object]:
    if name not in reviewed:
        return {
            "route/kind": "namespace",
            "review/disposition": "pending",
            "rewrite/safety": "manual",
            "hara/candidate": None,
            "review/rationale": "Name is absent from the HAL-owned review catalog.",
        }
    if name in parser:
        kind = "referred-symbol"
        disposition = "obsolete"
        target = None
        rationale = "Referred or publication symbol; it is not an external namespace route."
    elif name in jvm:
        kind = "jvm-class"
        disposition = "host-runtime-adapter"
        target = None
        rationale = "Imported JVM class or interface; portable rewriting is unsafe."
    elif name == "clojure.string":
        kind = "namespace"
        disposition = "portable-substitute"
        target = "std.foundation.string"
        rationale = "Portable string surface exists with explicit historical symbol mappings."
    elif name in readers:
        kind = "namespace"
        disposition = "manual-boundary"
        target = "std.block"
        rationale = "Reader state, pushback, positions, and host reader objects are structural."
    elif name in hosts:
        kind = "namespace"
        disposition = "host-runtime-adapter"
        target = None
        rationale = "Host integration requires an explicit runtime capability or adapter."
    elif name in semantic:
        kind = "namespace"
        disposition = "semantic-replacement"
        target = {
            "clojure.data.json": "Json",
            "clojure.set": "std.foundation",
            "clojure.walk": "std.lib.collection",
        }.get(name)
        rationale = "Related Hara semantics exist, but this is not a namespace-level rename."
    elif name in missing:
        kind = "namespace"
        disposition = "missing"
        target = None
        rationale = "No portable Hara contract with exact source and test evidence was found."
    else:
        kind = "namespace"
        disposition = "manual-boundary"
        target = None
        rationale = "Project-specific external boundary; no reusable automatic rewrite is proven."
    return {
        "route/kind": kind,
        "review/disposition": disposition,
        "rewrite/safety": "review" if name == "clojure.string" else "manual",
        "hara/candidate": target,
        "review/rationale": rationale,
    }


def edn(value: object, key: str | None = None) -> str:
    keyword_values = {
        "document/type",
        "profile/id",
        "route/kind",
        "review/disposition",
        "rewrite/safety",
        "review/status",
    }
    if value is None:
        return "nil"
    if value is True:
        return "true"
    if value is False:
        return "false"
    if isinstance(value, str):
        return ":" + value if key in keyword_values else json.dumps(value)
    if isinstance(value, int):
        return str(value)
    if isinstance(value, list):
        return "[" + " ".join(edn(item) for item in value) + "]"
    if isinstance(value, dict):
        return "{" + " ".join(
            f":{current} {edn(value[current], current)}"
            for current in sorted(value)
        ) + "}"
    raise TypeError(type(value))


def main() -> int:
    catalog_source = CATALOG.read_text(encoding="utf-8")
    corpus_source = CORPUS.read_text(encoding="utf-8")
    reviewed = string_set(catalog_source, "+reviewed-names+")
    parser = string_set(catalog_source, "+parser-candidates+")
    jvm = string_set(catalog_source, "+jvm-classes+")
    readers = string_set(catalog_source, "+reader-boundaries+")
    hosts = string_set(catalog_source, "+host-namespaces+")
    semantic = string_set(catalog_source, "+semantic-namespaces+")
    missing = string_set(catalog_source, "+missing-namespaces+")
    symbols = string_map(catalog_source, "+string-symbols+")

    source_records = routes(corpus_source)
    corpus_names = {record["external/name"] for record in source_records}
    if corpus_names != reviewed:
        raise EvidenceError(
            "HAL external catalog differs from the exact corpus: "
            f"missing={sorted(corpus_names - reviewed)}, "
            f"stale={sorted(reviewed - corpus_names)}"
        )

    string_source = ROOT / "core/lib/src/std/foundation/string.hal"
    string_test = ROOT / "core/lib/test/std/foundation/string_test.hal"
    target_evidence = {
        "hara/path": string_source.relative_to(ROOT).as_posix(),
        "hara/blob": git_blob(string_source),
        "hara/test-path": string_test.relative_to(ROOT).as_posix(),
        "hara/test-blob": git_blob(string_test),
    }

    reviewed_routes = []
    for source_record in source_records:
        name = str(source_record["external/name"])
        record = dict(source_record)
        record.update(
            classify(
                name,
                reviewed=reviewed,
                parser=parser,
                jvm=jvm,
                readers=readers,
                hosts=hosts,
                semantic=semantic,
                missing=missing,
            )
        )
        record["review/status"] = "reviewed"
        record["profile/id"] = PROFILE
        record["foundation/revision"] = REVISION
        if name == "clojure.string":
            record.update(target_evidence)
            record["symbol/mappings"] = [
                {"source": source, "target": target}
                for source, target in sorted(symbols.items())
            ]
        reviewed_routes.append(record)

    reviewed_routes.sort(
        key=lambda record: (
            str(record["external/name"]),
            str(record["source/namespace"]),
            str(record["source/path"]),
        )
    )
    pending = [
        record for record in reviewed_routes
        if record["review/disposition"] == "pending"
    ]
    if pending:
        raise EvidenceError(f"pending routes remain: {len(pending)}")

    dispositions = Counter(
        str(record["review/disposition"]) for record in reviewed_routes
    )
    document = {
        "document/type": "code-migrate-external-review",
        "document/version": 1,
        "profile/id": PROFILE,
        "foundation/revision": REVISION,
        "foundation/tree": TREE,
        "route/count": len(reviewed_routes),
        "name/count": len(reviewed),
        "reviewed/count": len(reviewed_routes),
        "pending/count": 0,
        "by/disposition": dict(sorted(dispositions.items())),
        "portable/rules": [
            {
                "source": "clojure.string",
                "target": "std.foundation.string",
                "safety": "review",
                "symbol/mappings": [
                    {"source": source, "target": target}
                    for source, target in sorted(symbols.items())
                ],
                **target_evidence,
            }
        ],
        "routes": reviewed_routes,
    }
    output_source = edn(document) + "\n"
    OUTPUT.write_text(output_source, encoding="utf-8")

    report_lines = [
        "# Foundation external dependency review: `foundation-baa75a`",
        "",
        f"- Pinned revision: `{REVISION}`",
        f"- Exact route occurrences: {len(reviewed_routes)}",
        f"- Unique external names: {len(reviewed)}",
        f"- Reviewed route occurrences: {len(reviewed_routes)}",
        "- Pending route occurrences: 0",
        "",
        "## Dispositions",
        "",
    ]
    for disposition, count in sorted(dispositions.items()):
        report_lines.append(f"- `{disposition}`: {count}")
    report_lines.extend(
        [
            "",
            "## Reusable rule admission",
            "",
            "Only `clojure.string` is admitted as a portable substitute. "
            "Its exact Hara source and paired test blobs are recorded in the EDN evidence. "
            "`triml`, `trimr`, and `trim-newline` use explicit symbol mappings.",
            "",
            "Reader types, JVM classes/interfaces, project integrations, and publication "
            "symbols remain diagnostic-only. No pending candidate is counted as reviewed.",
            "",
        ]
    )
    report_source = "\n".join(report_lines)
    REPORT.write_text(report_source, encoding="utf-8")

    MANIFEST.write_text(
        f"{hashlib.sha256(output_source.encode()).hexdigest()}  {OUTPUT.name}\n"
        f"{hashlib.sha256(report_source.encode()).hexdigest()}  {REPORT.name}\n",
        encoding="utf-8",
    )
    SELF.unlink()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
