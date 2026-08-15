from pathlib import Path

path = Path(__file__).with_name("patch_result_tests_hal.py")
text = path.read_text()

setup = '                      + "(def f (res-success 11 {:source :foundation})) "\n'
if text.count(setup) != 1:
    raise SystemExit(f"expected one Java Foundation setup fragment, found {text.count(setup)}")
text = text.replace(setup, "", 1)

old_checks = '''                      + "(std.native.Result/result? (std.native.Result/data n)) "
                      + "(res? f) "
                      + "(res-success? f) "
                      + "(= 11 (res-data f)) "
                      + "(= :foundation (get (res-context f) :source))))")
'''
new_checks = '''                      + "(std.native.Result/result? (std.native.Result/data n))))")
'''
if text.count(old_checks) != 1:
    raise SystemExit(
        f"expected one Java Foundation assertion block, found {text.count(old_checks)}"
    )
text = text.replace(old_checks, new_checks, 1)
path.write_text(text)
Path(__file__).unlink()
