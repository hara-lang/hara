from pathlib import Path
import subprocess

source = subprocess.check_output(
    ["git", "show", "HEAD^^:.github/agent-patches/832-promote-metaspec.py"],
    text=True,
)
needle = "(and (= :primitive (:kind expected))\n         (= :any (:name actual))) true"
replacement = "(and (= :primitive (:kind actual))\n         (= :any (:name actual))) true"
if source.count(needle) < 2:
    raise SystemExit("expected compatibility marker in transformer source")
source = source.replace(needle, replacement, 2)
namespace = {"__file__": str(Path(__file__).resolve()), "__name__": "__main__"}
exec(compile(source, __file__, "exec"), namespace)
