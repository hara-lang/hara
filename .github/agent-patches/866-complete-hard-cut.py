#!/usr/bin/env python3
"""Finish the selected code.manage translator hard cut for PR #866."""

from __future__ import annotations

import re
from pathlib import Path

CLI_PATH = Path("core/lib/src/code/manage/cli.hal")
CLI_TEST_PATH = Path("core/lib/test/code/manage_cli_test.hal")
ROUTE_TEST_PATH = Path("core/lib/test/code/migrate_manage_route_test.hal")
THIS_PATH = Path(".github/agent-patches/866-complete-hard-cut.py")


class PatchError(RuntimeError):
    pass


def top_level_forms(source: str) -> list[tuple[int, int, str]]:
    forms: list[tuple[int, int, str]] = []
    start: int | None = None
    stack: list[str] = []
    pairs = {")": "(", "]": "[", "}": "{"}
    in_string = False
    escaped = False
    in_comment = False
    in_character = False

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
        if in_character:
            if char.isspace():
                in_character = False
            continue
        if char == ";":
            in_comment = True
            continue
        if char == '"':
            in_string = True
            continue
        if char == "\\":
            in_character = True
            continue
        if char in "([{":
            if not stack and char == "(":
                start = index
            stack.append(char)
            continue
        if char in ")]}" and stack:
            if stack[-1] != pairs[char]:
                raise PatchError(f"unbalanced source near offset {index}")
            stack.pop()
            if not stack and start is not None:
                end = index + 1
                forms.append((start, end, source[start:end]))
                start = None

    if stack:
        raise PatchError("unbalanced source at EOF")
    return forms


def form_name(form: str) -> str | None:
    match = re.match(
        r"\((?:defn|defn-|defmacro|def)\s+"
        r"(?:\^\S+\s+)*([^\s\[\](){}]+)",
        form,
    )
    return match.group(1) if match else None


def rewrite_named_forms(
    source: str,
    removals: set[str],
    replacements: dict[str, str],
) -> str:
    spans: list[tuple[int, int, str]] = []
    seen: set[str] = set()
    for start, end, form in top_level_forms(source):
        name = form_name(form)
        if name in removals:
            spans.append((start, end, ""))
            seen.add(name)
        elif name in replacements:
            spans.append((start, end, replacements[name].rstrip() + "\n"))
            seen.add(name)

    missing = (removals | set(replacements)) - seen
    if missing:
        raise PatchError(f"missing expected forms: {sorted(missing)}")

    for start, end, replacement in sorted(spans, reverse=True):
        source = source[:start] + replacement + source[end:]
    return source


def balanced_span(source: str, start: int) -> tuple[int, int]:
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
        elif char in ")]}":
            if not stack or stack[-1] != pairs[char]:
                raise PatchError(f"unbalanced test near offset {index}")
            stack.pop()
            if not stack:
                return start, index + 1

    raise PatchError("unclosed test form")


def remove_test(source: str, title: str) -> str:
    marker = f'(test-check "{title}"'
    start = source.find(marker)
    if start < 0:
        raise PatchError(f"missing retired test: {title}")
    start, end = balanced_span(source, start)
    return source[:start] + source[end:]


def patch_cli() -> None:
    source = CLI_PATH.read_text(encoding="utf-8")
    source = source.replace(
        '"--form" :form "--var" :var "--corpus" :corpus\n'
        '   "--added" :added',
        '"--form" :form "--var" :var\n'
        '   "--added" :added',
    )

    removals = {
        "translation-field",
        "translation-entry",
        "translation-selected?",
        "translation-project-path",
        "translation-project",
        "translation-read-text",
        "translation-target-path",
        "dedupe-translation-units",
        "translation-routed-path",
        "translation-recipe-output",
        "translation-managed-units",
        "translation-units-for",
    }
    replacements = {
        "units-for": """(defn units-for
  [project parsed]
  (let [test-roots (:project/test-paths project)
        paths (project/files-in project
                                (roots-for project (:operation parsed))
                                ".hal")]
    (reduce
     (fn [output path]
       (if (selected? path (:namespaces parsed))
         (conj output
               (paired-unit
                project
                path
                (if (not
                     (empty?
                      (filter
                       (fn [root]
                         (str/includes?
                          path
                          (File/join (:project/root project) root)))
                       test-roots)))
                  :test
                  :source)))
         output))
     []
     paths)))""",
        "execute": """(defn execute [request]
  (let [parsed (parse-arguments (:request/arguments request))
        start (or (:project (:request/options request))
                  (:root (:request/options request))
                  (:request/cwd request)
                  (OS/cwd))
        current-project (project/discover start)
        plan (manage/plan (:operation parsed)
                          {:units (units-for current-project parsed)
                           :options (:options parsed)})
        output (render-plan plan parsed)]
    (if (:write parsed)
      (apply-edits (:project/root current-project) (:edits plan))
      nil)
    (model/success plan [(model/message (str output "\\n"))])))""",
    }

    source = rewrite_named_forms(source, removals, replacements)
    source = re.sub(r"\n{4,}", "\n\n\n", source).rstrip() + "\n"
    for token in ("clojure-to-hal", "translation-", "foundation-migrate.json"):
        if token in source:
            raise PatchError(f"retired CLI token remains: {token}")
    CLI_PATH.write_text(source, encoding="utf-8")


def patch_tests() -> None:
    source = CLI_TEST_PATH.read_text(encoding="utf-8")
    for title in (
        "translation selectors are exact corpus namespaces",
        "translation CLI retains Foundation and Hara roots",
        "semantic aggregation emits one unit for each target",
    ):
        source = remove_test(source, title)
    source = re.sub(r"\n{4,}", "\n\n\n", source).rstrip() + "\n"
    for token in ("clojure-to-hal", "translation-"):
        if token in source:
            raise PatchError(f"retired test token remains: {token}")
    CLI_TEST_PATH.write_text(source, encoding="utf-8")

    if not ROUTE_TEST_PATH.exists():
        raise PatchError(f"missing retired route test: {ROUTE_TEST_PATH}")
    ROUTE_TEST_PATH.unlink()


def main() -> int:
    patch_cli()
    patch_tests()
    THIS_PATH.unlink()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
