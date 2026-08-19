from pathlib import Path

path = Path(__file__).resolve().parents[2] / "core/rust/tests/std_typed_schema.rs"
text = path.read_text()
old = ':pattern "^a"'
new = ':pattern \\"^a\\"'
if text.count(old) != 1:
    raise SystemExit(f"expected one generated Rust regex marker, found {text.count(old)}")
path.write_text(text.replace(old, new, 1))
print("fixed #832 generated Rust probe quoting")
