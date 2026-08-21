# GitHub Actions

Hara uses `main` as the integration and production branch for feature pull
requests. The old `testing` branch remains available for legacy maintenance and
manual comparisons, but it is not part of the normal delivery path.

Workflow files are durable project infrastructure. One-off branch repair,
source export, migration, rebase, or patch-application jobs do not belong in
`.github/workflows`. Run those tasks in the authoring environment. A ChatGPT
webapp session must author direct commits through the GitHub connector and use
the permanent connector execution lane; it must not use a workflow to apply or
rewrite its source.

## Pull-request and branch checks

- `core-ci.yml` is the primary required check for `main` (and remains useful on
  the legacy `testing` branch). It
  validates the Rust, Java/Truffle, and HAL runtimes. Its bytecode-observation
  job verifies native and browser observation sessions.
- `connector-code-execution.yml` is the stable, read-only agent lane. On
  `agent/**`, `chat/**`, `codex/**`, and `copilot/**` pushes and on pull requests
  to `main`, it classifies the committed diff and executes the affected Rust,
  Java, Hara, and browser-loader vertical through checked-in scripts. Its
  `Agent readiness / Required gate` job is the stable preflight required before
  a connector-authored branch is considered ready. It complements rather than
  replaces `core-ci.yml` and focused workflows.
- `package-runtime-contract.yml` is the focused Rust package gate. It validates
  generated and authored `package.edn` parsing, complete archive-entry digest
  verification, exact JVM/Wasm preflight, content-addressed extraction, and
  installation rollback on package-related branch and pull-request changes.
- `copilot-setup-steps.yml` prepares the Copilot cloud-agent environment with
  JDK 21, Node 22, stable Rust plus Wasm targets, the pinned specifications
  registry, and prefetched dependencies. It does not build releases or write
  source.
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
   new file. The Copilot setup workflow is the one documented exception because
   GitHub requires that exact workflow name and job for cloud-agent setup.
6. Connector-authored Rust and Java must be committed before execution. Actions
   validates the exact commit; it never authors, materialises, or repairs the
   implementation.
