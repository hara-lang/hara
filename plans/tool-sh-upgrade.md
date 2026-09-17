# tool.sh Foundation shell boundary

Implemented locally against Hara base
`c89bc9b95b9c0895456db0bb17ef08b539f0f935`.
Behavioral authority: Foundation `src/std/lib/os.clj`, `sh`, at
`fe54cc866473a888bfbd4ce2d3de6d8011f09f72`.

## Owning API

`src/tool/sh.hal` owns the added `sh` operation and its `shell-options` and
`shell-output` implementation functions. No `std.lib.os` namespace, Future,
wrapped type, or native Process contract change was introduced.

- `run` remains strict: a nonzero exit is an error Result.
- `sh` accepts an executable plus argv/options, or a map with `:args`.
  Arguments remain literal; it never invokes a shell implicitly.
- `sh` defaults to trimmed stdout in a native Result. Nonzero exit alone is
  not an error. Stderr-only output throws unless ignored or selected.
- `:wrap false` returns text; `:output false` returns a completed process;
  `:wait false` returns a process without waiting; `:async true` returns a
  Promise of the corresponding outcome.
- `:root` selects cwd. Environment keys and values are stringified. `:trim`,
  `:output-errors`, and `:ignore-errors` follow the original output branches.
- Stdin remains open, as in Foundation. Callers receiving a process own
  `close-input` and `kill`; both have idempotence tests.
- `run-record` now actually synchronizes before returning its documented
  completion map, including nonzero exits. Previously an unresolved process
  Promise could escape through `deref`; immediate map access reproduced this.

Trimming and environment conversion are intentionally lossy. Wrapped shell
results retain the untrimmed completion record and normalized input options
(including the original environment values) in Result context. Choosing
`:wrap false` explicitly discards that provenance. Option normalization is
tested for idempotence, and stdin/output tests cover UTF-8 round trips.

## Explicit remaining differences

- Native Process does not support inherited stdio: `:inherit true` fails
  before spawning. No process-tree kill implementation was added.
- `:wait false :output true` is rejected deterministically instead of copying
  the original race against Java's `exitValue` on an unfinished process.
- `:print` prints argv, not Foundation's source-location diagnostic header.
- With no root or PWD, cwd falls back to `OS/cwd`.
- Unknown shell options are rejected. Existing strict `run` options are
  unchanged; `sh` does not add a timeout contract.
- Native Promise rejection currently renders thrown exceptions into host
  error text. Async stderr failure is tested as rejection retaining its
  message; synchronous stderr failure retains structured process data.
- Docker/bench migration rules and callers have not yet been switched to this
  boundary. Their historical test migration remains separate work.

## Validation

The complete candidate was evaluated and tested before writing. The written
source loaded in a fresh native process:

```
../hara-native/core/rust/target/release/hara-native run src/tool/sh.hal
../hara-native/core/rust/target/release/hara-native test --project . --file test/tool/sh_test.hal
```

The focused suite passes 50 facts using real local `/bin/sh`, `/bin/echo`, and
`/bin/cat` children. No Docker daemon or remote service was contacted. The same
50 facts all go red against deliberately incorrect function definitions in a
disposable native process: 34 assertion failures and 16 errors. Restoring the
correct candidate and testing the written file both pass.

The final combined run of `test/tool/sh_test.hal` and
`test/std/foundation/promise_test.hal` passes 62 checks (50 shell, 12 Promise).
The scoped `git diff --check` passes, and the temporary stdin test link was
removed.

Native `code.manage.unit` scaffold preview/replay and correspondence checks on
the written source/test pair report 28 definitions, no new scaffold obligations,
no incomplete tests, and no unchecked tests. The installed `hara manage
scaffold` preview and write commands both crashed with stack overflow, so the
underlying native inventory API was used instead. Tests were hand-written;
native structural formatting converted their initial Test/check wrappers to
fact blocks without generating their assertions.

No commits or publication were requested or performed.
