# Lua annex and eager-model prerequisite audit — 2026-09-15

Fresh native 0.1.33 execution of
test/lang/runtime/basic/impl/process_lua_test.hal terminates with zero passing
assertions and one load error: unbound lang.core.runtime/install-type.
This is a confirmed current first-load failure, not a timeout or missing Lua
executable. No process was left running by this check.

The installed process-lua source still registers through the old install-type
surface and imports the replacement type-basic implementation. Original
script-annex/start-runtime instead resolves the runtime registry's resource,
obtains its instance create/setup functions, and uses that lifecycle. Adding an
install-type compatibility alias would not restore the required architecture.

The complete original process-lua source was inspected at
fe54cc866473a888bfbd4ce2d3de6d8011f09f72. It uses install-type!, the original
resource type IDs, and emit-entry-deps/emit-as for oneshot/basic/websocket
bootstrap generation. Its source requires common-lib and common-promise as well
as the runtime owners. Preserve the original Lua/nginx program matrix and
local-rocks environment handling when porting; a default switch to system Lua
does not replace that contract.

Installed xt.lang.common-lib and xt.lang.spec-base still require lang.core.
Therefore a full process namespace load needs that facade's original eager-model
closure, not only the previously validated isolated return-eval fixture. The
existing Lua generated-bootstrap evidence remains useful, but does not prove
process registration or full common-lib loading. Continue the missing model
branches before claiming the deferred Lua annex execution assertions pass.

## Next branch: Bash

Read both complete original files:

- src/tahto/model/spec_bash.clj, SHA-256
  6c9833e3c1073889a3b91a7d185a572127614ad1c3f1777fc7eedc40b6557d9a
- test/tahto/model/spec_bash_test.clj, SHA-256
  0b89b5ad8dce3c5e692c869170b4dd018f716d70cde81ae4d4cc2f9798057e28

Structural inventory: 21 functions, 22 historical facts, 64 assertions.
The source depends on existing Book, grammar, emit/common/data/fn/top-level,
script, util and XTalk owners. Preserve the literal features/template topology,
:sh grammar tag, :bash Book language and :xtalk parent.

Required adaptations to verify include Java quote replacement, unary Clojure
conj, collection/form?, keyword rendering, optional rest bindings and quasiquote.
Top-level private bash-vector must become directly testable in an internal
namespace. Quoted DSL defn- is target syntax, not a private host definition;
do not rewrite it merely because the host disallows top-level defn-.

The historical tests mix helper expansion, emitted strings, real Bash execution
and notification sockets. They also include an emission expression followed by
an expected prose string without an assertion arrow in bash-defn; do not count
that unchecked expression as a historical assertion. Keep all 64 actual
assertions and account separately for added coverage. Shell tests mention bc,
process substitution, /dev/tcp, and a project.clj fixture; inventory their
environment requirements explicitly. Do not silently drop the integration
facts to obtain a green model-only count.

No production source/test files were changed during this audit. The next action
is a scoped Bash source/test adapter using the existing structural engine, with
full candidate evaluation before installation. The full migration goal remains
active.
