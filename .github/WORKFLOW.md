# GitHub Actions

Hara uses `testing` as its integration branch and `main` as its production
branch. Feature pull requests normally target `testing`; promotion pull
requests move reviewed changes from `testing` to `main`.

Workflow files are durable project infrastructure. One-off branch repair,
source export, migration, rebase, or patch-application jobs do not belong in
`.github/workflows`. Run those tasks locally or put reusable validation in a
script under `scripts/runtime/` and invoke that script from an existing
workflow.

## Pull-request and branch checks

- `core-ci.yml` is the primary required check for `main` and `testing`. It
  validates the Rust, Java/Truffle, and HAL runtimes. Its bytecode-observation
  job verifies native and browser observation sessions.
- `connector-runtime-validation.yml` is the read-only ChatGPT webapp connector
  lane. Pushes to `connector/**` and `agent/**`, and pull requests to `main` or
  `testing`, detect committed Rust/Java scope and invoke the checked-in
  connector validation scripts. It supplies early exact-commit evidence; the
  existing `core-ci.yml` (`Core CI`) remains authoritative.
- `language-source-layout.yml` is the read-only Language CI gate. It checks
  source layout, namespace ownership, `code.test` ownership,
  retired standard-library namespaces, and the Foundation parity ledger.
- `lang-runtime.yml` validates portable JavaScript, Python, and Lua emission
  plus the Foundation runtime smoke tests.
- `foundation-migrate.yml` validates the pinned Foundation migration audit and
  recorded Foundation evidence.
- `core-ci.yml` also owns collection protocol, live-session, tool-lint, and
  bang-name policy coverage through the checked-in runtime and library suites.
- `code-vm-conformance.yml` owns Code VM validation: Rust/Truffle report parity,
  live interpreter semantics, snapshots, and browser-Wasm compilation.
- `whole-wasm-native-browser-parity.yml` compiles one whole-Wasm artifact,
  executes that exact artifact with Wasmtime, then verifies it in Chromium.
- `std-db-graph.yml` and `std-db-runtime.yml` are focused database language and
  runtime checks.
- `truffle-cli-startup-evidence.yml` records focused Truffle CLI cold-start
  evidence when launcher code changes.

Focused workflows use path filters so unrelated changes do not pay for every
specialized suite. They should target both `main` and `testing` unless there is
a documented release-only reason not to.

## Manual conformance

- `main.yml` is the manually dispatched full runtime conformance matrix. It
  covers native Rust, raw/Wasm runtimes, browser loaders, Java/Truffle,
  native-image, parity corpora, and benchmark evidence. Run it before releases
  and after broad compiler or runtime changes; it is intentionally not a
  per-commit workflow.

## Publication and releases

- `publish-hta.yml` publishes `@hara-lang/hta` from `hta-v*` tags using npm
  trusted publishing.
- `publish-maven-snapshot.yml` manually publishes a validated Maven snapshot.
- `publish-rust-crates.yml` manually publishes Rust crates from an explicit tag
  or commit in dependency order.
- `cut-reviewed-release.yml` validates a reviewed release manifest, creates its
  immutable tag, starts `release.yml`, and verifies the public assets.
- `release.yml` builds CLI, Truffle, and Studio runtime artifacts from `v*`
  tags, creates or updates the GitHub release, smoke-tests the installers, and
  updates Homebrew formulas.

Publication remains separate from branch promotion because registry releases
and Git tags are immutable.

## Workflow rules

1. CI and validation workflows are read-only (`contents: read`). Only release
   and package publication workflows may write repository or registry state.
2. Workflows call checked-in scripts for non-trivial logic. Do not embed source
   patches or generated product code in YAML.
3. Do not create workflows tied to a feature branch, pull-request number, or
   temporary agent task.
4. Every workflow must have a stable owner and purpose documented above.
5. Prefer adding a job or path filter to an existing workflow over creating a
   new file.
