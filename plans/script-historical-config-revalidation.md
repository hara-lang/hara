# Script historical configuration gap — 2026-09-15

Authority: Foundation `test/tahto/core/script_test.clj` at
`fe54cc866473a888bfbd4ce2d3de6d8011f09f72`, the second assertion in
`resolve-runtime-config`'s historical fact.

The installed test retains the first quoted-symbol assertion and adds direct
resolver collection tests. Those do not preserve the second assertion's call
through `script-test-prep` with `:runtime :basic`. The installed Lua preparation
test checks library/module ownership but not this JavaScript configuration path.

A fresh native 0.1.33 probe passed (1 passed, zero failures/errors):

```clojure
(script/script-test-prep
 :js
 {:runtime :basic
  :config {'+runtime-config-key+ '+runtime-config-value+
           :startup {:args ["/bin/sh" "-lc" '(str "rea" "dy")]}}})
```

The probe defined the original key/value Vars as `:port` and `17001`, selected
`:port` and `:startup` from the result, and asserted exactly
`{:port 17001 :startup {:args ["/bin/sh" "-lc" "ready"]}}`. Selecting these keys
preserves the historical partial-map assertion; it does not weaken it to a
type check. It used the installed JavaScript model, a cloned library, isolated
space/resource bindings, and nested `finally` cleanup of the annex, spaces,
library, and settings. No process was started.

Command: `../hara-native/core/rust/target/release/hara-native test --project .
--file test/.script-historical-config-probe.hal`, with the complete probe
transported through a temporary `/dev/stdin` symlink. Exit 0. Symlink removed.

This is diagnostic evidence, not an installed historical test or a completed
script migration. Next: restore this assertion through the saved structural
migration/test replay chain, retain the installed test before-image, demonstrate
a failing negative control, and verify the complete written source/test pair
and byte-exact replay. Do not classify this assertion as process-runtime blocked.
Real Lua oneshot/annex execution and XTalk module reload assertions remain
separate obligations; emission-only default-runtime tests do not replace them.

## Installed assertion

The assertion is now appended to `test/lang/core/script_test.hal` as
`historical-prep-config`. The source implementation is unchanged. Full candidate:
49 passed. Negative control changing only the expected port to 17002: 48 passed,
one assertion failure, zero errors. The written suite passed all 49 tests twice
in fresh native processes, including the existing Lua Entry/emission assertions.

Foundation records `lang_core_script_historical_config_stage.edn` and
`lang_core_script_historical_config_written.edn` retain the full source/test
before-images, addition, native replay form, adaptations, hashes, and evidence.
Record read-back matched; native replay reproduced both installed files exactly
and repeated identically. This append record extends the previously verified
structural migration chain; catalog handler integration remains unfinished.

Current native CLI scaffold preview found zero changes/errors. An initial write
overlapped the test edit and rejected a stale before-image; a serial rerun passed
with zero changes/errors. This does not certify the older installed launcher.
Fresh native correspondence checks also reported zero incomplete and unchecked
findings, with zero errors. The temporary stdin symlink was removed.
