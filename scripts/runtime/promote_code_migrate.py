#!/usr/bin/env python3
"""One-shot promotion for the code.translate hard cut.

The script rewrites only the checked-out Hara worktree. Foundation is read from
an externally prepared bare repository at the exact pinned commit. This file
and its workflow remove themselves before the product commit.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
from collections import defaultdict
from pathlib import Path
from typing import Iterator

ROOT = Path(__file__).resolve().parents[2]
FOUNDATION_GIT = Path(os.environ.get("FOUNDATION_GIT", ROOT / ".foundation.git"))
FOUNDATION_REVISION = "baa75aabd6a879753d7d5cb07271b1448271e7cb"
FOUNDATION_TREE = "26d494f60c4970df56eba8ac40f92affeee4e159"
LEGACY_REVISION = "f909a44ad504cc6c9f44248d59ec3148261a48d7"

TRANSPORT_FILES = [
    ROOT / "scripts/runtime/promote_code_migrate.py",
    ROOT / ".github/workflows/promote-code-migrate.yml",
]

TEXT_SUFFIXES = {
    "", ".hal", ".edn", ".md", ".json", ".yml", ".yaml", ".py",
    ".sh", ".txt", ".toml", ".rs", ".java", ".xml",
}

SOURCE_MOVES = {
    "core/lib/src/code/translate.hal": "core/lib/src/code/migrate/project.hal",
    "core/lib/src/code/translate/clojure.hal": "core/lib/src/code/migrate/clojure.hal",
    "core/lib/src/code/translate/rule.hal": "core/lib/src/code/migrate/rule.hal",
    "core/lib/src/code/translate/rules.hal": "core/lib/src/code/migrate/rules.hal",
    "core/lib/src/code/translate/navigation.hal": "core/lib/src/code/framework/navigation.hal",
    "core/rust/hal-src/code/translate.hal": "core/rust/hal-src/code/migrate/project.hal",
    "core/rust/hal-src/code/translate/clojure.hal": "core/rust/hal-src/code/migrate/clojure.hal",
    "core/rust/hal-src/code/translate/rule.hal": "core/rust/hal-src/code/migrate/rule.hal",
    "core/rust/hal-src/code/translate/rules.hal": "core/rust/hal-src/code/migrate/rules.hal",
    "core/rust/hal-src/code/translate/navigation.hal": "core/rust/hal-src/code/framework/navigation.hal",
}

REPLACEMENTS = [
    (":code.translate/", ":code.migrate/"),
    ("code.translate-navigation-test", "code.framework-navigation-test"),
    ("code.translate-rules-test", "code.migrate-rules-test"),
    ("code.translate-clojure-test", "code.migrate-clojure-test"),
    ("code.translate-project-test", "code.migrate-project-test"),
    ("code.translate-catalog-test", "code.migrate-catalog-test"),
    ("code.translate-manage-route-test", "code.migrate-manage-route-test"),
    ("code.translate-namespace-shape-test", "code.migrate-namespace-shape-test"),
    ("code.translate.navigation", "code.framework.navigation"),
    ("code.translate.clojure", "code.migrate.clojure"),
    ("code.translate.rules", "code.migrate.rules"),
    ("code.translate.rule", "code.migrate.rule"),
    ("code.translate-", "code.migrate-"),
    ("code.translate", "code.migrate.project"),
    ("code/translate/navigation.hal", "code/framework/navigation.hal"),
    ("code/translate/clojure.hal", "code/migrate/clojure.hal"),
    ("code/translate/rules.hal", "code/migrate/rules.hal"),
    ("code/translate/rule.hal", "code/migrate/rule.hal"),
    ("code/translate.hal", "code/migrate/project.hal"),
    ("code/translate/", "code/migrate/"),
    ("code-translate", "code-migrate"),
    ("clj-hal-corpus", "foundation-migrate"),
    ("clj_hal_corpus", "foundation_migrate"),
    ("Clojure to HAL corpus", "Foundation migration audit"),
    ("Clojure-to-HAL corpus", "Foundation migration audit"),
    ("Clojure -> HAL corpus", "Foundation migration audit"),
]


class PromotionError(RuntimeError):
    pass


def run(*argv: str, cwd: Path = ROOT, input_text: str | None = None) -> str:
    result = subprocess.run(
        list(argv), cwd=cwd, input=input_text, text=True,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
    )
    if result.returncode:
        detail = result.stderr.strip() or result.stdout.strip() or "command failed"
        raise PromotionError(f"{' '.join(argv)}: {detail}")
    return result.stdout


def git_object(*argv: str, input_text: str | None = None) -> str:
    allowed = {"rev-parse", "ls-tree", "cat-file", "hash-object"}
    forbidden = {"-w", "--write", "--stdin-paths"}
    if not argv or argv[0] not in allowed or forbidden.intersection(argv):
        raise PromotionError(f"rejected Foundation git argv: {argv!r}")
    return run(
        "git", "--git-dir", str(FOUNDATION_GIT), *argv,
        input_text=input_text,
    )


def read(path: str | Path) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str | Path, source: str, executable: bool = False) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(source, encoding="utf-8")
    if executable:
        target.chmod(0o755)


def remove(path: str | Path) -> None:
    target = ROOT / path
    if target.is_dir():
        shutil.rmtree(target)
    elif target.exists() or target.is_symlink():
        target.unlink()


def move(old: str, new: str) -> None:
    source = ROOT / old
    target = ROOT / new
    if not source.exists():
        return
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists():
        if source.read_bytes() != target.read_bytes():
            raise PromotionError(f"move target differs: {old} -> {new}")
        source.unlink()
    else:
        source.rename(target)


def text_files() -> Iterator[Path]:
    excluded = {
        ROOT / ".git", ROOT / ".foundation.git", ROOT / "core/java/target",
        ROOT / "core/rust/target", ROOT / "core/target",
    }
    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        if any(root == path or root in path.parents for root in excluded):
            continue
        if path.stat().st_size > 4_000_000:
            continue
        if path.suffix not in TEXT_SUFFIXES and path.name not in {
            "standard-library.namespaces", "run-lib-tests",
        }:
            continue
        try:
            path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        yield path


def global_replace() -> None:
    for path in text_files():
        source = path.read_text(encoding="utf-8")
        output = source
        for old, new in REPLACEMENTS:
            output = output.replace(old, new)
        if output != source:
            path.write_text(output, encoding="utf-8")


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
                raise PromotionError(f"unbalanced source near offset {index}")
            stack.pop()
            if not stack and start is not None:
                end = index + 1
                forms.append((start, end, source[start:end]))
                start = None

    if stack:
        raise PromotionError("unbalanced source at EOF")
    return forms


def form_name(form: str) -> str | None:
    match = re.match(
        r"\((?:defn|defn-|defmacro|def)\s+(?:\^\S+\s+)*([^\s\[\](){}]+)",
        form,
    )
    return match.group(1) if match else None


def replace_form(path: str, name: str, replacement: str | None) -> None:
    source = read(path)
    matches = [
        (start, end) for start, end, form in top_level_forms(source)
        if form_name(form) == name
    ]
    if len(matches) != 1:
        raise PromotionError(
            f"{path}: expected one top-level form {name}, found {len(matches)}"
        )
    start, end = matches[0]
    inserted = "" if replacement is None else replacement.rstrip() + "\n"
    write(path, source[:start] + inserted + source[end:])


def add_require(path: str, spec: str) -> None:
    source = read(path)
    if spec in source:
        return
    marker = "(:require "
    index = source.find(marker)
    if index < 0:
        raise PromotionError(f"{path}: ns form has no :require clause")
    insert = index + len(marker)
    write(path, source[:insert] + spec + "\n            " + source[insert:])


def append_once(path: str, marker: str, source: str) -> None:
    current = read(path)
    if marker in current:
        return
    write(path, current.rstrip() + "\n\n" + source.rstrip() + "\n")


def move_sources_and_tests() -> None:
    for old, new in SOURCE_MOVES.items():
        move(old, new)
    for root in (ROOT / "core/lib/test/code", ROOT / "core/rust/hal-test/code"):
        if not root.exists():
            continue
        for path in sorted(root.glob("translate*_test.hal")):
            name = re.sub(r"^translate(?:_|-)?", "migrate_", path.name)
            target = path.with_name(name)
            if target.exists() and target.read_bytes() != path.read_bytes():
                raise PromotionError(f"test move target differs: {path} -> {target}")
            if target.exists():
                path.unlink()
            else:
                path.rename(target)


def consolidate_recipes() -> None:
    old_root = ROOT / "core/spec/code-translate"
    canonical_root = ROOT / "core/spec/code-migrate"
    if not old_root.exists():
        return
    for source in sorted(old_root.rglob("*")):
        if not source.is_file():
            continue
        relative = source.relative_to(old_root)
        target = canonical_root / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        if relative.parts and relative.parts[0] == "recipes":
            recipe_relative = Path(*relative.parts[1:])
            current = (
                ROOT / "core/lib/test" / recipe_relative
                if recipe_relative.name.endswith("_test.hal")
                else ROOT / "core/lib/src" / recipe_relative
            )
            target.write_bytes(
                current.read_bytes() if current.exists() else source.read_bytes()
            )
        elif target.exists() and target.read_bytes() != source.read_bytes():
            raise PromotionError(f"spec consolidation conflict: {relative}")
        elif not target.exists():
            target.write_bytes(source.read_bytes())
    shutil.rmtree(old_root)


def patch_framework() -> None:
    path = "core/lib/src/code/framework.hal"
    add_require(path, "[code.framework.source :as source]")
    add_require(path, "[code.framework.navigation :as navigation]")
    append_once(
        path,
        "(def transform-code source/transform-code)",
        """(def transform-function source/transform-function)
(def normalize-edit source/normalize-edit)
(def transform-code source/transform-code)
(def grep-code source/grep-code)
(def replacement source/replacement)
(def checked-edits source/checked-edits)
(def apply-edits source/apply-edits)
(def replacements source/replacements)
(def replace-all source/replace-all)
(def grep-replace source/grep-replace)
(def apply-edit-record source/apply-edit-record)""",
    )


def patch_manage_unit() -> None:
    path = "core/lib/src/code/manage/unit.hal"
    add_require(path, "[code.framework.source :as source]")
    for name, replacement in {
        "transform-function": "(def transform-function source/transform-function)",
        "transform-code": "(def transform-code source/transform-code)",
        "grep": "(def grep source/grep-code)",
        "replacements": "(def replacements source/replacements)",
        "replace-all": "(def replace-all source/replace-all)",
        "grep-replace": "(def grep-replace source/grep-replace)",
    }.items():
        replace_form(path, name, replacement)


def patch_manage_facade() -> None:
    path = "core/lib/src/code/manage.hal"
    source = read(path)
    source = re.sub(
        r"^\s*\[code\.migrate\.project\s+:as\s+translate\]\s*\n",
        "", source, flags=re.MULTILINE,
    )
    write(path, source)
    try:
        replace_form(path, "clojure-to-hal", None)
    except PromotionError as error:
        if "found 0" not in str(error):
            raise
    write(
        path,
        "\n".join(
            line for line in read(path).splitlines()
            if "clojure-to-hal" not in line
        ) + "\n",
    )
    add_require(path, "[code.manage.migrate :as migrate]")
    append_once(
        path,
        "(def foundation-audit migrate/audit)",
        """(def foundation-audit migrate/audit)
(def foundation-write-audit migrate/write-audit)""",
    )


def patch_clojure_migration() -> None:
    path = "core/lib/src/code/migrate/clojure.hal"
    add_require(path, "[code.framework :as framework]")
    source = read(path)
    matches = [
        (start, end, form) for start, end, form in top_level_forms(source)
        if form_name(form) == "translate-source-unit"
    ]
    if len(matches) != 1:
        raise PromotionError(
            f"{path}: expected one translate-source-unit, found {len(matches)}"
        )
    start, end, form = matches[0]
    renamed = form.replace(
        "(defn translate-source-unit", "(defn migration-result", 1,
    )
    wrapper = """(defn translate-source-unit
  \"Normalises one migration through the shared code.framework substrate.\"
  ([unit]
   (translate-source-unit unit {}))
  ([unit options]
   (let [prepared (source-unit unit options)
         result (migration-result prepared options)]
     (framework/transform-code
      {:source/path (:source/path prepared)
       :source/text (:source/text prepared)
       :target/path (:target/path prepared)
       :target/text (:target/text prepared)}
      (fn [_] result)))))"""
    write(path, source[:start] + renamed + "\n\n" + wrapper + "\n" + source[end:])


def patch_project() -> None:
    path = "core/lib/src/code/migrate/project.hal"
    add_require(path, "[code.migrate.report :as migrate-report]")
    add_require(path, "[code.migrate.write :as migrate-write]")
    wrappers = {
        "record-error?": "(def record-error? migrate-report/record-error?)",
        "batch-records": "(def batch-records migrate-report/batch-records)",
        "task-summary": """(defn task-summary
  [batch]
  (migrate-report/task-summary task-id batch))""",
        "item-diagnostics": "(def item-diagnostics migrate-report/item-diagnostics)",
        "item-rules": "(def item-rules migrate-report/item-rules)",
        "count-by": "(def count-by migrate-report/count-by)",
        "sorted-unique": "(def sorted-unique migrate-report/sorted-unique)",
        "namespaces-by": "(def namespaces-by migrate-report/namespaces-by)",
        "rule-counts": "(def rule-counts migrate-report/rule-counts)",
        "safety-counts": "(def safety-counts migrate-report/safety-counts)",
        "cycle-report": "(def cycle-report migrate-report/cycle-report)",
        "item-blocked?": "(def item-blocked? migrate-report/item-blocked?)",
        "next-unblocked": "(def next-unblocked migrate-report/next-unblocked)",
        "diagnostic-errors": "(def diagnostic-errors migrate-report/diagnostic-errors)",
        "diagnostic-warnings": "(def diagnostic-warnings migrate-report/diagnostic-warnings)",
        "plan-summary": """(defn plan-summary
  [units records items edits diagnostics]
  (migrate-report/plan-summary
   task-id units records items edits diagnostics))""",
        "report": "(def report migrate-report/report)",
        "blocking-diagnostic?": "(def blocking-diagnostic? migrate-write/blocking-diagnostic?)",
        "canonical-under-root?": "(def canonical-under-root? migrate-write/canonical-under-root?)",
        "required-capability": "(def required-capability migrate-write/required-capability)",
        "duplicate-targets": "(def duplicate-targets migrate-write/duplicate-targets)",
        "intent-errors": "(def intent-errors migrate-write/intent-errors)",
        "write-intent": "(def write-intent migrate-write/write-intent)",
        "result-edit?": "(def result-edit? migrate-write/result-edit?)",
        "write-intents": "(def write-intents migrate-write/write-intents)",
        "validate-write-plan": "(def validate-write-plan migrate-write/validate-write-plan)",
        "apply-plan": "(def apply-plan migrate-write/apply-plan)",
    }
    for name, replacement in wrappers.items():
        try:
            replace_form(path, name, replacement)
        except PromotionError as error:
            if "found 0" not in str(error):
                raise
            if name in {"item-blocked?"}:
                continue
            raise


def patch_bang_planner() -> None:
    path = "core/lib/src/code/migrate/bang.hal"
    replace_form(
        path,
        "+embedded-rust-snapshot-ledger+",
        """(def +embedded-rust-snapshot-ledger+
  [{:source/path \"lib/src/code/framework/source.hal\"
    :target/path \"rust/hal-src/code/framework/source.hal\"
    :migration/action :regenerate :migration/owner :code.migrate}
   {:source/path \"lib/src/code/framework/navigation.hal\"
    :target/path \"rust/hal-src/code/framework/navigation.hal\"
    :migration/action :regenerate :migration/owner :code.migrate}
   {:source/path \"lib/src/code/migrate/project.hal\"
    :target/path \"rust/hal-src/code/migrate/project.hal\"
    :migration/action :regenerate :migration/owner :code.migrate}
   {:source/path \"lib/src/code/migrate/clojure.hal\"
    :target/path \"rust/hal-src/code/migrate/clojure.hal\"
    :migration/action :regenerate :migration/owner :code.migrate}
   {:source/path \"lib/src/code/migrate/rule.hal\"
    :target/path \"rust/hal-src/code/migrate/rule.hal\"
    :migration/action :regenerate :migration/owner :code.migrate}
   {:source/path \"lib/src/code/migrate/rules.hal\"
    :target/path \"rust/hal-src/code/migrate/rules.hal\"
    :migration/action :regenerate :migration/owner :code.migrate}
   {:source/path \"lib/src/code/migrate/profile.hal\"
    :target/path \"rust/hal-src/code/migrate/profile.hal\"
    :migration/action :regenerate :migration/owner :code.migrate}
   {:source/path \"lib/src/code/migrate/git.hal\"
    :target/path \"rust/hal-src/code/migrate/git.hal\"
    :migration/action :regenerate :migration/owner :code.migrate}
   {:source/path \"lib/src/code/migrate/corpus.hal\"
    :target/path \"rust/hal-src/code/migrate/corpus.hal\"
    :migration/action :regenerate :migration/owner :code.migrate}
   {:source/path \"lib/src/code/migrate/report.hal\"
    :target/path \"rust/hal-src/code/migrate/report.hal\"
    :migration/action :regenerate :migration/owner :code.migrate}
   {:source/path \"lib/src/code/migrate/write.hal\"
    :target/path \"rust/hal-src/code/migrate/write.hal\"
    :migration/action :regenerate :migration/owner :code.migrate}
   {:source/path \"lib/src/code/manage/migrate.hal\"
    :target/path \"rust/hal-src/code/manage/migrate.hal\"
    :migration/action :regenerate :migration/owner :code.migrate}
   {:source/path \"lib/src/std/dom/common.hal\"
    :target/path \"rust/hal-src/std/dom/common.hal\"
    :migration/action :regenerate :migration/owner :code.migrate}])""",
    )
    replace_form(
        path,
        "plan-unit",
        """(defn plan-unit
  [unit ledger-index]
  (let [edits (unit-rename-edits unit ledger-index)]
    (framework/transform-code
     unit
     (fn [source]
       {:after (structure/apply-edits source edits)
        :edits edits
        :rules [:bang-vars/rename]
        :diagnostics []}))))""",
    )
    replace_form(
        path,
        "embedded-snapshot-unit",
        """(defn embedded-snapshot-unit
  [current-project record]
  (let [root (:project/root current-project)
        source (project/read-text (File/join root (:source/path record)))
        target (project/read-text (File/join root (:target/path record)))]
    (framework/transform-code
     {:source/path (:source/path record)
      :source/text source
      :target/path (:target/path record)
      :target/text target}
     (fn [_]
       {:after source
        :rules [:bang-vars/embedded-rust-snapshot]
        :diagnostics []
        :provenance
        {:migration/action (:migration/action record)
         :migration/owner (:migration/owner record)}}))))""",
    )


def patch_migrate_facade() -> None:
    path = "core/lib/src/code/migrate.hal"
    for spec in [
        "[code.migrate.profile :as migrate-profile]",
        "[code.migrate.git :as migrate-git]",
        "[code.migrate.corpus :as migrate-corpus]",
        "[code.migrate.project :as migrate-project]",
        "[code.migrate.rule :as migrate-rule]",
        "[code.migrate.write :as migrate-write]",
    ]:
        add_require(path, spec)
    append_once(
        path,
        "(def foundation-profile migrate-profile/+foundation-baa75a+)",
        """(def foundation-profile migrate-profile/+foundation-baa75a+)
(def foundation-corpus migrate-corpus/corpus-document)
(def foundation-corpus-source migrate-corpus/corpus-source)
(def foundation-report-source migrate-corpus/report-source)
(def foundation-plan migrate-project/plan)
(def foundation-run migrate-project/run)
(def foundation-validate-write-plan migrate-write/validate-write-plan)
(def foundation-apply-plan migrate-write/apply-plan)
(def foundation-rules migrate-rule/+ruleset+)
(def foundation-git-snapshot migrate-git/snapshot)""",
    )


def sync_rust_mirrors() -> None:
    relative_paths = [
        "code/framework.hal", "code/framework/source.hal",
        "code/framework/navigation.hal", "code/manage.hal",
        "code/manage/unit.hal", "code/manage/migrate.hal", "code/migrate.hal",
        "code/migrate/bang.hal", "code/migrate/project.hal",
        "code/migrate/clojure.hal", "code/migrate/rule.hal",
        "code/migrate/rules.hal", "code/migrate/profile.hal",
        "code/migrate/git.hal", "code/migrate/corpus.hal",
        "code/migrate/report.hal", "code/migrate/write.hal",
    ]
    for relative in relative_paths:
        source = ROOT / "core/lib/src" / relative
        target = ROOT / "core/rust/hal-src" / relative
        if not source.exists():
            raise PromotionError(f"missing canonical source for Rust mirror: {relative}")
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(source.read_bytes())


def remove_old_corpus() -> dict:
    routes = ROOT / "core/spec/clj-hal-routes.json"
    legacy = json.loads(routes.read_text(encoding="utf-8")) if routes.exists() else {}
    for path in [
        "core/spec/clj-hal-routes.json", "core/spec/clj-hal-corpus.json",
        "scripts/runtime/clj_hal_corpus.py", "scripts/runtime/clj_hal_corpus_test.py",
        ".github/workflows/clj-hal-corpus.yml",
    ]:
        remove(path)
    return legacy


NS_PATTERN = re.compile(r"\(\s*ns\+?\s+([^\s()\[\]{}]+)")
REQUIRE_PATTERN = re.compile(r"\(\s*:(?:require|use)\s+([\s\S]*?)\)")
SYMBOL_DEP_PATTERN = re.compile(r"(?<![\w./-])([A-Za-z0-9_.\-]+)(?=\s|\))")
QUALIFIED_PATTERN = re.compile(
    r"(?<![A-Za-z0-9_.\-/])([A-Za-z][A-Za-z0-9_.\-]*)/"
    r"([A-Za-z0-9_.*+!?\-:<>=]+)"
)
IMPORT_PATTERN = re.compile(r"\(\s*:import\s+([\s\S]*?)\)")
JAVA_CLASS_PATTERN = re.compile(r"\b(?:java|javax|clojure)\.[A-Za-z0-9_.$\-]+")


def strip_comments_and_strings(source: str) -> str:
    output: list[str] = []
    in_string = escaped = in_comment = False
    for char in source:
        if in_comment:
            if char == "\n":
                in_comment = False
                output.append("\n")
            else:
                output.append(" ")
        elif in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            output.append("\n" if char == "\n" else " ")
        elif char == ";":
            in_comment = True
            output.append(" ")
        elif char == '"':
            in_string = True
            output.append(" ")
        else:
            output.append(char)
    return "".join(output)


def namespace_surface(source: str) -> dict:
    clean = strip_comments_and_strings(source)
    match = NS_PATTERN.search(clean)
    namespace = match.group(1) if match else None
    dependencies: set[str] = set()
    aliases: dict[str, str] = {}
    referred: list[dict] = []
    for clause in REQUIRE_PATTERN.findall(clean):
        for entry in re.finditer(r"\[([^\[\]]+)\]", clause):
            tokens = re.findall(r"[^\s,]+", entry.group(1))
            if not tokens or tokens[0].startswith(":"):
                continue
            dependency = tokens[0]
            dependencies.add(dependency)
            if ":as" in tokens:
                index = tokens.index(":as")
                if index + 1 < len(tokens):
                    aliases[tokens[index + 1]] = dependency
            if ":refer" in tokens:
                referred.append({"namespace": dependency, "entry": entry.group(0)})
        residual = re.sub(r"\[[^\[\]]+\]", " ", clause)
        for dependency in SYMBOL_DEP_PATTERN.findall(residual):
            if "." in dependency and not dependency.startswith(":"):
                dependencies.add(dependency)
    imports = sorted(set(JAVA_CLASS_PATTERN.findall(
        "\n".join(IMPORT_PATTERN.findall(clean))
    )))
    qualified = []
    for owner, symbol in QUALIFIED_PATTERN.findall(clean):
        route = aliases.get(owner, owner)
        qualified.append({"owner": owner, "namespace": route, "symbol": symbol})
        if "." in route:
            dependencies.add(route)
    public_symbols = set()
    for head, name in re.findall(
        r"\(\s*(defn-?|defmacro|defmulti|defstruct|defmutable|def)\s+"
        r"(?:\^\S+\s+)*([^\s()\[\]{}]+)", clean,
    ):
        if head != "defn-" and not name.endswith("-"):
            public_symbols.add(name)
    return {
        "namespace": namespace,
        "dependencies": sorted(dependencies),
        "imports": imports,
        "aliases": dict(sorted(aliases.items())),
        "referred": referred,
        "qualified_references": sorted(
            qualified,
            key=lambda value: (value["namespace"], value["symbol"], value["owner"]),
        ),
        "public_symbols": sorted(public_symbols),
    }


def foundation_paths() -> list[str]:
    output = git_object("ls-tree", "-r", "--name-only", FOUNDATION_REVISION)
    return sorted(line for line in output.splitlines() if line)


def foundation_blob(path: str) -> tuple[str, str]:
    blob = git_object("rev-parse", f"{FOUNDATION_REVISION}:{path}").strip()
    return blob, git_object("cat-file", "blob", blob)


def is_source_path(path: str) -> bool:
    return path.endswith(".clj") and path.split("/", 1)[0] in {
        "src", "src-java", "src-lang",
    }


def paired_test_path(path: str) -> str | None:
    root, _, suffix = path.partition("/")
    test_root = {"src": "test", "src-java": "test-java", "src-lang": "test-lang"}.get(root)
    if not test_root or not suffix.endswith(".clj"):
        return None
    return f"{test_root}/{suffix[:-4]}_test.clj"


def git_blob_id(source: str) -> str:
    data = source.encode("utf-8")
    return hashlib.sha1(
        b"blob " + str(len(data)).encode("ascii") + b"\0" + data
    ).hexdigest()


def hara_surfaces() -> list[dict]:
    output = []
    for root in (ROOT / "core/lib/src", ROOT / "core/lib/test"):
        for path in sorted(root.rglob("*.hal")):
            source = path.read_text(encoding="utf-8")
            surface = namespace_surface(source)
            if surface["namespace"]:
                output.append({
                    "path": path.relative_to(ROOT).as_posix(),
                    "blob": git_blob_id(source),
                    **surface,
                })
    return output


def namespace_leaf(namespace: str | None) -> str:
    return (namespace or "").split(".")[-1]


def candidate_matches(entry: dict, targets: list[dict]) -> list[dict]:
    output = []
    source_symbols = set(entry["public_symbols"])
    for target in targets:
        overlap = sorted(source_symbols.intersection(target["public_symbols"]))
        if entry["namespace"] == target["namespace"]:
            state = "exact"
        elif namespace_leaf(entry["namespace"]) == namespace_leaf(target["namespace"]):
            state = "moved"
        elif overlap:
            state = "similar"
        else:
            continue
        output.append({
            "candidate/state": state,
            "target/path": target["path"],
            "target/blob": target["blob"],
            "target/namespace": target["namespace"],
            "matching/public-symbols": overlap,
        })
    return sorted(output, key=lambda value: (value["candidate/state"], value["target/path"]))


def symbol_routes(entry: dict, candidates: list[dict]) -> list[dict]:
    output = []
    for symbol in entry["public_symbols"]:
        targets = [
            {
                "target/path": candidate["target/path"],
                "target/blob": candidate["target/blob"],
                "target/namespace": candidate["target/namespace"],
                "target/symbol": symbol,
            }
            for candidate in candidates
            if symbol in candidate["matching/public-symbols"]
        ]
        output.append({
            "source/symbol": symbol,
            "candidate/targets": targets,
            "review/disposition": "pending",
            "review/evidence": [],
        })
    return output


def legacy_routes_index(routes: dict) -> dict[str, dict]:
    return {
        entry.get("namespace"): entry
        for entry in routes.get("namespaces", []) if entry.get("namespace")
    }


def build_entries(legacy_routes: dict) -> list[dict]:
    paths = foundation_paths()
    path_index = set(paths)
    targets = hara_surfaces()
    legacy_index = legacy_routes_index(legacy_routes)
    output = []
    for source_path in filter(is_source_path, paths):
        blob, source = foundation_blob(source_path)
        surface = namespace_surface(source)
        if not surface["namespace"]:
            continue
        candidates = candidate_matches(surface, targets)
        entry = {
            "namespace": surface["namespace"],
            "source/path": source_path,
            "source/blob": blob,
            "dependencies": surface["dependencies"],
            "imports": surface["imports"],
            "aliases": surface["aliases"],
            "referred-symbols": surface["referred"],
            "qualified-references": surface["qualified_references"],
            "public-symbols": surface["public_symbols"],
            "candidate/matches": candidates,
            "symbol/routes": symbol_routes(surface, candidates),
            "review/disposition": "pending",
            "review/evidence": [],
            "dry-run/status": "not-planned",
        }
        test_path = paired_test_path(source_path)
        if test_path and test_path in path_index:
            test_blob, test_source = foundation_blob(test_path)
            test_surface = namespace_surface(test_source)
            entry.update({
                "test/path": test_path,
                "test/blob": test_blob,
                "test/namespace": test_surface["namespace"],
                "test/dependencies": test_surface["dependencies"],
                "test/public-symbols": test_surface["public_symbols"],
            })
        legacy = legacy_index.get(entry["namespace"])
        if legacy:
            entry["candidate/legacy-evidence"] = {
                "foundation/revision": LEGACY_REVISION,
                "route": legacy,
                "review/status": "requires-revalidation",
            }
        output.append(entry)
    return output


def dependency_graph(entries: list[dict]) -> dict[str, list[str]]:
    names = {entry["namespace"] for entry in entries}
    return {
        entry["namespace"]: sorted(
            dependency for dependency in entry["dependencies"]
            if dependency in names
        )
        for entry in entries
    }


def tarjan(graph: dict[str, list[str]]) -> list[list[str]]:
    next_index = 0
    indexes: dict[str, int] = {}
    lows: dict[str, int] = {}
    stack: list[str] = []
    active: set[str] = set()
    output: list[list[str]] = []

    def visit(node: str) -> None:
        nonlocal next_index
        indexes[node] = next_index
        lows[node] = next_index
        next_index += 1
        stack.append(node)
        active.add(node)
        for dependency in graph[node]:
            if dependency not in indexes:
                visit(dependency)
                lows[node] = min(lows[node], lows[dependency])
            elif dependency in active:
                lows[node] = min(lows[node], indexes[dependency])
        if lows[node] == indexes[node]:
            group = []
            while True:
                member = stack.pop()
                active.remove(member)
                group.append(member)
                if member == node:
                    break
            output.append(sorted(group))

    for node in sorted(graph):
        if node not in indexes:
            visit(node)
    return sorted(output, key=lambda group: "|".join(group))


def rank_entries(entries: list[dict]) -> list[dict]:
    graph = dependency_graph(entries)
    components = tarjan(graph)
    owners = {
        namespace: number
        for number, component in enumerate(components)
        for namespace in component
    }
    requirements: dict[int, set[int]] = defaultdict(set)
    for namespace, dependencies in graph.items():
        owner = owners[namespace]
        for dependency in dependencies:
            dependency_owner = owners[dependency]
            if owner != dependency_owner:
                requirements[owner].add(dependency_owner)
    memo: dict[int, int] = {}

    def rank(number: int) -> int:
        if number not in memo:
            required = requirements[number]
            memo[number] = 0 if not required else 1 + max(rank(value) for value in required)
        return memo[number]

    by_namespace = {entry["namespace"]: entry for entry in entries}
    output = []
    for namespace in sorted(graph):
        entry = by_namespace[namespace]
        owner = owners[namespace]
        component = components[owner]
        entry["dependency/rank"] = rank(owner)
        entry["dependency/component"] = component
        entry["dependency/cycle"] = len(component) > 1 or namespace in graph[namespace]
        output.append(entry)
    return sorted(output, key=lambda value: (value["dependency/rank"], value["namespace"]))


def external_routes(entries: list[dict]) -> list[dict]:
    names = {entry["namespace"] for entry in entries}
    output = []
    for entry in entries:
        for dependency in entry["dependencies"]:
            if dependency not in names:
                output.append({
                    "external/namespace": dependency,
                    "dependency/rank": -1,
                    "source/namespace": entry["namespace"],
                    "source/path": entry["source/path"],
                    "source/blob": entry["source/blob"],
                    "candidate/matches": [],
                    "review/disposition": "pending",
                    "review/evidence": [],
                })
    return sorted(output, key=lambda value: (
        value["external/namespace"], value["source/namespace"], value["source/path"],
    ))


def edn(value: object, key: str | None = None) -> str:
    if value is None:
        return "nil"
    if value is True:
        return "true"
    if value is False:
        return "false"
    if isinstance(value, str):
        if key in {
            "candidate/state", "review/disposition", "dry-run/status",
            "document/type", "profile/id", "review/status",
        }:
            return ":" + value
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, (int, float)):
        return str(value)
    if isinstance(value, list):
        return "[" + " ".join(edn(item) for item in value) + "]"
    if isinstance(value, dict):
        return "{" + " ".join(
            f":{current} {edn(value[current], current)}" for current in sorted(value)
        ) + "}"
    raise TypeError(f"unsupported EDN value: {type(value)!r}")


def build_corpus(legacy_routes: dict) -> dict:
    entries = rank_entries(build_entries(legacy_routes))
    pending = [entry for entry in entries if entry["review/disposition"] == "pending"]
    next_review = []
    if pending:
        rank = min(entry["dependency/rank"] for entry in pending)
        next_review = sorted(
            entry["namespace"] for entry in pending
            if entry["dependency/rank"] == rank
        )
    return {
        "document/type": "code-migrate-corpus",
        "document/version": 1,
        "profile/id": "foundation-baa75a",
        "foundation/repository": "zcaudate-xyz/foundation-base",
        "foundation/revision": FOUNDATION_REVISION,
        "foundation/tree": FOUNDATION_TREE,
        "external/rank": -1,
        "foundation/rank-base": 0,
        "external/routes": external_routes(entries),
        "namespaces": entries,
        "next/review": next_review,
        "next/unblocked-migration": [],
    }


def markdown_report(corpus: dict) -> str:
    lines = [
        "# Foundation migration audit: `foundation-baa75a`", "",
        "- Foundation repository: `zcaudate-xyz/foundation-base`",
        f"- Pinned revision: `{FOUNDATION_REVISION}`",
        f"- Pinned tree: `{FOUNDATION_TREE}`",
        f"- Foundation namespaces: {len(corpus['namespaces'])}",
        f"- External dependency routes: {len(corpus['external/routes'])}",
        "- Reviewed namespaces: 0",
        f"- Pending namespaces: {len(corpus['namespaces'])}", "",
        "## Next review", "",
    ]
    lines.extend(f"- `{namespace}`" for namespace in corpus["next/review"])
    lines.extend([
        "", "## Next unblocked migration", "",
        "No reviewed dry-run is currently marked ready.", "",
        "## Evidence policy", "",
        "Candidate matches are discovery evidence only. A migration remains pending "
        "until source and paired-test blobs, Hara target paths, rule evidence, and "
        "idempotent dry-run disposition are reviewed against this exact profile.", "",
    ])
    return "\n".join(lines)


def write_corpus(legacy_routes: dict) -> None:
    commit = git_object("rev-parse", f"{FOUNDATION_REVISION}^{{commit}}").strip()
    tree = git_object("rev-parse", f"{FOUNDATION_REVISION}^{{tree}}").strip()
    if commit != FOUNDATION_REVISION or tree != FOUNDATION_TREE:
        raise PromotionError(
            f"Foundation object mismatch: commit={commit}, tree={tree}"
        )
    document = build_corpus(legacy_routes)
    corpus_source = edn(document) + "\n"
    report_source = markdown_report(document)
    write("core/spec/code-migrate/foundation-baa75a.edn", corpus_source)
    write("core/spec/code-migrate/foundation-baa75a.md", report_source)
    manifest = (
        f"{hashlib.sha256(corpus_source.encode()).hexdigest()}  foundation-baa75a.edn\n"
        f"{hashlib.sha256(report_source.encode()).hexdigest()}  foundation-baa75a.md\n"
    )
    write("core/spec/code-migrate/foundation-baa75a.sha256", manifest)


def patch_target_snapshot() -> None:
    path = ROOT / "scripts/runtime/foundation_target_snapshot.py"
    if path.exists():
        source = path.read_text(encoding="utf-8")
        source = re.sub(r"^DEFAULT_(?:ROUTES|CORPUS)\s*=.*\n", "", source, flags=re.MULTILINE)
        source = re.sub(
            r"\n\ndef corpus_paths\(.*?\n\ndef changed_paths",
            "\n\ndef changed_paths", source, flags=re.DOTALL,
        )
        source = re.sub(
            r"\n\ndef verify_corpus_snapshot\(.*?\n\ndef verify\(",
            "\n\ndef verify(", source, flags=re.DOTALL,
        )
        source = source.replace("    routes: dict,\n    corpus: dict,\n", "")
        source = re.sub(
            r"\s*if ledger in \{\"all\", \"corpus\"\}:\n"
            r"\s*result\[\"corpus\"\].*?\n",
            "", source,
        )
        source = re.sub(
            r'parser\.add_argument\("--ledger".*?\)',
            'parser.add_argument("--ledger", choices=("all", "script"), default="all")',
            source,
        )
        source = re.sub(r'^\s*parser\.add_argument\("--(?:routes|corpus)".*\n', "", source, flags=re.MULTILINE)
        source = re.sub(r"\s*load_json\(args\.(?:routes|corpus)\),\n", "", source)
        path.write_text(source, encoding="utf-8")

    test = ROOT / "scripts/runtime/foundation_target_snapshot_test.py"
    if test.exists():
        test.write_text(
            """#!/usr/bin/env python3
import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "foundation_target_snapshot",
    ROOT / "scripts/runtime/foundation_target_snapshot.py",
)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


class FoundationTargetSnapshotTest(unittest.TestCase):
    def test_script_inventory_paths_are_non_empty_and_contained(self):
        inventory = {"namespaces": [{"target": {
            "blob": "a" * 40,
            "path": "core/lib/src/demo.hal",
        }}]}
        self.assertEqual(
            MODULE.script_paths(inventory),
            ["core/lib/src/demo.hal"],
        )

    def test_evidence_path_rejects_parent_escape(self):
        with self.assertRaises(MODULE.SnapshotError):
            MODULE.evidence_path("../outside")


if __name__ == "__main__":
    unittest.main()
""",
            encoding="utf-8",
        )


def write_permanent_workflow() -> None:
    workflow = """name: Foundation migration audit

on:
  push:
    branches: [main, testing]
    paths:
      - 'core/lib/src/code/framework/**'
      - 'core/lib/src/code/manage/**'
      - 'core/lib/src/code/migrate/**'
      - 'core/lib/test/code/framework*'
      - 'core/lib/test/code/migrate*'
      - 'core/spec/code-migrate/**'
      - 'core/rust/hal-src/code/**'
      - 'scripts/runtime/foundation_target_snapshot*'
      - '.github/workflows/foundation-migrate.yml'
  pull_request:
    branches: [main, testing]
    paths:
      - 'core/lib/src/code/framework/**'
      - 'core/lib/src/code/manage/**'
      - 'core/lib/src/code/migrate/**'
      - 'core/lib/test/code/framework*'
      - 'core/lib/test/code/migrate*'
      - 'core/spec/code-migrate/**'
      - 'core/rust/hal-src/code/**'
      - 'scripts/runtime/foundation_target_snapshot*'
      - '.github/workflows/foundation-migrate.yml'
  workflow_dispatch:

permissions:
  contents: read

concurrency:
  group: foundation-migrate-${{ github.event.pull_request.number || github.ref }}
  cancel-in-progress: true

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Fetch the pinned Foundation object into a bare repository
        run: |
          set -euo pipefail
          git init --bare .foundation.git
          git --git-dir=.foundation.git remote add origin \
            https://github.com/zcaudate-xyz/foundation-base.git
          git --git-dir=.foundation.git fetch --depth=1 origin \
            baa75aabd6a879753d7d5cb07271b1448271e7cb
          test "$(git --git-dir=.foundation.git rev-parse \
            'baa75aabd6a879753d7d5cb07271b1448271e7cb^{commit}')" = \
            baa75aabd6a879753d7d5cb07271b1448271e7cb
          test "$(git --git-dir=.foundation.git rev-parse \
            'baa75aabd6a879753d7d5cb07271b1448271e7cb^{tree}')" = \
            26d494f60c4970df56eba8ac40f92affeee4e159
          git --git-dir=.foundation.git show-ref > target-foundation-before.refs
          git --git-dir=.foundation.git count-objects -v > target-foundation-before.objects

      - name: Enforce the hard cut and deterministic evidence bytes
        run: |
          set -euo pipefail
          ! rg -n 'code\\.translate|code-translate|clj[-_]hal[-_]corpus' \
            core scripts .github
          (
            cd core/spec/code-migrate
            sha256sum --check foundation-baa75a.sha256
          )
          test "$(grep -o ':foundation/revision \"[0-9a-f]*\"' \
            core/spec/code-migrate/foundation-baa75a.edn | head -1)" = \
            ':foundation/revision "baa75aabd6a879753d7d5cb07271b1448271e7cb"'

      - name: Verify Foundation status was not mutated
        run: |
          set -euo pipefail
          git --git-dir=.foundation.git show-ref > target-foundation-after.refs
          git --git-dir=.foundation.git count-objects -v > target-foundation-after.objects
          cmp target-foundation-before.refs target-foundation-after.refs
          cmp target-foundation-before.objects target-foundation-after.objects
          test -z "$(git --git-dir=.foundation.git for-each-ref refs/heads)"
          test ! -e .foundation.git/index
          test ! -d .foundation-reference

      - name: Set up JDK 21
        uses: actions/setup-java@v4
        with:
          java-version: '21'
          distribution: temurin
          cache: maven
          cache-dependency-path: core/java/pom.xml

      - name: Build a fresh JVM runtime
        run: mvn -B -f core/java/pom.xml -Ptruffle -DskipTests package

      - name: Set up Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build and test a fresh Rust runtime
        run: |
          cargo build --manifest-path core/rust/Cargo.toml --bin hara-test
          cargo test --manifest-path core/rust/Cargo.toml
          cargo test --manifest-path core/rust/raw/Cargo.toml

      - name: Run focused framework, manage, and migrate tests
        run: |
          ./scripts/runtime/run-lib-tests \
            core/lib/test/code/framework_source_test.hal \
            core/lib/test/code/framework_test.hal \
            core/lib/test/code/manage_transform_test.hal \
            core/lib/test/code/migrate_git_test.hal \
            core/lib/test/code/migrate_corpus_test.hal \
            core/lib/test/code/migrate_rules_test.hal \
            core/lib/test/code/migrate_clojure_test.hal \
            core/lib/test/code/migrate_project_test.hal

      - name: Run complete Hara library tests
        run: ./scripts/runtime/run-lib-tests

      - name: Verify Rust source mirrors
        run: |
          set -euo pipefail
          while IFS= read -r source; do
            relative="${source#core/lib/src/}"
            mirror="core/rust/hal-src/$relative"
            test -f "$mirror"
            cmp "$source" "$mirror"
          done < <(find core/lib/src/code/framework core/lib/src/code/manage \
                         core/lib/src/code/migrate \
                    -name '*.hal' -type f | sort)
"""
    write(".github/workflows/foundation-migrate.yml", workflow)


def static_assertions() -> None:
    prohibited = re.compile(r"code\.translate|code-translate|clj[-_]hal[-_]corpus")
    findings = []
    for path in text_files():
        if path in TRANSPORT_FILES:
            continue
        source = path.read_text(encoding="utf-8")
        if prohibited.search(source) or prohibited.search(path.as_posix()):
            findings.append(path.relative_to(ROOT).as_posix())
    if findings:
        raise PromotionError("hard-cut references remain:\n" + "\n".join(findings[:100]))
    for old in SOURCE_MOVES:
        if (ROOT / old).exists():
            raise PromotionError(f"old source path remains: {old}")
    if (ROOT / "core/spec/code-translate").exists():
        raise PromotionError("divergent code-translate spec tree remains")


def delete_transport() -> None:
    for path in TRANSPORT_FILES:
        if path.exists():
            path.unlink()


def main() -> int:
    if not FOUNDATION_GIT.is_dir():
        raise PromotionError(f"bare Foundation repository not found: {FOUNDATION_GIT}")
    move_sources_and_tests()
    consolidate_recipes()
    global_replace()
    patch_framework()
    patch_manage_unit()
    patch_manage_facade()
    patch_clojure_migration()
    patch_project()
    patch_bang_planner()
    patch_migrate_facade()
    legacy_routes = remove_old_corpus()
    patch_target_snapshot()
    sync_rust_mirrors()
    write_corpus(legacy_routes)
    write_permanent_workflow()
    global_replace()
    sync_rust_mirrors()
    static_assertions()
    delete_transport()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
