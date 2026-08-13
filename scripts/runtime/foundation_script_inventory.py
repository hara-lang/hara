#!/usr/bin/env python3
"""Generate and validate the pinned Foundation script compatibility inventory."""

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

ROOT = Path(__file__).resolve().parents[2] if len(Path(__file__).resolve().parents) >= 3 else Path.cwd()
DEFAULT_POLICY = ROOT / "core/spec/foundation-script-policy.json"
DEFAULT_OUTPUT = ROOT / "core/spec/foundation-script-inventory.json"
SHA = re.compile(r"^[0-9a-f]{40}$")
NS = re.compile(r"^\(\s*ns\+?\s+([^\s\[\](){}\";,]+)")
CLAUSE = re.compile(r"\(\s*:(?:require|use)\b")
REQ_VECTOR = re.compile(r"\[\s*([A-Za-z0-9_.-]+)(?=\s|\])")
LIST_HEAD = re.compile(r"\(\s*([^\s\[\](){}\";,]+)")
TAGGED_MACRO = re.compile(r"^(?:!|def[^./]*)\.[A-Za-z0-9_-]+$")
DEFPTR = re.compile(r"^defptr\.[A-Za-z0-9_-]+$")
SCRIPT_HEAD = re.compile(r"^\(\s*([^\s\[\](){}\";,]+)")
LANGUAGE = re.compile(r":([A-Za-z0-9_.-]+)")
SYMBOL = re.compile(r"[A-Za-z0-9_.*+!?$%&=<>:-]+(?:/[A-Za-z0-9_.*+!?$%&=<>:-]+)?")
HIGHLIGHTS = {"return", "break", "set=", "br"}


class InventoryError(RuntimeError):
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
        raise InventoryError(run.stderr.strip() or "git command failed")
    return run.stdout


def forms_with_offsets(source: str) -> Iterator[tuple[int, str]]:
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
                yield start, source[start : index + 1]
                start = None


def forms(source: str) -> Iterator[str]:
    for _, form in forms_with_offsets(source):
        yield form


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
    raise InventoryError("unbalanced namespace clause")


def scrub(source: str) -> str:
    output: list[str] = []
    string = escaped = comment = False
    for char in source:
        if comment:
            if char == "\n":
                comment = False
                output.append(char)
            else:
                output.append(" ")
            continue
        if string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                string = False
            output.append("\n" if char == "\n" else " ")
            continue
        if char == ";":
            comment = True
            output.append(" ")
        elif char == '"':
            string = True
            output.append(" ")
        else:
            output.append(char)
    return "".join(output)


def namespace_form(source: str) -> str | None:
    return next((form for form in forms(source) if NS.match(form)), None)


def source_surface(source: str) -> dict:
    ns_form = namespace_form(source)
    namespace = NS.match(ns_form).group(1) if ns_form else None
    dependencies: set[str] = set()
    if ns_form:
        for match in CLAUSE.finditer(ns_form):
            clause = balanced(ns_form, match.start())
            dependencies.update(REQ_VECTOR.findall(clause))
    return {"namespace": namespace, "dependencies": sorted(dependencies)}


def tahto_aliases(source: str) -> set[str]:
    ns_form = namespace_form(source) or ""
    aliases = {"l"}
    for match in re.finditer(r"\[\s*(?:tahto\.core|lang\.core)\b([^\]]*)\]", ns_form, re.S):
        options = match.group(1)
        alias = re.search(r":as\s+([^\s\]]+)", options)
        if alias:
            aliases.add(alias.group(1))
        if re.search(r":refer\s+\[[^\]]*\bscript-?\b", options):
            aliases.add("")
    return aliases


def script_declarations(source: str) -> list[dict]:
    aliases = tahto_aliases(source)
    accepted = {f"{alias}/script" if alias else "script" for alias in aliases}
    accepted |= {f"{alias}/script-" if alias else "script-" for alias in aliases}
    declarations: list[dict] = []
    for start, form in forms_with_offsets(source):
        head_match = SCRIPT_HEAD.match(form)
        if not head_match or head_match.group(1) not in accepted:
            continue
        head = head_match.group(1)
        rest = form[head_match.end() :]
        language_match = LANGUAGE.search(rest)
        if not language_match:
            raise InventoryError(f"script declaration has no language: {form[:80]}")
        language = language_match.group(1)
        tail = rest[language_match.end() :].lstrip()
        module = None
        if tail and tail[0] not in "{[)":
            module_match = SYMBOL.match(tail)
            if module_match and not module_match.group(0).startswith(":"):
                module = module_match.group(0)
        line = source.count("\n", 0, start) + 1
        declarations.append(
            {
                "deferred": head.endswith("-"),
                "form_sha256": hashlib.sha256(form.encode()).hexdigest(),
                "language": language,
                "line": line,
                **({"module": module} if module else {}),
            }
        )
    return declarations


def macro_surface(source: str) -> dict:
    heads = [match.group(1) for match in LIST_HEAD.finditer(scrub(source))]
    counts = Counter(heads)
    tagged = sorted({head for head in heads if TAGGED_MACRO.fullmatch(head)})
    obsolete = sorted(head for head in tagged if DEFPTR.fullmatch(head))
    required = sorted(set(tagged) - set(obsolete))
    highlights = sorted(set(heads) & HIGHLIGHTS)
    return {
        "required_macros": required,
        "obsolete_macros": obsolete,
        "highlights": highlights,
        "macro_counts": {name: counts[name] for name in required + obsolete + highlights},
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
            group: list[str] = []
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
    source = list(entries)
    names = {entry["namespace"] for entry in source}
    graph = {name: set() for name in names}
    for entry in source:
        graph[entry["namespace"]].update(
            set(entry.get("dependencies", [])) & names
        )
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


def file_blob(repo: Path, path: Path) -> str:
    return git(repo, "hash-object", str(path.relative_to(repo))).strip()


def source_blob(repo: Path, commit: str, path: str) -> str:
    return git(repo, "rev-parse", f"{commit}:{path}").strip()


def reference_paths(reference: Path, commit: str, root: str) -> list[str]:
    return [
        path
        for path in git(reference, "ls-tree", "-r", "--name-only", commit, root).splitlines()
        if path.endswith(".clj")
    ]


def target_source_path(policy: dict, source_path: str) -> str:
    source_root = policy["scope"]["source_root"].rstrip("/") + "/"
    target_root = policy["scope"]["target_root"].rstrip("/") + "/"
    relative = source_path[len(source_root) :]
    return target_root + relative[:-4] + ".hal"


def test_paths(policy: dict, source_path: str) -> tuple[str, str]:
    source_root = policy["scope"]["source_root"].rstrip("/") + "/"
    relative = source_path[len(source_root) :]
    stem = relative[:-4] + "_test"
    return (
        policy["scope"]["reference_test_root"].rstrip("/") + "/" + stem + ".clj",
        policy["scope"]["target_test_root"].rstrip("/") + "/" + stem + ".hal",
    )


def checksum(entries: list[dict]) -> str:
    encoded = json.dumps(entries, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(encoded.encode()).hexdigest()


def generate(policy: dict, reference: Path, target_root: Path) -> dict:
    commit = policy["reference"]["commit"]
    paths = reference_paths(reference, commit, policy["scope"]["source_root"])
    reference_files = set(reference_paths(reference, commit, policy["scope"]["reference_test_root"]))
    tranche_by_namespace = {
        namespace: tranche["id"]
        for tranche in policy.get("tranches", [])
        for namespace in tranche["namespaces"]
    }
    entries: list[dict] = []
    for path in paths:
        source = git(reference, "show", f"{commit}:{path}")
        declarations = script_declarations(source)
        if not declarations:
            continue
        surface = source_surface(source)
        namespace = surface["namespace"]
        if not namespace:
            raise InventoryError(f"script source has no namespace: {path}")
        source_macros = macro_surface(source)
        target_path = target_source_path(policy, path)
        target_file = target_root / target_path
        reference_test_path, target_test_path = test_paths(policy, path)
        target_test_file = target_root / target_test_path
        reference_test_present = reference_test_path in reference_files
        reference_test_source = (
            git(reference, "show", f"{commit}:{reference_test_path}")
            if reference_test_present
            else ""
        )
        reference_test_macros = macro_surface(reference_test_source)
        reference_test_scripts = script_declarations(reference_test_source)
        target_test_source = (
            target_test_file.read_text(encoding="utf-8")
            if target_test_file.is_file()
            else ""
        )
        target_test_macros = macro_surface(target_test_source)
        target_test_scripts = script_declarations(target_test_source)
        target_macros = {
            "required_macros": [],
            "obsolete_macros": [],
            "highlights": [],
            "macro_counts": {},
        }
        target_namespace = None
        if target_file.is_file():
            target_source = target_file.read_text(encoding="utf-8")
            target_namespace = source_surface(target_source)["namespace"]
            target_macros = macro_surface(target_source)
        missing_macros = sorted(
            set(source_macros["required_macros"])
            - set(target_macros["required_macros"])
        )
        status = (
            "missing"
            if not target_file.is_file()
            else "ported-with-tests"
            if target_test_file.is_file()
            else "ported"
        )
        entry = {
            "namespace": namespace,
            "source_path": path,
            "source_blob": source_blob(reference, commit, path),
            "dependencies": surface["dependencies"],
            "scripts": declarations,
            "required_macros": source_macros["required_macros"],
            "obsolete_macros": source_macros["obsolete_macros"],
            "highlights": source_macros["highlights"],
            "macro_counts": source_macros["macro_counts"],
            "target": {
                "path": target_path,
                "status": status,
                **({"namespace": target_namespace} if target_namespace else {}),
                **({"blob": file_blob(target_root, target_file)} if target_file.is_file() else {}),
                "required_macros": target_macros["required_macros"],
                "obsolete_macros": target_macros["obsolete_macros"],
                "highlights": target_macros["highlights"],
                "missing_source_macros": missing_macros,
            },
            "tests": {
                "reference_path": reference_test_path,
                "reference_present": reference_test_present,
                **(
                    {"reference_blob": source_blob(reference, commit, reference_test_path)}
                    if reference_test_present
                    else {}
                ),
                "reference_scripts": reference_test_scripts,
                "reference_required_macros": reference_test_macros["required_macros"],
                "reference_obsolete_macros": reference_test_macros["obsolete_macros"],
                "reference_highlights": reference_test_macros["highlights"],
                "target_path": target_test_path,
                "target_present": target_test_file.is_file(),
                **(
                    {"target_blob": file_blob(target_root, target_test_file)}
                    if target_test_file.is_file()
                    else {}
                ),
                "target_scripts": target_test_scripts,
                "target_required_macros": target_test_macros["required_macros"],
                "target_obsolete_macros": target_test_macros["obsolete_macros"],
                "target_highlights": target_test_macros["highlights"],
                "missing_source_macros": sorted(
                    set(reference_test_macros["required_macros"])
                    - set(target_test_macros["required_macros"])
                ),
            },
        }
        tranche = tranche_by_namespace.get(namespace)
        if tranche:
            entry["tranche"] = tranche
        entries.append(entry)
    plan = dependency_plan(entries)
    entries = sorted(
        ({**entry, **plan[entry["namespace"]]} for entry in entries),
        key=lambda entry: (
            entry["dependency_rank"],
            entry["namespace"],
            entry["source_path"],
        ),
    )
    statuses = Counter(entry["target"]["status"] for entry in entries)
    required_macros = sorted(
        {
            macro
            for entry in entries
            for macro in (
                entry["required_macros"]
                + entry["tests"]["reference_required_macros"]
            )
        }
    )
    obsolete_macros = sorted(
        {
            macro
            for entry in entries
            for macro in (
                entry["obsolete_macros"]
                + entry["tests"]["reference_obsolete_macros"]
            )
        }
    )
    highlights = sorted(
        {
            macro
            for entry in entries
            for macro in (
                entry["highlights"]
                + entry["tests"]["reference_highlights"]
            )
        }
    )
    return {
        "schema_version": 1,
        "reference": policy["reference"],
        "target": {
            "repository": policy["target"]["repository"],
            "base_commit": policy["target"]["base_commit"],
        },
        "scope": policy["scope"],
        "macro_policy": policy["macro_policy"],
        "tranches": policy.get("tranches", []),
        "summary": {
            "namespaces": len(entries),
            "statuses": dict(sorted(statuses.items())),
            "required_macros": required_macros,
            "obsolete_macros": obsolete_macros,
            "highlights": highlights,
        },
        "namespaces": entries,
        "inventory_sha256": checksum(entries),
    }


def validate(inventory: dict) -> None:
    if inventory.get("schema_version") != 1:
        raise InventoryError("unsupported inventory schema")
    commit = inventory.get("reference", {}).get("commit", "")
    if not SHA.fullmatch(commit):
        raise InventoryError("Foundation reference is not pinned")
    entries = inventory.get("namespaces", [])
    if not entries:
        raise InventoryError("script inventory has no namespaces")
    names: set[str] = set()
    source_paths: set[str] = set()
    for entry in entries:
        name = entry.get("namespace")
        source_path = entry.get("source_path")
        if not name:
            raise InventoryError("script entry has no namespace")
        if not source_path or source_path in source_paths:
            raise InventoryError(f"duplicate script source path: {source_path}")
        names.add(name)
        source_paths.add(source_path)
        if not entry.get("scripts"):
            raise InventoryError(f"script namespace has no declaration: {name}")
        if any(DEFPTR.fullmatch(macro) for macro in entry.get("required_macros", [])):
            raise InventoryError(f"defptr is incorrectly required: {name}")
        target = entry.get("target", {})
        if target.get("status") != "missing" and target.get("namespace") != name:
            raise InventoryError(f"target namespace mismatch: {name}")
    if inventory.get("inventory_sha256") != checksum(entries):
        raise InventoryError("script inventory checksum mismatch")
    entries_by_name: dict[str, list[dict]] = {}
    for entry in entries:
        entries_by_name.setdefault(entry["namespace"], []).append(entry)
    for tranche in inventory.get("tranches", []):
        missing = sorted(set(tranche["namespaces"]) - names)
        if missing:
            raise InventoryError(f"tranche contains unknown namespaces: {missing}")
        for name in tranche["namespaces"]:
            candidates = entries_by_name[name]
            if len(candidates) != 1:
                raise InventoryError(
                    f"tranche namespace is ambiguous across source paths: {name}"
                )
            entry = candidates[0]
            if entry["target"]["status"] != "ported-with-tests":
                raise InventoryError(
                    f"tranche namespace is not ported with tests: {name}"
                )
            if entry["target"]["missing_source_macros"]:
                raise InventoryError(
                    f"tranche namespace is missing source macros: {name}: "
                    f"{entry['target']['missing_source_macros']}"
                )
            if not entry["tests"]["reference_present"]:
                raise InventoryError(f"tranche reference test is missing: {name}")
            if entry["tests"]["missing_source_macros"]:
                raise InventoryError(
                    f"tranche test is missing source macros: {name}: "
                    f"{entry['tests']['missing_source_macros']}"
                )


def load(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise InventoryError(f"cannot read JSON: {path}: {error}") from error


def write(path: Path, inventory: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(inventory, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--policy", type=Path, default=DEFAULT_POLICY)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--target-root", type=Path, default=ROOT)
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args(argv)
    try:
        policy = load(args.policy)
        inventory = generate(policy, args.reference, args.target_root)
        validate(inventory)
        rendered = json.dumps(inventory, indent=2, sort_keys=True) + "\n"
        if args.write:
            write(args.output, inventory)
        if args.check:
            current = args.output.read_text(encoding="utf-8") if args.output.exists() else ""
            if current != rendered:
                print("Foundation script inventory is stale.", file=sys.stderr)
                return 1
        if not args.write and not args.check:
            sys.stdout.write(rendered)
        return 0
    except InventoryError as error:
        print(f"foundation-script-inventory: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
