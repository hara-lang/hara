#!/usr/bin/env python3
"""Build and check the pinned Foundation -> Hara parity ledger.

The committed snapshot makes the check usable in Hara CI without cloning the
legacy repository.  Refreshing the snapshot is an explicit operation which
requires the pinned Foundation checkout and may also record downstream users.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path
from typing import Any, Iterable


REPOSITORY = Path(__file__).resolve().parents[2]
CONFIG_PATH = REPOSITORY / "core/spec/foundation-parity.json"
SNAPSHOT_PATH = REPOSITORY / "core/spec/foundation-parity-baseline.json"
INVENTORY_PATH = REPOSITORY / "core/spec/foundation-parity-inventory.json"
TOKEN = re.compile(r"[^\s\[\](){}\";,]+")
NS_FORM = re.compile(r"^\(\s*ns\+?\s+([^\s\[\](){}\";,]+)")
DEF_FORM = re.compile(r"^\(\s*(def[^\s\[\](){}\";,]*)\s+")
REQUIRE_ENTRY = re.compile(r"\[\s*([A-Za-z0-9_.-]+)(?=\s|\])")
NON_BINDING_DEFS = {"defmethod", "defimpl", "defimpl.xt"}
INTERN_IN = {"f/intern-in", "std.foundation/intern-in"}
INTERN_ALL = {"f/intern-all", "std.foundation/intern-all"}


def run_git(repository: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(repository), *args],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode:
        raise RuntimeError(result.stderr.strip() or "git command failed")
    return result.stdout


def git_path_exists(repository: Path, commit: str, path: str) -> bool:
    result = subprocess.run(
        ["git", "-C", str(repository), "cat-file", "-e", f"{commit}:{path}"],
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    return result.returncode == 0


def top_level_forms(source: str) -> Iterable[str]:
    """Yield top-level list forms while ignoring strings and line comments."""
    start = None
    depth = 0
    in_string = False
    escaped = False
    in_comment = False
    for index, char in enumerate(source):
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
        elif char == "(":
            if depth == 0:
                start = index
            depth += 1
        elif char == ")" and depth:
            depth -= 1
            if depth == 0 and start is not None:
                yield source[start : index + 1]
                start = None


def skip_metadata(text: str, offset: int) -> int:
    length = len(text)
    while offset < length:
        while offset < length and text[offset].isspace():
            offset += 1
        if offset >= length or text[offset] != "^":
            return offset
        offset += 1
        if offset < length and text[offset] in "{[":
            opening = text[offset]
            closing = "}" if opening == "{" else "]"
            depth = 0
            in_string = False
            escaped = False
            while offset < length:
                char = text[offset]
                if in_string:
                    if escaped:
                        escaped = False
                    elif char == "\\":
                        escaped = True
                    elif char == '"':
                        in_string = False
                elif char == '"':
                    in_string = True
                elif char == opening:
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
    return offset


def intern_form_surface(form: str, operator: str) -> tuple[set[str], set[str]]:
    without_comments = re.sub(r";[^\n]*", "", form)
    body = without_comments[without_comments.find(operator) + len(operator) : -1]
    if operator in INTERN_ALL:
        namespaces = {
            token
            for token in TOKEN.findall(body)
            if "." in token and "/" not in token and not token.startswith(":")
        }
        return set(), namespaces

    renamed = {
        match.group(1)
        for match in re.finditer(r"\[\s*([^\s\[\](){}\";,]+)\s+[^\]]+\]", body)
    }
    body = re.sub(r"\[[^\]]+\]", "", body)
    imported = {
        token.rsplit("/", 1)[1]
        for token in TOKEN.findall(body)
        if "/" in token and not token.startswith("#_")
    }
    return renamed | imported, set()


def namespace_surface(source: str) -> tuple[str | None, list[str], list[str]]:
    namespace = None
    public: set[str] = set()
    intern_all: set[str] = set()
    for form in top_level_forms(source):
        if namespace is None:
            match = NS_FORM.match(form)
            if match:
                namespace = match.group(1)
                continue
        operator_match = re.match(r"^\(\s*([^\s\[\](){}\";,]+)", form)
        if operator_match and operator_match.group(1) in INTERN_IN | INTERN_ALL:
            imported, imported_all = intern_form_surface(form, operator_match.group(1))
            public.update(imported)
            intern_all.update(imported_all)
            continue
        match = DEF_FORM.match(form)
        if not match:
            continue
        operator = match.group(1)
        if operator.endswith("-") or operator in NON_BINDING_DEFS:
            continue
        offset = skip_metadata(form, match.end())
        name = TOKEN.match(form, offset)
        if name:
            symbol = name.group(0)
            if symbol != "-" and not symbol.startswith(":"):
                public.add(symbol)
    return namespace, sorted(public), sorted(intern_all)


def namespace_and_publics(source: str) -> tuple[str | None, list[str]]:
    namespace, public, _ = namespace_surface(source)
    return namespace, public


def resolve_intern_all(entries: list[dict]) -> None:
    by_namespace = {
        entry["namespace"]: entry for entry in entries if entry.get("namespace")
    }
    changed = True
    while changed:
        changed = False
        for entry in entries:
            expanded = set(entry["public"])
            for imported in entry.get("intern_all", []):
                target = by_namespace.get(imported)
                if target:
                    expanded.update(target["public"])
            ordered = sorted(expanded)
            if ordered != entry["public"]:
                entry["public"] = ordered
                changed = True


def sha256(source: str) -> str:
    return hashlib.sha256(source.encode("utf-8")).hexdigest()


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise RuntimeError(f"cannot read {path.relative_to(REPOSITORY)}: {error}") from error


def source_paths(reference: Path, commit: str, roots: list[dict]) -> list[str]:
    output = run_git(
        reference,
        "ls-tree",
        "-r",
        "--name-only",
        commit,
        "--",
        *(item["source_root"] for item in roots),
    )
    return sorted(path for path in output.splitlines() if path.endswith(".clj"))


def _configured_roots(family: dict, key: str) -> list[str]:
    roots = family.get(key)
    if roots is None:
        root = family.get(key.removesuffix("s"))
        roots = [root] if root else []
    if isinstance(roots, str):
        roots = [roots]
    return [root.rstrip("/") for root in roots]


def reference_test_candidates(path: str, family: dict) -> list[str]:
    """Return configured Foundation test locations for a source path."""
    source_root = family["source_root"].rstrip("/") + "/"
    if not path.startswith(source_root):
        raise RuntimeError(f"source path is outside its family: {path}")
    relative = path[len(source_root) :]
    relative = relative[:-4] + "_test.clj"
    return [f"{root}/{relative}" for root in _configured_roots(family, "reference_test_roots")]


def target_test_path(path: str, family: dict) -> str | None:
    """Map a Foundation source path to its target language test path."""
    target_root = family.get("target_test_root")
    if not target_root:
        return None
    source_root = family["source_root"].rstrip("/") + "/"
    if not path.startswith(source_root):
        raise RuntimeError(f"source path is outside its family: {path}")
    relative = path[len(source_root) :]
    relative = relative[:-4] + "_test.hal"
    return target_root.rstrip("/") + "/" + relative


def macro_surface(source: str) -> list[str]:
    """Extract public macro names without confusing ordinary definitions."""
    macros = set()
    for form in top_level_forms(source):
        match = re.match(r"^\(\s*defmacro(?:\.[^\s\[\](){}\";,]*)?\s+", form)
        if not match:
            continue
        offset = skip_metadata(form, match.end())
        name = TOKEN.match(form, offset)
        if name and not name.group(0).endswith("-"):
            macros.add(name.group(0))
    return sorted(macros)


def required_namespaces(source: str, known: set[str] | None = None) -> list[str]:
    """Extract namespace dependencies from require/intern forms."""
    dependencies: set[str] = set()
    for form in top_level_forms(source):
        operator = re.match(r"^\(\s*([^\s\[\](){}\";,]+)", form)
        markers = [marker for marker in (":require", ":use", ":import") if marker in form]
        if markers:
            for marker in markers:
                body = form.split(marker, 1)[1]
                dependencies.update(match.group(1) for match in REQUIRE_ENTRY.finditer(body))
        elif operator and operator.group(1) in {"require", "require-macros"}:
            dependencies.update(match.group(1) for match in REQUIRE_ENTRY.finditer(form))
        elif operator and operator.group(1) in INTERN_ALL:
            dependencies.update(
                token
                for token in TOKEN.findall(form)
                if "." in token and "/" not in token and not token.startswith(":")
            )
    if known is not None:
        dependencies &= known
    return sorted(dependencies)


def dependency_graph(entries: list[dict]) -> dict[str, list[str]]:
    """Build a deterministic graph from the complete source inventory."""
    known = {entry["namespace"] for entry in entries if entry.get("namespace")}
    return {
        entry["namespace"]: required_namespaces(entry.get("source_blob", ""), known)
        for entry in entries
        if entry.get("namespace")
    }


def dependency_components(graph: dict[str, list[str]]) -> tuple[list[list[str]], dict[str, int], dict[int, int]]:
    """Return deterministic SCCs, owners, and dependency ranks."""
    index = 0
    indexes: dict[str, int] = {}
    lows: dict[str, int] = {}
    stack: list[str] = []
    active: set[str] = set()
    components: list[list[str]] = []

    def push(node: str) -> list[Any]:
        nonlocal index
        indexes[node] = index
        lows[node] = index
        index += 1
        stack.append(node)
        active.add(node)
        return [node, iter(graph.get(node, [])), None]

    for node in sorted(graph):
        if node not in indexes:
            frames = [push(node)]
            while frames:
                current, dependencies, parent = frames[-1]
                try:
                    dependency = next(dependencies)
                except StopIteration:
                    frames.pop()
                    if lows[current] == indexes[current]:
                        group = []
                        while True:
                            member = stack.pop()
                            active.remove(member)
                            group.append(member)
                            if member == current:
                                break
                        components.append(sorted(group))
                    if parent is not None:
                        lows[parent] = min(lows[parent], lows[current])
                    continue
                if dependency not in indexes:
                    child = push(dependency)
                    child[2] = current
                    frames.append(child)
                elif dependency in active:
                    lows[current] = min(lows[current], indexes[dependency])
    components.sort(key=lambda group: "|".join(group))
    owners = {
        namespace: number
        for number, component in enumerate(components)
        for namespace in component
    }
    requirements = {
        number: sorted(
            {
                owners[dependency]
                for namespace in component
                for dependency in graph.get(namespace, [])
                if owners[dependency] != number
            }
        )
        for number, component in enumerate(components)
    }
    memo: dict[int, int] = {}

    def rank(number: int) -> int:
        if number not in memo:
            dependencies = requirements[number]
            memo[number] = 0 if not dependencies else 1 + max(rank(dep) for dep in dependencies)
        return memo[number]

    ranks = {number: rank(number) for number in requirements}
    return components, owners, ranks


def complete_inventory(config: dict, reference: Path) -> dict:
    """Extract source/test blobs and dependency order from the Foundation tree."""
    commit = config["reference"]["commit"]
    resolved = run_git(reference, "rev-parse", f"{commit}^{{commit}}").strip()
    if resolved != commit:
        raise RuntimeError(f"reference commit resolved to {resolved}, expected {commit}")

    entries = []
    for path in source_paths(reference, commit, config["families"]):
        source = run_git(reference, "show", f"{commit}:{path}")
        namespace, public, intern_all = namespace_surface(source)
        family = family_for_path(path, config["families"])
        test_path = next(
            (
                candidate
                for candidate in reference_test_candidates(path, family)
                if git_path_exists(reference, commit, candidate)
            ),
            None,
        )
        test_source = run_git(reference, "show", f"{commit}:{test_path}") if test_path else ""
        test_namespace, test_public, _ = (
            namespace_surface(test_source) if test_path else (None, [], [])
        )
        entry = {
            "id": namespace or f"@file:{path}",
            "family": family["id"],
            "namespace": namespace,
            "source_path": path,
            "source_blob": source,
            "source_sha256": sha256(source),
            "public_symbols": public,
            "macros": macro_surface(source),
            "intern_all": intern_all,
            "dependencies": required_namespaces(source),
            "reference_test_path": test_path,
            "reference_test_blob": test_source if test_path else None,
            "reference_test_sha256": sha256(test_source) if test_path else None,
            "reference_test_namespace": test_namespace,
            "reference_test_public_symbols": test_public,
            "reference_test_macros": macro_surface(test_source) if test_path else [],
            "target_namespace": mapped_namespace(namespace, family) if namespace else None,
            "target_test_path": target_test_path(path, family),
        }
        entries.append(entry)

    known = {entry["namespace"] for entry in entries if entry.get("namespace")}
    for entry in entries:
        entry["external_dependencies"] = sorted(
            set(entry["dependencies"]) - known
        )

    targets = target_index(config)
    for entry in entries:
        target = targets.get(entry.get("target_namespace"))
        target_path = target["path"] if target else None
        target_test = (
            REPOSITORY / entry["target_test_path"]
            if entry.get("target_test_path")
            else None
        )
        entry["target_source_path"] = target_path
        entry["target_source_sha256"] = target["sha256"] if target else None
        entry["target_test_present"] = bool(target_test and target_test.is_file())
        entry["target_test_sha256"] = (
            sha256(target_test.read_text(encoding="utf-8"))
            if target_test and target_test.is_file()
            else None
        )

    graph = dependency_graph(entries)
    components, owners, ranks = dependency_components(graph)
    for entry in entries:
        namespace = entry.get("namespace")
        if namespace is None:
            continue
        owner = owners[namespace]
        entry["dependency/component"] = components[owner]
        entry["dependency/rank"] = ranks[owner]
        entry["dependency/cycle"] = len(components[owner]) > 1 or namespace in graph[namespace]

    entries.sort(key=lambda entry: (entry.get("dependency/rank", 0), entry["id"]))
    digest_input = {
        "reference_commit": commit,
        "namespaces": entries,
        "dependency_graph": graph,
        "components": components,
        "ranks": ranks,
    }
    return {
        "schema_version": 2,
        "reference_commit": commit,
        "inventory_sha256": sha256(json.dumps(digest_input, sort_keys=True, separators=(",", ":"))),
        "dependency_graph": graph,
        "components": components,
        "ranks": ranks,
        "namespaces": entries,
    }


def family_for_path(path: str, roots: list[dict]) -> dict:
    matches = [item for item in roots if path.startswith(item["source_root"] + "/")]
    if len(matches) != 1:
        raise RuntimeError(f"source path has no unique family: {path}")
    return matches[0]


def mapped_namespace(namespace: str, family: dict) -> str:
    source_prefix = family["source_namespace"]
    target_prefix = family["target_namespace"]
    if namespace == source_prefix:
        return target_prefix
    if namespace.startswith(source_prefix + "."):
        return target_prefix + namespace[len(source_prefix) :]
    raise RuntimeError(f"{namespace} does not belong to {source_prefix}")


def downstream_requires(root: Path, known: set[str]) -> set[str]:
    required: set[str] = set()
    if not root.is_dir():
        return required
    for path in root.rglob("*.clj"):
        try:
            source = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        required.update(match.group(1) for match in REQUIRE_ENTRY.finditer(source))
    return required & known


def refresh_snapshot(config: dict, reference: Path, consumers: dict[str, Path]) -> dict:
    commit = config["reference"]["commit"]
    resolved = run_git(reference, "rev-parse", f"{commit}^{{commit}}").strip()
    if resolved != commit:
        raise RuntimeError(f"reference commit resolved to {resolved}, expected {commit}")

    entries = []
    for path in source_paths(reference, commit, config["families"]):
        source = run_git(reference, "show", f"{commit}:{path}")
        namespace, public, intern_all = namespace_surface(source)
        family = family_for_path(path, config["families"])
        identifier = namespace or f"@file:{path}"
        entries.append(
            {
                "id": identifier,
                "family": family["id"],
                "source_path": path,
                "namespace": namespace,
                "target_namespace": mapped_namespace(namespace, family) if namespace else None,
                "source_sha256": sha256(source),
                "public": public,
                "intern_all": intern_all,
            }
        )

    resolve_intern_all(entries)
    known = {entry["namespace"] for entry in entries if entry["namespace"]}
    consumer_sets = {
        name: downstream_requires(path, known) for name, path in consumers.items()
    }
    for entry in entries:
        entry["consumers"] = sorted(
            name
            for name, required in consumer_sets.items()
            if entry["namespace"] and entry["namespace"] in required
        )

    digest_input = json.dumps(entries, sort_keys=True, separators=(",", ":"))
    return {
        "schema_version": 1,
        "reference_commit": commit,
        "inventory_sha256": sha256(digest_input),
        "namespaces": entries,
    }


def target_index(config: dict) -> dict[str, dict]:
    entries: list[dict] = []
    for family in config["families"]:
        root = REPOSITORY / family["target_root"]
        if not root.is_dir():
            continue
        for path in sorted(root.rglob("*.hal")):
            source = path.read_text(encoding="utf-8")
            namespace, public, intern_all = namespace_surface(source)
            if namespace:
                entries.append({
                    "namespace": namespace,
                    "path": str(path.relative_to(REPOSITORY)),
                    "sha256": sha256(source),
                    "public": public,
                    "intern_all": intern_all,
                })
    resolve_intern_all(entries)
    indexed: dict[str, dict] = {}
    for entry in entries:
        namespace = entry["namespace"]
        if namespace in indexed:
            raise RuntimeError(f"duplicate target namespace: {namespace}")
        indexed[namespace] = {**entry, "public": set(entry["public"])}
    return indexed


def ledger(config: dict, snapshot: dict) -> list[dict]:
    allowed = set(config["status_policy"]["allowed"])
    overrides = config.get("namespace_overrides", {})
    symbol_overrides = config.get("symbol_overrides", {})
    targets = target_index(config)
    result = []

    for source in snapshot["namespaces"]:
        target = targets.get(source["target_namespace"])
        identifier = source["id"]
        override = overrides.get(identifier)
        if override:
            status = override["status"]
            evidence = override.get("evidence", [])
            reason = override.get("reason")
        elif target is None:
            status, evidence, reason = "missing", [], None
        elif target["sha256"] == source["source_sha256"]:
            status, evidence, reason = "parity", ["byte-identical-source"], None
        else:
            status, evidence, reason = "different", [], None

        if status not in allowed:
            raise RuntimeError(f"unsupported status {status} for {identifier}")
        if status in {"host-adapter", "replaced", "fixture-only", "obsolete"} and not reason:
            raise RuntimeError(f"{identifier} status {status} requires a reason")
        if status == "parity" and not evidence:
            raise RuntimeError(f"{identifier} parity requires evidence")

        symbols = []
        target_public = target["public"] if target else set()
        for symbol in source["public"]:
            key = f"{identifier}/{symbol}"
            explicit = symbol_overrides.get(key)
            if explicit:
                symbol_status = explicit["status"]
                symbol_evidence = explicit.get("evidence", [])
            elif status in {"host-adapter", "replaced", "fixture-only", "obsolete"}:
                symbol_status, symbol_evidence = status, evidence
            elif status == "parity" and symbol in target_public:
                symbol_status, symbol_evidence = "parity", evidence
            elif symbol in target_public:
                symbol_status, symbol_evidence = "different", []
            else:
                symbol_status, symbol_evidence = "missing", []
            if symbol_status not in allowed:
                raise RuntimeError(f"unsupported status {symbol_status} for {key}")
            symbols.append(
                {"symbol": symbol, "status": symbol_status, "evidence": symbol_evidence}
            )

        result.append(
            {
                **source,
                "target_path": target["path"] if target else None,
                "status": status,
                "evidence": evidence,
                "reason": reason,
                "symbols": symbols,
            }
        )
    return result


def validate_snapshot(config: dict, snapshot: dict) -> None:
    commit = config["reference"]["commit"]
    if snapshot.get("reference_commit") != commit:
        raise RuntimeError("parity snapshot does not match the configured reference commit")
    entries = snapshot.get("namespaces")
    if not isinstance(entries, list) or not entries:
        raise RuntimeError("parity snapshot has no namespace inventory")
    digest_input = json.dumps(entries, sort_keys=True, separators=(",", ":"))
    if snapshot.get("inventory_sha256") != sha256(digest_input):
        raise RuntimeError("parity snapshot inventory checksum is stale")
    names = [entry["id"] for entry in entries]
    if len(names) != len(set(names)):
        raise RuntimeError("parity snapshot contains duplicate namespaces")


def print_summary(entries: list[dict]) -> None:
    namespaces = Counter(entry["status"] for entry in entries)
    symbols = Counter(
        symbol["status"] for entry in entries for symbol in entry["symbols"]
    )
    critical = [entry for entry in entries if entry["consumers"]]
    critical_status = Counter(entry["status"] for entry in critical)
    print(f"Foundation parity: {len(entries)} namespaces, {sum(symbols.values())} public symbols")
    print("  namespaces: " + ", ".join(f"{key}={value}" for key, value in sorted(namespaces.items())))
    print("  symbols: " + ", ".join(f"{key}={value}" for key, value in sorted(symbols.items())))
    print(
        f"  downstream-critical: {len(critical)} namespaces; "
        + ", ".join(f"{key}={value}" for key, value in sorted(critical_status.items()))
    )


def strict_failures(entries: list[dict], downstream_only: bool) -> list[str]:
    incomplete = {"missing", "different"}
    failures = []
    for entry in entries:
        if downstream_only and not entry["consumers"]:
            continue
        if entry["status"] in incomplete:
            failures.append(f"{entry['id']}: {entry['status']}")
            continue
        missing_symbols = [
            item["symbol"] for item in entry["symbols"] if item["status"] in incomplete
        ]
        if missing_symbols:
            failures.append(
                f"{entry['id']}: incomplete symbols " + ", ".join(missing_symbols)
            )
    return failures


def parser() -> argparse.ArgumentParser:
    default_reference = REPOSITORY.parents[2] / "reference/foundation-base"
    default_ignatius = REPOSITORY.parent / "ignatius"
    default_v2 = REPOSITORY.parents[2] / "reference/gw-v2"
    command = argparse.ArgumentParser()
    command.add_argument("--config", type=Path, default=CONFIG_PATH)
    command.add_argument("--snapshot", type=Path, default=SNAPSHOT_PATH)
    command.add_argument("--inventory", type=Path, default=INVENTORY_PATH)
    command.add_argument("--reference", type=Path, default=default_reference)
    command.add_argument("--ignatius", type=Path, default=default_ignatius)
    command.add_argument("--v2", type=Path, default=default_v2)
    command.add_argument("--refresh-snapshot", action="store_true")
    command.add_argument("--refresh-inventory", action="store_true")
    command.add_argument("--strict", action="store_true")
    command.add_argument("--downstream-strict", action="store_true")
    command.add_argument("--json", action="store_true")
    return command


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    try:
        config = load_json(args.config)
        if args.refresh_inventory:
            inventory = complete_inventory(config, args.reference)
            args.inventory.write_text(
                json.dumps(inventory, indent=2, sort_keys=True) + "\n", encoding="utf-8"
            )
            print(f"wrote {args.inventory.relative_to(REPOSITORY)}")
        if args.refresh_snapshot:
            consumers = {
                "ignatius": args.ignatius / "db",
                "v2": args.v2 / "backend",
            }
            snapshot = refresh_snapshot(config, args.reference, consumers)
            args.snapshot.write_text(
                json.dumps(snapshot, indent=2, sort_keys=True) + "\n", encoding="utf-8"
            )
            print(f"wrote {args.snapshot.relative_to(REPOSITORY)}")
        snapshot = load_json(args.snapshot)
        validate_snapshot(config, snapshot)
        entries = ledger(config, snapshot)
    except RuntimeError as error:
        print(f"Foundation parity configuration error: {error}", file=sys.stderr)
        return 2

    if args.json:
        print(json.dumps({"namespaces": entries}, indent=2, sort_keys=True))
    else:
        print_summary(entries)

    if args.strict or args.downstream_strict:
        failures = strict_failures(entries, downstream_only=args.downstream_strict and not args.strict)
        if failures:
            for failure in failures:
                print(f"incomplete Foundation parity: {failure}", file=sys.stderr)
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
