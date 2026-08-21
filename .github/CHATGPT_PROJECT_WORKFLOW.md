# ChatGPT webapp project workflow

This is the canonical project instruction for a ChatGPT webapp project working
on `hara-lang/hara`. GitHub is the execution host and durable record. MCP is not
part of this workflow.

## Operating contract

Use the GitHub connector for every repository read and write: issues,
relationships, branches, files, commits, pull requests, checks, jobs, logs, and
repairs. Do not treat chat text, an uploaded patch, a local worktree, or an
uncommitted code block as delivered Hara source.

Do not design, install, call, or depend on `hara-mcp`, `mcp.hara-lang.org`, a
browser evaluator service, or any other MCP transport while this contract is in
force. Those surfaces are explicitly deferred.

GitHub Actions validates source but does not author it. Validation workflows
must keep `contents: read`; do not use a workflow to materialise a patch,
rewrite a branch, remove its own transport, or push product commits. For an
atomic multi-file change, use the connector's Git blob/tree/commit operations.

## Start of every implementation

1. Read `AGENTS.md`, `.github/WORKFLOW.md`, this file, the owning issue, linked
   issues and pull requests, and the current versions of every file to change.
2. Resolve the exact base from the owning issue or existing PR stack. Feature
   work targets `main`; `testing` is a legacy integration branch and is not a
   required stop in the normal flow. Record the chosen base SHA in the PR.
   Never absorb unrelated concurrent work.
3. Create or update an executable issue with Outcome, Scope, Acceptance
   criteria, Validation, Relationships, Readiness, and Delivery.
4. Create `agent/<issue>-<slug>` from the exact base SHA. Push the branch and
   let the push-triggered readiness lane run before opening a pull request.
   Reuse an existing branch and draft PR when they already own the same work.
5. Establish the current Actions baseline before changing code when an existing
   failure could affect classification.

## Rust and Java commit rule

A request that needs Rust or Java is not implemented by returning code in chat.
Commit one bounded, runnable slice through the GitHub connector. Open a draft
pull request after the push-triggered readiness lane is green; this keeps
incubating branches testable without filling the repository with premature
pull requests. Copilot cloud-agent work is the exception: GitHub creates its
draft pull request automatically, and it must remain draft until the same gate
and focused workflows are green.

The first executable commit must contain:

- the production change;
- focused tests or a committed executable smoke path;
- any reusable validation-script change needed to run that proof;
- no generated build output, temporary patch transport, or issue-specific
  workflow.

Use `Connector code execution` as the minimum committed-code lane:

- Rust changes under `core/rust/`, except browser-only `core/rust/web/` changes,
  run formatting, clippy, the main Rust tests, the raw-runtime tests, and a CLI
  smoke evaluation;
- Java changes under `core/java/` run the JDK 21 Truffle Maven package/test
  suite and a CLI smoke evaluation;
- a mixed commit runs both jobs independently.

The lane runs on pushes to `agent/**`, `chat/**`, `codex/**`, and `copilot/**`,
and on pull requests to `main`. The push run is the preflight; the base-aware
pull-request run is the authoritative comparison and exposes the normal
repository checks once the draft exists.

`Connector code execution` is a floor, not a substitute for `Core CI` or a
focused permanent workflow. Rust, Java, Hara, and browser-loader changes select
their corresponding vertical jobs; Wasm, native-image, conformance, benchmark,
or provider work must also run the relevant existing lane. When no permanent
lane can execute required behavior, add a reusable script and extend the
closest stable workflow in the same product commit; do not create a temporary
agent workflow.

## Actions repair loop

After each connector-authored commit:

1. Read the exact workflow runs for the commit.
2. Inspect failed jobs, steps, and logs rather than guessing from check names.
3. Classify failures as introduced, unchanged baseline, environmental, or
   blocked by another explicit change.
4. Repair introduced failures with a follow-up commit on the same branch and
   let Actions rerun. Do not open a replacement PR.
5. Keep the draft PR body current with the exact head SHA, commands exercised,
   passed evidence, remaining failures, and scope boundaries.

Do not claim that code compiled, ran, or passed until the corresponding Actions
run for the reported SHA says so. Do not mark a PR ready or merge it unless the
user explicitly asks and all acceptance-relevant introduced failures are
resolved.

## Delivery report

A completed response names the issue, branch, commit SHA, draft PR, Actions
runs, passed Rust/Java evidence, and any precisely classified blocker. Durable
decisions belong in the issue or PR; chat is only the control surface.
