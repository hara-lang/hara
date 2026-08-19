#!/usr/bin/env python3
"""Canonicalise the migrated task identifiers for PR #866, then delete self."""

from __future__ import annotations

from pathlib import Path

PATCHES = {
    Path("core/lib/src/code/migrate/clojure.hal"): [
        (
            "(def task-id :code.migrate/clojure-to-hal)",
            "(def task-id :code.migrate/clojure)",
        ),
    ],
    Path("core/lib/src/code/migrate/project.hal"): [
        (
            "(def task-id :code.migrate/clojure-to-hal-project)",
            "(def task-id :code.migrate/project)",
        ),
        ("\n(def clojure-to-hal migration-work)\n", "\n"),
    ],
    Path("core/lib/test/code/migrate_clojure_test.hal"): [
        (":code.migrate/clojure-to-hal", ":code.migrate/clojure"),
    ],
    Path("core/lib/test/code/migrate_project_test.hal"): [
        (
            ":code.migrate/clojure-to-hal-project",
            ":code.migrate/project",
        ),
    ],
    Path("core/spec/std/private-definition-namespace-survey.tsv"): [
        (
            "code.manage|code.translate|code.translate.clojure|std.foundation.pretty",
            "code.manage|code.migrate.clojure|code.migrate.project|std.foundation.pretty",
        ),
    ],
}

SELF = Path(".github/agent-patches/866-canonical-task-names.py")


def patch_file(path: Path, replacements: list[tuple[str, str]]) -> None:
    source = path.read_text(encoding="utf-8")
    output = source
    for old, new in replacements:
        occurrences = output.count(old)
        if occurrences != 1 and not (
            path.name == "migrate_clojure_test.hal" and occurrences == 2
        ):
            raise RuntimeError(
                f"{path}: expected canonical replacement count, "
                f"found {occurrences} for {old!r}"
            )
        output = output.replace(old, new)
    if output == source:
        raise RuntimeError(f"{path}: patch made no change")
    path.write_text(output.rstrip() + "\n", encoding="utf-8")


def main() -> int:
    for path, replacements in PATCHES.items():
        patch_file(path, replacements)
    SELF.unlink()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
