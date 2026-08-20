#!/usr/bin/env python3
import base64
import gzip
from pathlib import Path

parts = Path(__file__).resolve().parent / ".apply-849"
encoded = "".join(
    (parts / f"part{index:02d}").read_text().strip()
    for index in range(6)
)
source = gzip.decompress(base64.b64decode(encoded)).decode()
old = '"\\nfn native_test_events("'
new = '"\\n/// Installs the explicit host-call boundary"'
if source.count(old) != 1:
    raise SystemExit(f"unexpected core/native.rs transform seam count: {source.count(old)}")
source = source.replace(old, new, 1)
exec(compile(source, "apply-849.materialized.py", "exec"))
