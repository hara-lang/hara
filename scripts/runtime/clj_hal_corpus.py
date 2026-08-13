#!/usr/bin/env python3
"""Validate the pinned, Foundation-led Clojure -> HAL migration corpus."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path
from typing import Iterable, Iterator

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_CORPUS = ROOT / "core/spec/clj-hal-corpus.json"
SHA = re.compile(r"^[0-9a-f]{40}$")
TOKEN = re.compile(r"[^\s\[\](){}\";,]+")
NS = re.compile(r"^\(\s*ns\+?\s+([^\s\[\](){}\";,]+)")
DEF = re.compile(r"^\(\s*(def[^\s\[\](){}\";,]*)\s+")
CLAUSE = re.compile(r"\(\s*:(?:require|use)\b")
REQ_VECTOR = re.compile(r"\[\s*([A-Za-z0-9_.-]+)(?=\s|\])")
NON_BINDING = {"defmethod", "defimpl", "defimpl.xt"}


class CorpusError(RuntimeError):
    pass


def git(repo: Path, *args: str) -> str:
    run = subprocess.run(
        ["git", "-C", str(repo), *args],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if run.returncode:
        raise CorpusError(run.stderr.strip() or "git command failed")
    return run.stdout


def forms(source: str) -> Iterator[str]:
    start = None
    depth = 0
    string = escaped = comment = False
    for index, char in enumerate(source):
        if comment:
            comment = char != "\n"
            continue
        if string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                string = False
            continue
        if char == ";":
            comment = True
        elif char == '"':
            string = True
        elif char == "(":
            if depth == 0:
                start = index
            depth += 1
        elif char == ")" and depth:
            depth -= 1
            if depth == 0 and start is not None:
                yield source[start : index + 1]
                start = None


def balanced(text: str, start: int) -> str:
    depth = 0
    string = escaped = comment = False
    for index in range(start, len(text)):
        char = text[index]
        if comment:
            comment = char != "\n"
            continue
        if string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                string = False
            continue
        if char == ";":
            comment = True
        elif char == '"':
            string = True
        elif char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
            if depth == 0:
                return text[start : index + 1]
    raise CorpusError("unbalanced namespace clause")


def skip_meta(text: str, offset: int) -> int:
    while True:
        while offset < len(text) and text[offset].isspace():
            offset += 1
        if offset >= len(text) or text[offset] != "^":
            return offset
        offset += 1
        if offset < len(text) and text[offset] in "[{":
            opening = text[offset]
            closing = "]" if opening == "[" else "}"
            depth = 0
            while offset < len(text):
                char = text[offset]
                if char == opening:
                    depth += 1
                elif char == closing:
                    depth -= 1
                    if depth == 0:
                        offset += 1
                        break
                offset += 1
        else:
            match = TOKEN.match(text, offset)
            if not match:
                return offset
            offset = match.end()


def source_surface(source: str) -> dict:
    ns_form = next((form for form in forms(source) if NS.match(form)), None)
    namespace = NS.match(ns_form).group(1) if ns_form else None
    dependencies: set[str] = set()
    if ns_form:
        for match in CLAUSE.finditer(ns_form):
            clause = balanced(ns_form, match.start())
            dependencies.update(REQ_VECTOR.findall(clause))
    public: set[str] = set()
    for form in forms(source):
        match = DEF.match(form)
        if not match or match.group(1).endswith("-") or match.group(1) in NON_BINDING:
            continue
        name = TOKEN.match(form, skip_meta(form, match.end()))
        if name and not name.group(0).startswith(":"):
            public.add(name.group(0))
    return {
        "namespace": namespace,
        "dependencies": sorted(dependencies),
        "public_symbols": sorted(public),
    }


def components(graph: dict[str, set[str]]) -> list[list[str]]:
    index = 0
    indices: dict[str, int] = {}
    low: dict[str, int] = {}
    stack: list[str] = []
    active: set[str] = set()
    output: list[list[str]] = []

    def visit(node: str) -> None:
        nonlocal index
        indices[node] = low[node] = index
        index += 1
        stack.append(node)
        active.add(node)
        for dependency in sorted(graph[node]):
            if dependency not in indices:
                visit(dependency)
                low[node] = min(low[node], low[dependency])
            elif dependency in active:
                low[node] = min(low[node], indices[dependency])
        if low[node] == indices[node]:
            group = []
            while True:
                member = stack.pop()
                active.remove(member)
                group.append(member)
                if member == node:
                    break
            output.append(sorted(group))

    for node in sorted(graph):
        if node not in indices:
            visit(node)
    return sorted(output, key=lambda group: tuple(group))


def dependency_plan(entries: Iterable[dict]) -> dict[str, dict]:
    names = {entry["namespace"] for entry in entries}
    graph = {
        entry["namespace"]: set(entry.get("dependencies", [])) & names
        for entry in entries
    }
    groups = components(graph)
    owner = {name: number for number, group in enumerate(groups) for name in group}
    requires = {
        number: {
            owner[dependency]
            for name in group
            for dependency in graph[name]
            if owner[dependency] != number
        }
        for number, group in enumerate(groups)
    }
    ranks: dict[int, int] = {}

    def rank(number: int) -> int:
        if number not in ranks:
            ranks[number] = 0 if not requires[number] else 1 + max(map(rank, requires[number]))
        return ranks[number]

    return {
        name: {
            "dependency_rank": rank(owner[name]),
            "dependency_component": groups[owner[name]],
            "dependency_cycle": len(groups[owner[name]]) > 1 or name in graph[name],
        }
        for name in graph
    }


def compile_entries(entries: Iterable[dict]) -> list[dict]:
    source = [dict(entry) for entry in entries]
    plan = dependency_plan(source)
    compiled = [{**entry, **plan[entry["namespace"]]} for entry in source]
    return sorted(compiled, key=lambda entry: (entry["dependency_rank"], entry["namespace"]))


def checksum(entries: list[dict]) -> str:
    encoded = json.dumps(entries, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(encoded.encode()).hexdigest()


def load(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise CorpusError(f"cannot read corpus: {error}") from error


def validate(corpus: dict) -> list[dict]:
    if corpus.get("schema_version") != 1:
        raise CorpusError("unsupported corpus schema")
    if not SHA.fullmatch(corpus.get("reference", {}).get("commit", "")):
        raise CorpusError("Foundation reference is not pinned")
    if not SHA.fullmatch(corpus.get("target", {}).get("base_commit", "")):
        raise CorpusError("Hara base is not pinned")
    allowed = set(corpus.get("status_policy", {}).get("allowed", []))
    entries = corpus.get("namespaces")
    if not entries:
        raise CorpusError("corpus has no namespace entries")
    names: set[str] = set()
    source_paths: set[str] = set()
    target_paths: set[str] = set()
    for entry in entries:
        name = entry.get("namespace")
        if not name or name in names:
            raise CorpusError(f"duplicate namespace: {name}")
        if entry.get("status") not in allowed:
            raise CorpusError(f"invalid status for {name}")
        if not SHA.fullmatch(entry.get("source_blob", "")):
            raise CorpusError(f"invalid Foundation blob for {name}")
        source_path = entry.get("source_path")
        if not source_path or source_path in source_paths or not source_path.endswith(".clj"):
            raise CorpusError(f"invalid Foundation path for {name}")
        target_path = entry.get("target_path")
        if target_path:
            if target_path in target_paths or not target_path.endswith(".hal"):
                raise CorpusError(f"invalid Hara path for {name}")
            if not SHA.fullmatch(entry.get("target_blob", "")):
                raise CorpusError(f"invalid Hara blob for {name}")
            target_paths.add(target_path)
        elif entry.get("target_blob") is not None:
            raise CorpusError(f"target blob without path for {name}")
        names.add(name)
        source_paths.add(source_path)
    if compile_entries(entries) != entries:
        raise CorpusError("entries or dependency ranks are not deterministic")
    if corpus.get("inventory_sha256") != checksum(entries):
        raise CorpusError("inventory checksum is stale")
    return entries


def verify(corpus: dict, reference: Path, target_root: Path) -> None:
    commit = corpus["reference"]["commit"]
    if git(reference, "rev-parse", f"{commit}^{{commit}}").strip() != commit:
        raise CorpusError("Foundation checkout does not contain the pinned commit")
    for entry in corpus["namespaces"]:
        name = entry["namespace"]
        source = git(reference, "show", f"{commit}:{entry['source_path']}")
        if git(reference, "rev-parse", f"{commit}:{entry['source_path']}").strip() != entry["source_blob"]:
            raise CorpusError(f"Foundation blob drift for {name}")
        surface = source_surface(source)
        if surface["namespace"] != name or surface["dependencies"] != entry["dependencies"]:
            raise CorpusError(f"Foundation surface drift for {name}")
        target_path = entry.get("target_path")
        if not target_path:
            continue
        target = target_root / target_path
        if not target.is_file():
            raise CorpusError(f"missing Hara target for {name}")
        if git(target_root, "hash-object", target_path).strip() != entry["target_blob"]:
            raise CorpusError(f"Hara blob drift for {name}")
        if source_surface(target.read_text(encoding="utf-8"))["namespace"] != entry["target_namespace"]:
            raise CorpusError(f"Hara namespace drift for {name}")


def summary(entries: list[dict]) -> None:
    statuses = Counter(entry["status"] for entry in entries)
    ranks = Counter(entry["dependency_rank"] for entry in entries)
    cycles = sum(entry["dependency_cycle"] for entry in entries)
    print(f"Clojure -> HAL corpus: {len(entries)} Foundation namespaces")
    print("  statuses: " + ", ".join(f"{key}={value}" for key, value in sorted(statuses.items())))
    print("  ranks: " + ", ".join(f"{key}={value}" for key, value in sorted(ranks.items())))
    print(f"  cyclic namespaces: {cycles}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--corpus", type=Path, default=DEFAULT_CORPUS)
    parser.add_argument("--reference", type=Path)
    parser.add_argument("--target-root", type=Path, default=ROOT)
    parser.add_argument("--verify-reference", action="store_true")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    try:
        corpus = load(args.corpus)
        entries = validate(corpus)
        if args.verify_reference:
            if args.reference is None:
                raise CorpusError("--verify-reference requires --reference")
            verify(corpus, args.reference, args.target_root)
    except CorpusError as error:
        print(f"Clojure -> HAL corpus error: {error}", file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps({"namespaces": entries}, indent=2, sort_keys=True))
    else:
        summary(entries)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
