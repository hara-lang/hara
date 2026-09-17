# Foundation dissoc: zero-key contract

The native owner is `src/std/foundation.hal`; its path-matched tests are in
`test/std/foundation_test.hal`. The Foundation Var now has signature
`[value & keys]` and schema `[:fn [:any & :any] :any]`. No-key calls return the
original value without copying or dispatching a removal. Each supplied key
still calls the fixed-arity `IDissoc/dissoc` protocol method.

`(dissoc m)` and `(apply dissoc m [])` return m, retaining map identity and
metadata. Nil remains nil, sets remain supported, and duplicate/missing keys
retain the existing semantics. This implements zero **keys**, not the literal
zero-argument call `(dissoc)`; the latter's return contract is awaiting the
user's clarification.

The archived variadic inventory describes the old `[value key & more]`
contract. This note records the current source-backed replacement without
editing the historical archive.

Validation used the current native release binary. A complete Foundation
candidate and the written Foundation file loaded successfully. All 219
Foundation unit checks passed, with four added dissoc checks. Schema.base's
19 checks and PostgreSQL application's 38 checks also passed. The old ordinary
project evaluator rejected the Foundation Var call with "function expects at
least 2 arguments"; the new evaluator returns the exact retained/removal values.
The unit runner already accepted the old call, so it was not counted as a
negative control for this evaluator discrepancy.

Schema.base now uses the original `(apply dissoc m (keys expanded))` again.
The temporary reduction adaptation in `schema_scope_empty_keys_installed.edn`
is superseded by this Foundation contract change; retain that historical record
as evidence, not as the current replay target. Scaffold preview/write for
schema.base reported zero changes/findings. Its complete candidate and written
file load, and its existing focused tests pass.

The restored schema also passes all cold and warm interactive forms. Evidence:
`.hara/test/dissoc-zero-keys-interactive.edn`.

The source changes are limited to the dissoc signature/schema/loop seed and the
schema call above. Existing Foundation tests are retained; the four new checks
are appended to the existing dissoc correspondence block. No reference source,
native protocol, generated runtime artifact, or database was changed.
