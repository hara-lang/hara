#!/usr/bin/env python3
"""Close the final code.migrate.corpus form, then remove this transport."""

from pathlib import Path

path = Path("core/lib/src/code/migrate/corpus.hal")
self_path = Path(".github/agent-patches/866-close-corpus.py")
source = path.read_text(encoding="utf-8")

expected = "         (:next/unblocked-migration document))))))\n"
if not source.endswith(expected):
    raise SystemExit("unexpected code.migrate.corpus tail")

path.write_text(source + ")\n", encoding="utf-8")
self_path.unlink()
