# Original JavaScript bootstrap against installed models

Authority: Foundation `fe54cc866473a888bfbd4ce2d3de6d8011f09f72`.
Target HEAD: `c89bc9b95b9c0895456db0bb17ef08b539f0f935`, with existing local changes.
Audit date: 2026-09-14. No production source changes in this audit.

## Current production failure

Fresh native command from the Hara project:

```text
../hara-native/core/rust/target/release/hara-native test --project . --file test/lang/runtime/basic/impl/process_js_test.hal
```

Session 91897 exited 1: zero assertions ran, one namespace-load error.
`process-js` calls the obsolete `lang.core.runtime/install-type` at line 45;
the restored original runtime owns `install-type!`. This is a mixed-pipeline
failure, not evidence that the original registration needs a compatibility alias.
The installed process also configures the handwritten `source-js/transport-source`.
Neither that bootstrap nor its registration shape is the requested final state.

The saved `lang_runtime_basic_type_basic_pending_stage.edn` requires the original
container and bench dependencies. Their native source files are absent:
`src/lang/runtime/basic/type_container.hal` and `type_bench.hal`.
Its previous 12-pass/one-error result stops on the same process registration.
Do not fix this by restoring the obsolete registration API or installing only
the process source without its complete dependencies and paired tests.

## Installed-model emission check

The replay authority is Foundation candidate record
`resources/code/migrate/candidates/lang_runtime_basic_js_bootstrap_native_stage.edn`.
Evaluate its existing `:replay-form`. For each returned program, keep the suffix
beginning at `(ns xt.lang.spec-base`, and prepend a namespace requiring the
installed `lang.model.v1.spec-js` and `lang.model.v1.spec-xtalk`. This removes
the old helper/JSX/root model overlays while retaining the saved XTalk fixture
source and original verification fact. Do not claim that these XTalk fixtures
are installed production sources.

Execute the resulting complete program through stdin in a fresh native project
test process. Session 52982 exited 0: one fact passed. Real Node v26.6.0 returned
all six exact data payloads: `42`, null, false, `hello λ`, `[1,2,3]`, and
`{"a":1}`. Emission used `impl/emit-entry-deps` with `:lang :js`, `:layout :flat`;
the printed output contains the original `return_encode`, `return_wrap` and
`return_eval` functions.

Negative proof: change only the first expected payload from 42 to 43 in the
saved verification fact. Session 38116 exited 1 with exactly one failed fact,
zero errors; the actual payload remained 42. No production expectation changed.
The restored complete program passed again in fresh session 49011, exit 0,
one fact passed and zero failures/errors.

The socket variant retains the saved original client DSL and invokes both
`emit-entry-deps` and `emit-as`. Its first run (39398) exited 1 before payload
execution because the sandbox denied socket listening. This is an environmental
failure, not a passing socket check.
After granting socket permission, fresh session 70839 exited 0: one fact passed,
zero failures/errors. All six payloads matched through a single real Node socket
connection. The emitted client calls `client_basic` with loopback host and an
ephemeral port; the native server currently listens on `0.0.0.0`. The saved test
dynamically isolates server state and kills/waits for its child and stops its
server in `finally`. Its printed bootstrap contains the original `net`,
`readline`, and `createRequire` imports and DSL-generated `client_basic` body.

This resolves the old model-overlay prerequisite for these two bootstrap probes.
It does not prove the complete process registration or basic-runtime lifecycle,
nor does it install the retained XTalk source fixtures. Continue with the saved
container/bench dependency pairs, then the full basic/process composition.

The next bench candidate correction is retained in Foundation candidates as
`lang_runtime_basic_type_bench_shell_forwarding_stage.edn` with its adjacent
audit note. It restores original shell-option forwarding through `tool.sh/sh`;
23 checks pass and both added contracts fail against the prior candidate.
The pair remains uninstalled pending the documented host fidelity boundaries.

## Dependency fingerprints

SHA-256 at validation time:

```text
7b8ac1fa255deb1ef86111e80711ede10cad10fdedcf1da520e0ee90c6bce721  src/lang/model/v1/spec_js.hal
7f58203638821d1a6c9af5a0d6a3652010ff04322ca7e9425c0391a5d489bad5  src/lang/model/v1/spec_xtalk.hal
18eb704ef1edb65a1e66cc19f81af1c1d24bc1f749441365a60825f6b1070bae  src/lang/core/impl.hal
0863b35c8581e188e8e43c34a1d57ef7192cf8de288cbea5bba706d166b79684  src/lang/runtime/basic/type_oneshot.hal
1c61208c5a8966cc71c1e29720a6ba0a1f435980b0b059f63e36a2a68128bef6  src/lang/runtime/basic/server_basic.hal
```

The full migration, production process replacement, historical namespace tests,
and removal of all remaining grammar-api consumers are incomplete.
