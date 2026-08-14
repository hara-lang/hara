#!/usr/bin/env python3
"""Prepare the generated #403 application script for duplicated workflow paths."""

from pathlib import Path

path = Path("scripts/runtime/apply-code-vm-semantic-boundaries.py")
source = path.read_text()
old = '''    count = source.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one replacement, found {count}")
    target.write_text(source.replace(old, new, 1))
'''
new = '''    count = source.count(old)
    duplicated_workflow_path = (
        path == ".github/workflows/code-vm-live-interpreter.yml" and count == 2
    )
    if count != 1 and not duplicated_workflow_path:
        raise SystemExit(f"{path}: expected one replacement, found {count}")
    target.write_text(source.replace(old, new, 1))
'''
if source.count(old) != 1:
    raise SystemExit("apply-script replacement helper changed unexpectedly")
path.write_text(source.replace(old, new, 1))
