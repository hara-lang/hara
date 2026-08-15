from pathlib import Path

path = Path(__file__).with_name("patch_code_test_result_timeout.py")
text = path.read_text()

old_require = '''(ns code.test.base.process
  (:require [code.test.checker.common :as common]
            [std.foundation :as foundation]))
'''
new_require = '''(ns code.test.base.process
  (:require [code.test.checker.common :as common]))
'''
count = text.count(old_require)
if count != 1:
    raise SystemExit(f"expected one Foundation require block, found {count}")
text = text.replace(old_require, new_require, 1)

replacements = {
    "foundation/res-synchronize": "Result/synchronize",
    "foundation/res-error-value": "Result/error-value",
    "foundation/res-context": "Result/context",
    "foundation/res-error?": "Result/error?",
    "foundation/res-data": "Result/data",
    "foundation/res-error": "Result/error",
    "foundation/res-timeout?": "Result/timeout?",
    "std.foundation/res-timeout?": "Result/timeout?",
    "std.foundation/res-error-value": "Result/error-value",
}
for old, new in replacements.items():
    if old in text:
        text = text.replace(old, new)

path.write_text(text)
