#!/usr/bin/env python3
"""Remove the retired std.lib.walk and std.lib.test compatibility modules.

The default mode audits a checkout.  --write performs the one-time source
migration and then runs the same audit.  The transformer is deliberately
strict: unsupported aliases or call shapes abort rather than being rewritten
silently.
"""

from __future__ import annotations

import argparse
import dataclasses
import pathlib
import re
import subprocess
import sys
from collections.abc import Iterable, Iterator

ROOT = pathlib.Path(__file__).resolve().parents[2]

RETIRED = ("std.lib.walk", "std.lib.test")
MODULE_PATHS = (
    "core/lib/src/std/lib/walk.hal",
    "core/rust/hal-src/std/lib/walk.hal",
    "core/lib/src/std/lib/test.hal",
    "core/rust/hal-src/std/lib/test.hal",
)
INVENTORY = pathlib.Path("core/rust/standard-library.namespaces")
HBX = pathlib.Path("core/rust/assets/std.foundation.hbx")
FOUNDATION = pathlib.Path("core/lib/src/std/foundation.hal")
TASK_TEMPLATE = pathlib.Path("core/lib/src/std/work/template/task.hal")
TASK_PARITY = pathlib.Path("core/spec/std-work-task-parity.edn")
TASK_TEST = pathlib.Path("core/lib/test/std/work_task_redirection_test.hal")
RUST_PROJECT = pathlib.Path("core/rust/src/bin/hara/cli/project.rs")
RUN_LIB_TESTS = pathlib.Path("scripts/runtime/run-lib-tests")
JAVA_MAIN_TEST = pathlib.Path("core/java/src/test/java/hara/truffle/MainTest.java")
FOUNDATION_WALK_TEST = pathlib.Path("core/lib/test/std/foundation_walk_test.hal")
FOUNDATION_TEST_PRIMITIVES_TEST = pathlib.Path(
    "core/lib/test/std/foundation_test_primitives_test.hal"
)

# These are migration evidence, not live dependencies.  Every other occurrence
# of a retired namespace is a hard-cut failure.
LEGACY_ALLOWLIST = {
    "core/lib/src/code/translate/rules.hal",
    "core/rust/hal-src/code/translate/rules.hal",
    "core/lib/test/code/translate_rules_test.hal",
    "core/lib/test-lang/lang/core_test.hal",
    "core/spec/clj-hal-routes.json",
    "core/spec/clj-hal-corpus.json",
    "core/spec/foundation-script-inventory.json",
    "core/rust/src/lib.rs",
    "scripts/runtime/clj_hal_corpus.py",
    "scripts/runtime/retired_stdlib_hard_cut.py",
    "scripts/runtime/retired_stdlib_hard_cut_test.py",
}

# Quoted forms in these focused migration fixtures intentionally retain legacy
# input spellings.  Their executable harness is still migrated.
PRESERVE_QUOTED_LEGACY = {
    "core/lib/test/code/translate_rules_test.hal",
    "core/lib/test-lang/lang/core_test.hal",
}

WALK_MEMBERS = {
    "form?": "std.foundation/form?",
    "walk": "std.foundation/walk",
    "prewalk": "std.foundation/prewalk",
    "postwalk": "std.foundation/postwalk",
    "prewalk-replace": "std.foundation/prewalk-replace",
    "postwalk-replace": "std.foundation/postwalk-replace",
    "macroexpand-all": "std.foundation/macroexpand-all",
}
TEST_MEMBERS = {
    "check": "test-check",
    "config": "test-config",
    "passed?": "test-passed?",
}


class MigrationError(RuntimeError):
    pass


@dataclasses.dataclass
class Node:
    kind: str
    start: int
    end: int
    value: str | None = None
    children: list["Node"] = dataclasses.field(default_factory=list)
    parent: "Node | None" = None
    literal: bool = False

    def atom(self) -> str | None:
        return self.value if self.kind == "atom" else None


PREFIXES = ("~@", "#_", "#'", "'", "`", "~", "@", "^")
OPENERS = {"(": ")", "[": "]", "{": "}"}


class Reader:
    def __init__(self, source: str):
        self.source = source
        self.length = len(source)

    def skip(self, index: int) -> int:
        while index < self.length:
            ch = self.source[index]
            if ch.isspace() or ch == ",":
                index += 1
                continue
            if ch == ";":
                newline = self.source.find("\n", index)
                return self.length if newline < 0 else self.skip(newline + 1)
            return index
        return index

    def parse_all(self) -> list[Node]:
        nodes: list[Node] = []
        index = self.skip(0)
        while index < self.length:
            node, index = self.parse(index)
            nodes.append(node)
            index = self.skip(index)
        return nodes

    def parse(self, index: int) -> tuple[Node, int]:
        index = self.skip(index)
        if index >= self.length:
            raise MigrationError("unexpected end of HAL source")
        source = self.source

        # Reader prefixes. Metadata consumes metadata and value; all others one
        # form. Quote/discard nodes mark descendants as literal for focused
        # migration fixtures.
        for prefix in PREFIXES:
            if source.startswith(prefix, index):
                start = index
                index += len(prefix)
                children: list[Node] = []
                count = 2 if prefix == "^" else 1
                for _ in range(count):
                    child, index = self.parse(index)
                    children.append(child)
                node = Node("prefix", start, index, prefix, children)
                literal = prefix in {"'", "`", "#_"}
                node.literal = literal
                for child in children:
                    child.parent = node
                return node, index

        if source.startswith("#{", index):
            return self.parse_collection(index, "set", 2, "}")
        if source.startswith("#(", index):
            return self.parse_collection(index, "list", 2, ")")
        if source.startswith('#"', index):
            end = self.string_end(index + 1)
            return Node("string", index, end, source[index:end]), end
        if source[index] == '"':
            end = self.string_end(index)
            return Node("string", index, end, source[index:end]), end
        if source[index] in OPENERS:
            kinds = {"(": "list", "[": "vector", "{": "map"}
            return self.parse_collection(index, kinds[source[index]], 1, OPENERS[source[index]])
        if source[index] in ")]}":
            raise MigrationError(f"unexpected delimiter {source[index]!r} at {index}")

        # Character literals and atoms both run to a delimiter. A backslash may
        # name whitespace (\\space), so it is treated as an ordinary atom start.
        start = index
        while index < self.length:
            ch = source[index]
            if ch.isspace() or ch == "," or ch in "()[]{};\"":
                break
            index += 1
        if index == start:
            raise MigrationError(f"unable to parse HAL source at {index}: {source[index:index+20]!r}")
        return Node("atom", start, index, source[start:index]), index

    def string_end(self, index: int) -> int:
        # index points at the opening quote (or the quote in #").
        if self.source[index] != '"':
            raise AssertionError(index)
        index += 1
        escaped = False
        while index < self.length:
            ch = self.source[index]
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == '"':
                return index + 1
            index += 1
        raise MigrationError("unterminated string")

    def parse_collection(
        self, start: int, kind: str, opener_length: int, closer: str
    ) -> tuple[Node, int]:
        index = start + opener_length
        children: list[Node] = []
        while True:
            index = self.skip(index)
            if index >= self.length:
                raise MigrationError(f"unterminated {kind} starting at {start}")
            if self.source[index] == closer:
                node = Node(kind, start, index + 1, children=children)
                for child in children:
                    child.parent = node
                return node, index + 1
            child, index = self.parse(index)
            children.append(child)


def walk_nodes(nodes: Iterable[Node], literal: bool = False) -> Iterator[tuple[Node, bool]]:
    for node in nodes:
        inherited = literal or node.literal
        yield node, inherited
        # (quote ...) and (syntax-quote ...) are literal data for focused
        # migration fixtures. Elsewhere generated code is intentionally
        # migrated, so callers may ignore this flag.
        list_literal = False
        if node.kind == "list" and node.children:
            list_literal = node.children[0].atom() in {"quote", "syntax-quote"}
        yield from walk_nodes(node.children, inherited or list_literal)


def top_level_ns(nodes: list[Node]) -> Node | None:
    for node in nodes:
        if node.kind == "list" and node.children and node.children[0].atom() in {"ns", "ns+"}:
            return node
    return None


def edits_apply(source: str, edits: list[tuple[int, int, str]], path: str) -> str:
    # Nested replacements are not allowed. Apply right-to-left.
    ordered = sorted(edits, key=lambda edit: (edit[0], edit[1]))
    for previous, current in zip(ordered, ordered[1:]):
        if current[0] < previous[1]:
            raise MigrationError(
                f"{path}: overlapping edits {previous[:2]} and {current[:2]}"
            )
    output = source
    for start, end, replacement in reversed(ordered):
        output = output[:start] + replacement + output[end:]
    return output


def require_dependencies(ns: Node) -> list[Node]:
    dependencies: list[Node] = []
    for clause in ns.children[2:]:
        if clause.kind == "list" and clause.children and clause.children[0].atom() == ":require":
            dependencies.extend(
                child for child in clause.children[1:] if child.kind == "vector"
            )
    return dependencies


def dependency_info(node: Node) -> tuple[str | None, str | None]:
    if not node.children:
        return None, None
    namespace = node.children[0].atom()
    alias = None
    for index, child in enumerate(node.children[:-1]):
        if child.atom() == ":as":
            alias = node.children[index + 1].atom()
    return namespace, alias


def dependency_edits(ns: Node, targets: list[Node]) -> list[tuple[int, int, str]]:
    edits: list[tuple[int, int, str]] = []
    grouped: dict[int, tuple[Node, list[Node]]] = {}
    for target in targets:
        parent = target.parent
        if parent is None:
            raise AssertionError(target)
        grouped.setdefault(id(parent), (parent, []))[1].append(target)
    for parent, removed in grouped.values():
        dependency_children = [child for child in parent.children[1:] if child.kind == "vector"]
        if len(removed) == len(dependency_children):
            edits.append((parent.start, parent.end, ""))
        else:
            edits.extend((node.start, node.end, "") for node in removed)
    return edits


def migrate_hal(path: pathlib.Path, source: str) -> str:
    relative = path.as_posix()
    reader = Reader(source)
    nodes = reader.parse_all()
    ns = top_level_ns(nodes)
    if ns is None:
        return source

    aliases: dict[str, str] = {}
    target_nodes: list[Node] = []
    for dependency in require_dependencies(ns):
        namespace, alias = dependency_info(dependency)
        if namespace not in RETIRED:
            continue
        if not alias:
            raise MigrationError(
                f"{relative}: retired dependency {namespace} must have an explicit :as alias"
            )
        aliases[alias] = namespace
        target_nodes.append(dependency)

    # Fully-qualified legacy calls can occur without a dependency. Include
    # those synthetic prefixes in the same strict member validation.
    prefixes = dict(aliases)
    prefixes["std.lib.walk"] = "std.lib.walk"
    prefixes["std.lib.test"] = "std.lib.test"

    preserve_quoted = relative in PRESERVE_QUOTED_LEGACY
    edits: list[tuple[int, int, str]] = dependency_edits(ns, target_nodes)
    handled_spans: set[tuple[int, int]] = set()

    # Whole-call rewrites first.
    for node, literal in walk_nodes(nodes):
        if node.kind != "list" or not node.children:
            continue
        if preserve_quoted and literal:
            continue
        head = node.children[0].atom()
        if not head or "/" not in head:
            continue
        prefix, member = head.rsplit("/", 1)
        namespace = prefixes.get(prefix)
        if namespace != "std.lib.test":
            continue
        if member == "check-error":
            if len(node.children) != 3:
                raise MigrationError(
                    f"{relative}: {head} must have name and expression arguments"
                )
            name = source[node.children[1].start : node.children[1].end]
            expression = source[node.children[2].start : node.children[2].end]
            replacement = (
                f"(test-check {name}\n"
                f"    (try (do {expression} false)\n"
                f"         (catch Throwable error true))\n"
                f"    true)"
            )
            edits.append((node.start, node.end, replacement))
            handled_spans.add((node.start, node.end))
        elif member == "print-results":
            if len(node.children) != 2:
                raise MigrationError(f"{relative}: {head} must have one result argument")
            # Mark only the call head in the first pass. Nested test/check
            # forms in an inline result vector must still be migrated before
            # the wrapper is removed in the second pass.
            head_node = node.children[0]
            edits.append(
                (
                    head_node.start,
                    head_node.end,
                    "__retired_test_print_results__",
                )
            )
            handled_spans.add((head_node.start, head_node.end))

    def inside_handled(node: Node) -> bool:
        return any(start <= node.start and node.end <= end for start, end in handled_spans)

    # Symbol-level rewrites and strict unknown-member detection.
    for node, literal in walk_nodes(nodes):
        if node.kind != "atom" or inside_handled(node):
            continue
        if preserve_quoted and literal:
            continue
        value = node.atom()
        if not value or "/" not in value:
            continue
        prefix, member = value.rsplit("/", 1)
        namespace = prefixes.get(prefix)
        if namespace is None:
            continue
        if namespace == "std.lib.walk":
            replacement = WALK_MEMBERS.get(member)
            if replacement is None:
                raise MigrationError(f"{relative}: unsupported retired walk member {value}")
        else:
            replacement = TEST_MEMBERS.get(member)
            if replacement is None:
                if member in {"check-error", "print-results"}:
                    raise MigrationError(
                        f"{relative}: {value} is only supported as a direct call"
                    )
                raise MigrationError(f"{relative}: unsupported retired test member {value}")
        edits.append((node.start, node.end, replacement))

    output = edits_apply(source, edits, relative) if edits else source

    # Remove print-results wrappers after nested assertions have been
    # rewritten. The marker prevents ambiguity once the dependency alias has
    # disappeared from the ns form.
    if "__retired_test_print_results__" in output:
        second_nodes = Reader(output).parse_all()
        second_edits: list[tuple[int, int, str]] = []
        for node, literal in walk_nodes(second_nodes):
            if preserve_quoted and literal:
                continue
            if (
                node.kind == "list"
                and len(node.children) == 2
                and node.children[0].atom() == "__retired_test_print_results__"
            ):
                argument = node.children[1]
                second_edits.append(
                    (node.start, node.end, output[argument.start : argument.end])
                )
        if not second_edits:
            raise MigrationError(f"{relative}: unable to remove test/print-results marker")
        output = edits_apply(output, second_edits, relative)
    return output


def replace_once(source: str, old: str, new: str, path: pathlib.Path) -> str:
    count = source.count(old)
    if count != 1:
        raise MigrationError(f"{path}: expected one occurrence, found {count}: {old[:80]!r}")
    return source.replace(old, new, 1)


def add_foundation_form_predicate(source: str) -> str:
    if re.search(r"\n\s*form\?\n\s*\"Returns true when value is a persistent Hara source form", source):
        return source
    marker = ";; Structural walking\n;; ---------------------------------------------------------------------------\n\n"
    definition = (
        "(defn ^{:schema [:fn [:any] :bool]}\n"
        "  form?\n"
        "  \"Returns true when value is a persistent Hara source form.\"\n"
        "  [value]\n"
        "  (list? value))\n\n"
    )
    return replace_once(source, marker, marker + definition, FOUNDATION)


def update_task_template(source: str) -> str:
    source = replace_once(
        source,
        "   {:feature :execution/random :status :missing}\n",
        "   {:feature :execution/random :status :intentional-change\n"
        "    :reason \"Portable task templates preserve deterministic input order; random scheduling belongs to an execution provider or explicit caller input.\"}\n",
        TASK_TEMPLATE,
    )
    source = replace_once(
        source,
        "   {:feature :execution/deterministic-report-order :status :partial}\n",
        "   {:feature :execution/deterministic-report-order :status :implemented}\n",
        TASK_TEMPLATE,
    )
    anchor = "(defn prepare-bundle\n  [definition batch context]\n  (let [prepared (:input batch)\n        params (or (:task/params prepared) {})\n        initial (result/summarise batch (result-options definition))\n"
    replacement = (
        "(defn selected-order-completions\n"
        "  \"Orders supplied completion records by their selected item index.\n\n"
        "   Providers may complete work out of order; portable reports remain stable\n"
        "   and preserve the selected input order. Records without an item index keep\n"
        "   their supplied relative position.\"\n"
        "  [completions]\n"
        "  (let [indexed\n"
        "        (map-indexed (fn [index completion] [index completion]) completions)]\n"
        "    (vec\n"
        "     (map\n"
        "      second\n"
        "      (sort-by\n"
        "       (fn [entry]\n"
        "         (let [supplied-index (first entry)\n"
        "               completion (second entry)\n"
        "               selected-index\n"
        "               (get-in completion [:item/value :item/index])]\n"
        "           [(if (nil? selected-index) supplied-index selected-index)\n"
        "            supplied-index]))\n"
        "       indexed)))))\n\n"
        "(defn prepare-bundle\n"
        "  [definition batch context]\n"
        "  (let [prepared (:input batch)\n"
        "        params (or (:task/params prepared) {})\n"
        "        ordered-batch\n"
        "        (assoc batch\n"
        "               :results\n"
        "               (selected-order-completions (:results batch)))\n"
        "        initial (result/summarise ordered-batch (result-options definition))\n"
    )
    return replace_once(source, anchor, replacement, TASK_TEMPLATE)


def update_task_parity(source: str) -> str:
    source = replace_once(
        source,
        "  {:feature :execution/random :status :missing}\n",
        "  {:feature :execution/random :status :intentional-change\n"
        "   :reason \"Portable templates are deterministic; random scheduling belongs to a runtime/provider or explicit caller input.\"}\n",
        TASK_PARITY,
    )
    source = replace_once(
        source,
        "  {:feature :execution/deterministic-report-order :status :partial}\n",
        "  {:feature :execution/deterministic-report-order :status :implemented\n"
        "   :evidence [\"std.work.template.task/selected-order-completions\"\n"
        "              \"std.work-task-redirection-test out-of-order completion records\"]}\n",
        TASK_PARITY,
    )
    return source


def update_task_test(source: str) -> str:
    # This is applied after generic std.lib.test migration.
    definition_anchor = "(def _removed-defaults\n  (task/uninstall-defaults! :example/legacy))\n\n"
    ordered_fixture = (
        "(def ordered-completion-definition\n"
        "  (task/normalise-definition\n"
        "   {:id :example/ordered-completion\n"
        "    :item {:run (fn [value context] value)}\n"
        "    :report {:sections [:results]}\n"
        "    :return {:select :all :package :map}}))\n\n"
        "(def ordered-completion-bundle\n"
        "  (task/prepare-bundle\n"
        "   ordered-completion-definition\n"
        "   {:input {:task/params {}}\n"
        "    :items [{:item/id :first} {:item/id :second} {:item/id :third}]\n"
        "    :results\n"
        "    [{:item/id :third\n"
        "      :item/value {:item/id :third :item/index 2\n"
        "                   :status :return :data 3}}\n"
        "     {:item/id :first\n"
        "      :item/value {:item/id :first :item/index 0\n"
        "                   :status :return :data 1}}\n"
        "     {:item/id :second\n"
        "      :item/value {:item/id :second :item/index 1\n"
        "                   :status :return :data 2}}]}\n"
        "   {}))\n\n"
        "(def ordered-completion-report\n"
        "  (task/task-report ordered-completion-definition\n"
        "                    ordered-completion-bundle))\n\n"
    )
    source = replace_once(source, definition_anchor, definition_anchor + ordered_fixture, TASK_TEST)

    assertion_anchor = (
        "   (test-check\n"
        "    \"item and result output projections package map and vector returns faithfully\"\n"
    )
    ordered_assertion = (
        "   (test-check\n"
        "    \"reports preserve selected input order for out-of-order completion records\"\n"
        "    [(vec (map (fn [record] (:item/id record))\n"
        "               (:records ordered-completion-bundle)))\n"
        "     (vec (map (fn [record] (:item/id record))\n"
        "               (:section/entries\n"
        "                (report/section-by-id ordered-completion-report :results))))]\n"
        "    [[:first :second :third]\n"
        "     [:first :second :third]])\n\n"
    )
    source = replace_once(source, assertion_anchor, ordered_assertion + assertion_anchor, TASK_TEST)

    old = (
        "     (task/feature-status :report/plain-terminal-renderers)\n"
        "     (< 0 (count (task/incomplete-features)))]\n"
        "    [:implemented :implemented :implemented :implemented\n"
        "     :implemented :implemented :missing true])"
    )
    new = (
        "     (task/feature-status :report/plain-terminal-renderers)\n"
        "     (task/feature-status :execution/deterministic-report-order)\n"
        "     (task/feature-status :execution/random)\n"
        "     (count (task/incomplete-features))]\n"
        "    [:implemented :implemented :implemented :implemented\n"
        "     :implemented :implemented :implemented :implemented\n"
        "     :intentional-change 0])"
    )
    return replace_once(source, old, new, TASK_TEST)


def update_rust_project(source: str) -> str:
    source = replace_once(
        source,
        '        runtime.eval_native(include_str!("../../../../hal-src/std/lib/test.hal"))?;\n',
        "",
        RUST_PROJECT,
    )
    start = source.find("fn test_results(value: &str) -> Result<(usize, usize), String> {")
    end = source.find("\nfn hal_string(value: &str) -> String {", start)
    if start < 0 or end < 0:
        raise MigrationError(f"{RUST_PROJECT}: unable to locate test_results")
    replacement = r'''fn form_keyword<'a>(form: &'a Form, name: &str) -> Option<&'a Form> {
    let Form::Map(entries) = form else {
        return None;
    };
    entries
        .iter()
        .find(|(key, _)| matches!(key, Form::Keyword(value) if value == name))
        .map(|(_, value)| value)
}

fn form_count(form: &Form, name: &str, fallback: usize) -> Result<usize, String> {
    match form_keyword(form, name) {
        Some(Form::Number(value)) => usize::try_from(*value)
            .map_err(|_| format!("test summary :{name} must be non-negative")),
        Some(Form::BigInteger(value)) => value
            .parse::<usize>()
            .map_err(|_| format!("test summary :{name} is outside usize range")),
        Some(_) => Err(format!("test summary :{name} must be an integer")),
        None => Ok(fallback),
    }
}

fn direct_test_results(results: &[Form]) -> Result<(usize, usize), String> {
    let mut passed = 0;
    let mut failed = 0;
    for result in results {
        match form_keyword(result, "pass") {
            Some(Form::Bool(true)) => passed += 1,
            Some(Form::Bool(false)) => failed += 1,
            _ => return Err("test result is missing boolean :pass".into()),
        }
    }
    Ok((passed, failed))
}

fn structured_test_results(summary: &Form) -> Result<(usize, usize), String> {
    let status = form_keyword(summary, "status")
        .ok_or_else(|| "code.test summary is missing :status".to_owned())?;
    let Form::Keyword(status) = status else {
        return Err("code.test summary :status must be a keyword".into());
    };
    let counts = form_keyword(summary, "counts")
        .ok_or_else(|| "code.test summary is missing :counts".to_owned())?;
    if !matches!(counts, Form::Map(_)) {
        return Err("code.test summary :counts must be a map".into());
    }
    let passed_facts = form_count(counts, "passed", 0)?;
    let failed_facts = form_count(counts, "failed", 0)?;
    let errors = form_count(counts, "error", 0)?;
    let timeouts = form_count(counts, "timeout", 0)?;
    let passed = form_count(summary, "passed", passed_facts)?;
    let mut failed = form_count(summary, "failed", failed_facts + errors + timeouts)?;
    if status != "passed" && failed == 0 {
        // Preserve a failing structured outcome even when a runner reports no
        // failed assertion count (for example a cancelled execution).
        failed = 1;
    }
    Ok((passed, failed))
}

fn parsed_test_results(form: &Form) -> Result<(usize, usize), String> {
    match form {
        Form::Vector(results) | Form::List(results) => direct_test_results(results),
        Form::Map(_) => structured_test_results(form),
        _ => Err(
            "test file must return a direct result vector/list or a code.test summary".into(),
        ),
    }
}

fn test_results(value: &str) -> Result<(usize, usize), String> {
    let parsed = parse(value)?;
    match parsed {
        // Encoded strings remain representation compatibility only. New test
        // files return vectors/lists or code.test summaries directly.
        Form::String(source) => parsed_test_results(&parse(&source)?),
        form => parsed_test_results(&form),
    }
}

#[cfg(test)]
mod test_result_compatibility_tests {
    use super::test_results;

    #[test]
    fn accepts_direct_vectors_and_lists() {
        assert_eq!(
            test_results("[{:name \"pass\" :pass true} {:name \"fail\" :pass false}]")
                .unwrap(),
            (1, 1)
        );
        assert_eq!(
            test_results("({:name \"pass\" :pass true})").unwrap(),
            (1, 0)
        );
    }

    #[test]
    fn retains_encoded_vector_compatibility() {
        assert_eq!(
            test_results("\"[{:name \\\"pass\\\" :pass true}]\"").unwrap(),
            (1, 0)
        );
    }

    #[test]
    fn accepts_structured_code_test_summaries() {
        assert_eq!(
            test_results(
                "{:status :passed :counts {:passed 2 :failed 0 :error 0 :timeout 0} :passed 3 :failed 0}"
            )
            .unwrap(),
            (3, 0)
        );
        assert_eq!(
            test_results(
                "{:status :failed :counts {:passed 1 :failed 0 :error 1 :timeout 0}}"
            )
            .unwrap(),
            (1, 1)
        );
    }
}
'''
    return source[:start] + replacement + source[end:]


def update_java_main_test(source: str) -> str:
    old = (
        '          "(ns demo_app.main-test (:require [std.lib.test :as test])) "\n'
        '              + "(test/print-results [(test/check \\\"starter project runs\\\" true true)])");'
    )
    new = (
        '          "(ns demo_app.main-test) "\n'
        '              + "[(test-check \\\"starter project runs\\\" true true)]");'
    )
    return replace_once(source, old, new, JAVA_MAIN_TEST)


def foundation_walk_test_source() -> str:
    return """(ns std.foundation-walk-test)

(def post-order (atom []))
(def pre-order (atom []))

(def postwalk-value
  (postwalk
   (fn [value]
     (swap! post-order conj value)
     value)
   [1 [2]]))

(def prewalk-value
  (prewalk
   (fn [value]
     (swap! pre-order conj value)
     value)
   [1 [2]]))

(def tagged-form (with-meta '(+ 1 2) {:source :walk-test}))
(def walked-tagged-form (prewalk identity tagged-form))

(def results
  [(test-check
    \"form? recognises persistent source forms only\"
    [(form? '(+ 1 2)) (form? '()) (form? [1 2]) (form? nil) (form? 1)]
    [true true false false false])

   (test-check
    \"prewalk and postwalk visit nested forms in their documented order\"
    [(deref pre-order) (deref post-order) prewalk-value postwalk-value]
    [[[1 [2]] 1 [2] 2]
     [1 2 [2] [1 [2]]]
     [1 [2]]
     [1 [2]]])

   (test-check
    \"walking preserves metadata on persistent forms\"
    [(meta tagged-form) (meta walked-tagged-form) (= tagged-form walked-tagged-form)]
    [{:source :walk-test} {:source :walk-test} true])

   (test-check
    \"walk transforms map keys, map values, sets, lists, vectors, and scalars\"
    [(postwalk-replace {:a :b 1 2} {:a #{1} :nested '(1 [:a])})
     (walk identity identity 42)]
    [{:b #{2} :nested '(2 [:b])} 42])

   (test-check
    \"replacement walks preserve false and nil replacement values\"
    [(prewalk-replace {:false false :nil nil} [:false :nil])
     (postwalk-replace {:false false :nil nil} '(:false :nil))]
    [[false nil] '(false nil)])])

results
"""


def foundation_test_primitives_source() -> str:
    return """(ns std.foundation-test-primitives-test)

(def evaluations (atom 0))
(def passing (test-check \"inner pass\" {:a [1 2]} {:a [1 2]}))
(def failing (test-check \"inner fail\" 1 2))
(def thrown
  (test-check
   \"inner throw\"
   (do
     (swap! evaluations inc)
     (throw (ex-info \"boom\" {:kind :test})))
   :never))

(def config-result (test-config {:session-scope :fresh}))

(def results
  [(test-check
    \"test-check returns portable passing and failing maps\"
    [(:pass passing) (:status passing) (:pass failing) (:actual failing) (:expected failing)]
    [true :passed false 1 2])

   (test-check
    \"test-check captures thrown expressions and evaluates them once\"
    [(:pass thrown) (:status thrown) (:error thrown) (:error-data thrown)
     (deref evaluations)]
    [false :error \"boom\" {:kind :test} 1])

   (test-check
    \"test-check compares nested collections\"
    (:pass passing)
    true)

   (test-check
    \"test-passed? follows the portable result contract\"
    [(test-passed? passing) (test-passed? failing) (test-passed? thrown)]
    [true false false])

   (test-check
    \"test-config validates options and remains evaluation-inert\"
    config-result
    nil)])

results
"""


def update_run_lib_tests(source: str) -> str:
    retired_path = "core/lib/test/std/lib/test.hal"
    lines = source.splitlines(keepends=True)
    exclusion_count = sum(
        1 for line in lines if "! -path" in line and retired_path in line
    )
    if exclusion_count != 2:
        raise MigrationError(
            f"{RUN_LIB_TESTS}: expected two retired path exclusions, "
            f"found {exclusion_count}"
        )
    lines = [
        line
        for line in lines
        if not ("! -path" in line and retired_path in line)
    ]

    condition = f"if [ \"$target\" != '{retired_path}' ]; then"
    rewritten: list[str] = []
    condition_count = 0
    index = 0
    while index < len(lines):
        if lines[index].strip() != condition:
            rewritten.append(lines[index])
            index += 1
            continue
        condition_count += 1
        indent = lines[index][: len(lines[index]) - len(lines[index].lstrip())]
        if index + 2 >= len(lines):
            raise MigrationError(f"{RUN_LIB_TESTS}: truncated retired file guard")
        body = lines[index + 1]
        closing = lines[index + 2]
        if not body.strip().startswith("printf '%s\\n' \"$target\""):
            raise MigrationError(f"{RUN_LIB_TESTS}: unexpected retired file guard body")
        if closing.strip() != "fi" or not closing.startswith(indent):
            raise MigrationError(f"{RUN_LIB_TESTS}: unexpected retired file guard close")
        rewritten.append(body)
        index += 3

    if condition_count != 1:
        raise MigrationError(
            f"{RUN_LIB_TESTS}: expected one retired file guard, found {condition_count}"
        )
    source = "".join(rewritten)
    if retired_path in source:
        raise MigrationError(f"{RUN_LIB_TESTS}: retired exclusion remains")
    return source


def tracked_files() -> list[pathlib.Path]:
    completed = subprocess.run(
        ["git", "ls-files", "-z"], cwd=ROOT, check=True, stdout=subprocess.PIPE
    )
    return [
        ROOT / value.decode()
        for value in completed.stdout.split(b"\0")
        if value and (ROOT / value.decode()).exists()
    ]


def write_text(path: pathlib.Path, content: str) -> bool:
    full = ROOT / path if not path.is_absolute() else path
    before = full.read_text() if full.exists() else None
    if before == content:
        return False
    full.parent.mkdir(parents=True, exist_ok=True)
    full.write_text(content)
    return True


def migrate() -> list[str]:
    changed: list[str] = []

    # Canonical HAL only; Rust mirrors are regenerated by sync-rust-hal-src.
    for full in tracked_files():
        relative = full.relative_to(ROOT).as_posix()
        if full.suffix != ".hal" or relative.startswith("core/rust/hal-src/"):
            continue
        source = full.read_text()
        if not any(name in source for name in RETIRED):
            continue
        migrated = migrate_hal(full.relative_to(ROOT), source)
        if migrated != source:
            full.write_text(migrated)
            changed.append(relative)

    foundation = (ROOT / FOUNDATION).read_text()
    updated = add_foundation_form_predicate(foundation)
    if updated != foundation:
        (ROOT / FOUNDATION).write_text(updated)
        changed.append(FOUNDATION.as_posix())

    task_template = (ROOT / TASK_TEMPLATE).read_text()
    updated = update_task_template(task_template)
    if updated != task_template:
        (ROOT / TASK_TEMPLATE).write_text(updated)
        changed.append(TASK_TEMPLATE.as_posix())

    parity = (ROOT / TASK_PARITY).read_text()
    updated = update_task_parity(parity)
    if updated != parity:
        (ROOT / TASK_PARITY).write_text(updated)
        changed.append(TASK_PARITY.as_posix())

    task_test = (ROOT / TASK_TEST).read_text()
    updated = update_task_test(task_test)
    if updated != task_test:
        (ROOT / TASK_TEST).write_text(updated)
        changed.append(TASK_TEST.as_posix())

    rust_project = (ROOT / RUST_PROJECT).read_text()
    updated = update_rust_project(rust_project)
    if updated != rust_project:
        (ROOT / RUST_PROJECT).write_text(updated)
        changed.append(RUST_PROJECT.as_posix())

    java_main_test = (ROOT / JAVA_MAIN_TEST).read_text()
    updated = update_java_main_test(java_main_test)
    if updated != java_main_test:
        (ROOT / JAVA_MAIN_TEST).write_text(updated)
        changed.append(JAVA_MAIN_TEST.as_posix())

    for test_path, test_source in (
        (FOUNDATION_WALK_TEST, foundation_walk_test_source()),
        (FOUNDATION_TEST_PRIMITIVES_TEST, foundation_test_primitives_source()),
    ):
        full = ROOT / test_path
        if not full.exists() or full.read_text() != test_source:
            full.parent.mkdir(parents=True, exist_ok=True)
            full.write_text(test_source)
            changed.append(test_path.as_posix())

    runner = (ROOT / RUN_LIB_TESTS).read_text()
    updated = update_run_lib_tests(runner)
    if updated != runner:
        (ROOT / RUN_LIB_TESTS).write_text(updated)
        changed.append(RUN_LIB_TESTS.as_posix())

    inventory_path = ROOT / INVENTORY
    inventory = inventory_path.read_text().splitlines()
    next_inventory = [line for line in inventory if line not in RETIRED]
    if next_inventory != inventory:
        inventory_path.write_text("\n".join(next_inventory) + "\n")
        changed.append(INVENTORY.as_posix())

    for module in MODULE_PATHS:
        path = ROOT / module
        if path.exists():
            path.unlink()
            changed.append(module)

    return sorted(set(changed))


def audit() -> list[str]:
    failures: list[str] = []
    tracked = tracked_files()
    tracked_relative = {path.relative_to(ROOT).as_posix() for path in tracked}

    for module in MODULE_PATHS:
        if module in tracked_relative or (ROOT / module).exists():
            failures.append(f"retired source path remains: {module}")

    inventory = (ROOT / INVENTORY).read_text().splitlines()
    for namespace in RETIRED:
        if namespace in inventory:
            failures.append(f"retired namespace remains in inventory: {namespace}")

    for full in tracked:
        relative = full.relative_to(ROOT).as_posix()
        try:
            data = full.read_bytes()
        except OSError as error:
            failures.append(f"cannot read {relative}: {error}")
            continue
        if not any(namespace.encode() in data for namespace in RETIRED):
            continue
        if relative == HBX.as_posix():
            failures.append(f"retired namespace remains embedded in HBX: {relative}")
        elif relative not in LEGACY_ALLOWLIST:
            names = [name for name in RETIRED if name.encode() in data]
            failures.append(f"live retired namespace reference in {relative}: {', '.join(names)}")

    foundation = (ROOT / FOUNDATION).read_text()
    if not re.search(r"\(defn \^\{:schema \[:fn \[:any\] :bool\]\}\s+form\?", foundation):
        failures.append("std.foundation/form? is missing or lacks its public schema")
    for operation in ("walk", "prewalk", "postwalk", "prewalk-replace", "postwalk-replace"):
        if not re.search(rf"\n\s+{re.escape(operation)}\n\s+\"", foundation):
            failures.append(f"std.foundation/{operation} is missing documentation")

    project = (ROOT / RUST_PROJECT).read_text()
    if "hal-src/std/lib/test.hal" in project:
        failures.append("Rust CLI still explicitly loads std.lib.test")
    if "direct result vector/list or a code.test summary" not in project:
        failures.append("Rust CLI direct/structured result compatibility is missing")

    runner = (ROOT / RUN_LIB_TESTS).read_text()
    if "std/lib/test.hal" in runner:
        failures.append("library runner still excludes the retired test module")

    java_fixture = (ROOT / JAVA_MAIN_TEST).read_text()
    if "std.lib.test" in java_fixture or "test/print-results" in java_fixture:
        failures.append("Java compatibility fixture still returns legacy test output")

    for focused in (FOUNDATION_WALK_TEST, FOUNDATION_TEST_PRIMITIVES_TEST):
        if not (ROOT / focused).is_file():
            failures.append(f"focused Foundation test is missing: {focused}")
        elif not (ROOT / focused).read_text().rstrip().endswith("results"):
            failures.append(
                f"focused Foundation test does not return a direct result vector: {focused}"
            )

    task = (ROOT / TASK_TEMPLATE).read_text()
    required_task_markers = (
        ":execution/random :status :intentional-change",
        ":execution/deterministic-report-order :status :implemented",
        "selected-order-completions",
    )
    for marker in required_task_markers:
        if marker not in task:
            failures.append(f"std.work parity marker missing: {marker}")

    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true", help="apply the one-time migration")
    parser.add_argument(
        "--defer-audit",
        action="store_true",
        help="return after applying the migration so generated mirrors can be synchronized",
    )
    parser.add_argument(
        "--skip-hbx", action="store_true", help="ignore HBX legacy strings before regeneration"
    )
    args = parser.parse_args()

    try:
        if args.defer_audit and not args.write:
            raise MigrationError("--defer-audit requires --write")
        if args.write:
            changed = migrate()
            print(f"migrated {len(changed)} paths")
            for path in changed:
                print(path)
            if args.defer_audit:
                print("migration applied; post-sync audit deferred")
                return 0
        failures = audit()
        if args.skip_hbx:
            failures = [failure for failure in failures if "embedded in HBX" not in failure]
        if failures:
            for failure in failures:
                print(f"ERROR: {failure}", file=sys.stderr)
            return 1
        print("retired stdlib hard-cut audit passed")
        return 0
    except (MigrationError, OSError, subprocess.CalledProcessError) as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
