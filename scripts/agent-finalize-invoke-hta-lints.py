#!/usr/bin/env python3
from pathlib import Path

path = Path("core/rust/tests/invoke_hta.rs")
text = path.read_text(encoding="utf-8")
old = "use hara_wasm::{InvokeHtaError, Runtime, MAX_INVOKE_HTA_RESULT_BYTES};\n"
new = "use hara_wasm::{InvokeHtaError, Runtime};\n"
count = text.count(old)
if count != 1:
    raise SystemExit(f"expected one invoke HTA import replacement, found {count}")
path.write_text(text.replace(old, new, 1), encoding="utf-8")
