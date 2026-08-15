from pathlib import Path

path = Path(__file__).with_name("patch_result_tests_hal.py")
text = path.read_text()
replacements = {
    "(def f (res-success 11": "(def f (std.foundation/res-success 11",
    "(res? f)": "(std.foundation/res? f)",
    "(res-success? f)": "(std.foundation/res-success? f)",
    "(= 11 (res-data f))": "(= 11 (std.foundation/res-data f))",
    "(= :foundation (get (res-context f) :source))": "(= :foundation (get (std.foundation/res-context f) :source))",
}
for old, new in replacements.items():
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected one Java res-* test fragment {old!r}, found {count}")
    text = text.replace(old, new, 1)
path.write_text(text)
Path(__file__).unlink()
