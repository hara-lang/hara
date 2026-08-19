#!/usr/bin/env python3
"""Deterministically inventory top-level private Hara definitions.

This is a migration guard, not a replacement for std.block source analysis.
It scans the production roots used by core/project.edn and emits TSV rows:

    namespace  path  line  kind  symbol

Supported private mechanisms are defn-, defmacro-, and :private metadata on
top-level def forms.
"""

from __future__ import annotations

import argparse
import dataclasses
from pathlib import Path
import sys
from typing import Iterable, Iterator, Sequence

DEFAULT_ROOTS = (Path("core/lib/src"), Path("core/lib/src-lang"))
OPEN_TO_CLOSE = {"(": ")", "[": "]", "{": "}"}
DELIMITERS = set("()[]{}\";, \t\r\n")


@dataclasses.dataclass(frozen=True, order=True)
class Finding:
    namespace: str
    path: str
    line: int
    kind: str
    symbol: str


def skip_space_and_comments(source: str, index: int) -> int:
    length = len(source)
    while index < length:
        character = source[index]
        if character in " \t\r\n,":
            index += 1
            continue
        if character == ";":
            newline = source.find("\n", index)
            return length if newline < 0 else skip_space_and_comments(source, newline + 1)
        return index
    return index


def skip_string(source: str, index: int) -> int:
    assert source[index] == '"'
    index += 1
    while index < len(source):
        character = source[index]
        if character == "\\":
            index += 2
        elif character == '"':
            return index + 1
        else:
            index += 1
    raise ValueError("unterminated string literal")


def skip_balanced(source: str, index: int) -> int:
    opening = source[index]
    if opening not in OPEN_TO_CLOSE:
        raise ValueError(f"expected balanced form at offset {index}")
    stack = [OPEN_TO_CLOSE[opening]]
    index += 1
    while index < len(source) and stack:
        character = source[index]
        if character == ";":
            newline = source.find("\n", index)
            index = len(source) if newline < 0 else newline + 1
        elif character == '"':
            index = skip_string(source, index)
        elif character == "\\":
            # Character literal or escaped token character.
            index += 2
        elif character in OPEN_TO_CLOSE:
            stack.append(OPEN_TO_CLOSE[character])
            index += 1
        elif character == stack[-1]:
            stack.pop()
            index += 1
        else:
            index += 1
    if stack:
        raise ValueError(f"unterminated form beginning at offset {index}")
    return index


def skip_one_form(source: str, index: int) -> int:
    index = skip_space_and_comments(source, index)
    if index >= len(source):
        return index
    character = source[index]
    if character in OPEN_TO_CLOSE:
        return skip_balanced(source, index)
    if character == '"':
        return skip_string(source, index)
    while index < len(source) and source[index] not in DELIMITERS:
        index += 1
    return index


def read_token(source: str, index: int) -> tuple[str | None, int]:
    index = skip_space_and_comments(source, index)
    if index >= len(source) or source[index] in OPEN_TO_CLOSE or source[index] in ")]}":
        return None, index
    end = index
    while end < len(source) and source[end] not in DELIMITERS:
        end += 1
    return source[index:end], end


def skip_metadata(source: str, index: int) -> tuple[list[str], int]:
    metadata: list[str] = []
    index = skip_space_and_comments(source, index)
    while index < len(source) and source[index] == "^":
        start = index
        index = skip_one_form(source, index + 1)
        metadata.append(source[start:index])
        index = skip_space_and_comments(source, index)
    return metadata, index


def top_level_lists(source: str) -> Iterator[tuple[int, str]]:
    index = 0
    depth = 0
    while index < len(source):
        character = source[index]
        if character == ";":
            newline = source.find("\n", index)
            index = len(source) if newline < 0 else newline + 1
        elif character == '"':
            index = skip_string(source, index)
        elif character == "\\":
            index += 2
        elif character == "(" and depth == 0:
            end = skip_balanced(source, index)
            yield index, source[index:end]
            index = end
        elif character in OPEN_TO_CLOSE:
            # A non-list top-level literal. Skip it as one form.
            index = skip_balanced(source, index)
        else:
            index += 1


def form_head(form: str) -> tuple[str | None, int]:
    if not form.startswith("("):
        return None, 0
    return read_token(form, 1)


def definition_name(form: str, after_head: int) -> tuple[str | None, list[str]]:
    metadata, index = skip_metadata(form, after_head)
    name, _ = read_token(form, index)
    return name, metadata


def source_namespace(source: str, path: Path) -> str:
    for _, form in top_level_lists(source):
        head, index = form_head(form)
        if head in {"ns", "ns+"}:
            name, _ = definition_name(form, index)
            if name:
                return name
    raise ValueError(f"{path}: no ns or ns+ declaration")


def private_findings(path: Path, relative_to: Path | None = None) -> list[Finding]:
    source = path.read_text(encoding="utf-8")
    namespace = source_namespace(source, path)
    display_path = str(path if relative_to is None else path.relative_to(relative_to))
    findings: list[Finding] = []
    for offset, form in top_level_lists(source):
        head, index = form_head(form)
        if head not in {"def", "defn-", "defmacro-"}:
            continue
        name, metadata = definition_name(form, index)
        if not name:
            continue
        if head == "defn-":
            kind = "defn-"
        elif head == "defmacro-":
            kind = "defmacro-"
        elif any(":private" in item for item in metadata):
            kind = "private-var"
        else:
            continue
        findings.append(
            Finding(
                namespace=namespace,
                path=display_path,
                line=source.count("\n", 0, offset) + 1,
                kind=kind,
                symbol=name,
            )
        )
    return findings


def discover(roots: Sequence[Path], repository_root: Path) -> list[Finding]:
    output: list[Finding] = []
    for root in roots:
        if not root.exists():
            raise FileNotFoundError(f"source root does not exist: {root}")
        for path in sorted(root.rglob("*.hal")):
            output.extend(private_findings(path, repository_root))
    return sorted(output)


def render_tsv(findings: Iterable[Finding]) -> str:
    rows = ["namespace\tpath\tline\tkind\tsymbol"]
    rows.extend(
        f"{item.namespace}\t{item.path}\t{item.line}\t{item.kind}\t{item.symbol}"
        for item in findings
    )
    return "\n".join(rows) + "\n"


def self_test() -> None:
    source = """(ns sample.core)
(defn- plain [x] x)
(defn- ^{:schema [:fn [:any] :any]} typed [x] x)
(defmacro- macro [] nil)
(def ^:private value 1)
(def ^{:private true :doc \"x\"} other 2)
(letfn [(local [x] x)] (local 1))
"""
    path = Path("/tmp/hara-private-definition-audit-self-test.hal")
    path.write_text(source, encoding="utf-8")
    try:
        values = private_findings(path)
    finally:
        path.unlink(missing_ok=True)
    actual = [(item.kind, item.symbol) for item in values]
    expected = [
        ("defn-", "plain"),
        ("defn-", "typed"),
        ("defmacro-", "macro"),
        ("private-var", "value"),
        ("private-var", "other"),
    ]
    if actual != expected:
        raise AssertionError(f"self-test failed: {actual!r} != {expected!r}")


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--root",
        action="append",
        dest="roots",
        type=Path,
        help="production source root; repeat to supply more than one",
    )
    parser.add_argument(
        "--repository-root",
        type=Path,
        default=Path("."),
        help="base used to render repository-relative paths",
    )
    parser.add_argument("--output", type=Path, help="write TSV to this path")
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    options = parse_args(sys.argv[1:] if argv is None else argv)
    if options.self_test:
        self_test()
        return 0
    roots = tuple(options.roots or DEFAULT_ROOTS)
    findings = discover(roots, options.repository_root)
    rendered = render_tsv(findings)
    if options.output:
        options.output.parent.mkdir(parents=True, exist_ok=True)
        options.output.write_text(rendered, encoding="utf-8")
    else:
        sys.stdout.write(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
