from pathlib import Path

script = Path(__file__).with_name("832-promote-metaspec.py")
source = script.read_text()
needle = "(and (= :primitive (:kind expected))\n         (= :any (:name actual))) true"
replacement = "(and (= :primitive (:kind actual))\n         (= :any (:name actual))) true"
if source.count(needle) < 2:
    raise SystemExit("expected compatibility marker in transformer")
source = source.replace(needle, replacement, 2)
namespace = {"__file__": str(script), "__name__": "__main__"}
exec(compile(source, str(script), "exec"), namespace)
