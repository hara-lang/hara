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

main_block = '\nif __name__ == "__main__":\n    main()\n'
if source.count(main_block) != 1:
    raise SystemExit(f"unexpected materializer main block count: {source.count(main_block)}")
source = source.replace(
    main_block,
    r'''
def _repair_provenance_keywords(text):
    return (
        text.replace(":lang/:sause-class", ":lang/cause-class")
        .replace(":lang/sause-class", ":lang/cause-class")
        .replace(":lang/:cause-class", ":lang/cause-class")
        .replace(":lang/sause-native-type", ":lang/cause-native-type")
        .replace(":lang/:cause-native-type", ":lang/cause-native-type")
    )

_original_transform_hal = transform_hal

def transform_hal(relative, source):
    return _repair_provenance_keywords(_original_transform_hal(relative, source))

_original_build_candidates = build_candidates

def build_candidates(root, output):
    paths = _original_build_candidates(root, output)
    for relative in [
        "core/lib/src/lang/common/provenance.hal",
        "core/lib/test/lang/common/provenance_test.hal",
    ]:
        candidate = output / relative
        if candidate.is_file():
            write(candidate, _repair_provenance_keywords(read(candidate)))
    return paths

if __name__ == "__main__":
    main()
''',
    1,
)
exec(compile(source, "apply-849.materialized.py", "exec"))
