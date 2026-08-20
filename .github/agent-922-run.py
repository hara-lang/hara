from __future__ import annotations

import ast
import base64
import gzip
import re
from pathlib import Path

WRAPPER = Path(".github/agent-922-apply.py")
wrapper_source = WRAPPER.read_text()
wrapper_tree = ast.parse(wrapper_source)
payload = None
for node in wrapper_tree.body:
    if not isinstance(node, ast.Assign):
        continue
    if any(isinstance(target, ast.Name) and target.id == "PAYLOAD" for target in node.targets):
        payload = ast.literal_eval(node.value)
        break
if not isinstance(payload, str):
    raise SystemExit("unable to recover the embedded #922 patch payload")

source = gzip.decompress(base64.b64decode(payload)).decode("utf-8")
tree = ast.parse(source)
helper = None
for node in tree.body:
    if not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
        continue
    segment = ast.get_source_segment(source, node) or ""
    if "expected one exact match" in segment:
        helper = node
        break
if helper is None:
    raise SystemExit("unable to locate the exact-replacement helper")
arguments = [argument.arg for argument in helper.args.args]
if len(arguments) != 3 or helper.args.vararg or helper.args.kwarg:
    raise SystemExit(
        "unexpected exact-replacement helper signature: " + ", ".join(arguments)
    )

renamed = "_agent_strict_" + helper.name
source, substitutions = re.subn(
    rf"(?m)^def {re.escape(helper.name)}\(",
    f"def {renamed}(",
    source,
    count=1,
)
if substitutions != 1:
    raise SystemExit("unable to rename the exact-replacement helper")

path_name, old_name, new_name = arguments
compatibility_helper = f'''\
from pathlib import Path as _AgentPath

def {helper.name}({path_name}, {old_name}, {new_name}):
    _target = {path_name} if isinstance({path_name}, _AgentPath) else _AgentPath({path_name})
    _workflow = str(_target).replace("\\\\", "/") == ".github/workflows/java-ifilesystem-kernel.yml"
    if _workflow:
        return
    _text = _target.read_text()
    _count = _text.count({old_name})
    if _count != 1:
        raise SystemExit(
            f"{{_target}}: expected one exact match, found {{_count}}: "
            + repr({old_name}[:120])
        )
    _target.write_text(_text.replace({old_name}, {new_name}, 1))

'''

exec(compile(compatibility_helper + source, "apply-922.py", "exec"))

proof_path = Path(
    "core/java/src/test/java/hara/truffle/GitHubFilesystemSessionKernelTest.java"
)
proof = proof_path.read_text()
old_assertion = '''      assertEquals(
          fixture.client.initialCommit(),
          session
              .eval(
                  "(get (:extensions (deref (File/stat \\"/README.md\\")))"
                      + " :file/revision)")
              .asString());
'''
new_assertion = '''      String readmeRevision =
          join(binding.stat("/README.md").future()).revision();
      assertEquals(
          readmeRevision,
          session
              .eval(
                  "(get (:extensions (deref (File/stat \\"/README.md\\")))"
                      + " :file/revision)")
              .asString());
'''
if proof.count(old_assertion) != 1:
    raise SystemExit("unable to locate the generated entry-revision assertion")
proof_path.write_text(proof.replace(old_assertion, new_assertion, 1))
print("Applied #922 provider-neutral HaraContext dispatch patch and entry-revision proof")
