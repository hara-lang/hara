# Hara ChatGPT webapp workflow

Use this document as the project-specific instruction set for ChatGPT webapp
work on `hara-lang/hara`.

## Operating boundary

- Use the GitHub connector as the sole durable path for repository discovery,
  issue state, branches, commits, pull requests, reviews, and Actions evidence.
- Do not leave an implementation as chat-only source, an unapplied patch, or a
  local-only file. Repository code exists only after it has been committed to a
  GitHub branch.
- Do not use, build, or depend on MCP for this workflow. In particular, do not
  route work through `hara-mcp`, `mcp.hara-lang.org`, or a browser evaluator.
- GitHub Actions is the execution environment for Rust and Java work initiated
  from the webapp. The connector controls delivery; Actions runs the committed
  code.

## Start every coding task

1. Resolve the owning repository and executable issue through the connector.
2. Read the root `AGENTS.md`, every applicable nested instruction file, the
   issue relationships, linked pull requests, and the relevant existing
   workflow and source files.
3. Fetch the exact declared base branch and SHA. Use the issue's Delivery base;
   when no base is declared, use `testing` for ordinary feature work.
4. Record a work-start comment containing the base SHA, branch, bounded scope,
   validation plan, and material assumptions.
5. Reuse a matching open branch or create
   `connector/<issue-number>-<short-slug>`. Never overwrite or absorb unrelated
   concurrent work.

## Mandatory Rust and Java rule

Whenever the requested outcome requires Rust under `core/rust/` or Java under
`core/java/`:

1. Create the smallest coherent implementation commit on the connector branch.
   Do not respond with proposed source only.
2. Open or update a draft pull request immediately after the first
   implementation commit. The pull request must identify one Primary issue.
3. Let `.github/workflows/connector-runtime-validation.yml` run from that exact
   committed branch head. Do not invent a temporary, issue-numbered,
   self-removing, source-patching, or write-enabled workflow.
4. Inspect the workflow run, jobs, steps, and failure logs through the GitHub
   connector. Treat the exact commit SHA and Actions result as the execution
   evidence.
5. Repair failures with follow-up commits on the same branch and inspect the
   replacement runs. Do not create a fresh branch merely to retry.
6. Keep `Core CI` authoritative. The connector lane is early, exact-commit
   evidence; it does not waive broader repository checks.

The permanent lane executes these checked-in commands:

```text
Rust:  bash scripts/runtime/run-connector-rust-validation
Java:  bash scripts/runtime/run-connector-java-validation
```

A Rust-only change runs the Rust job, a Java-only change runs the Java job, and
mixed changes run both independently. If a task needs a new reusable execution
check, add it to an existing permanent checked-in validation script or a
properly owned permanent workflow in the same pull request. Never interpolate
connector-supplied shell text into Actions.

## Hara source and other languages

For `.hal` changes, follow the native Hara source workflow in `AGENTS.md` and
its referenced skill. Rust or Java support needed by that change still follows
the mandatory commit-and-Actions rule above. For documentation or non-runtime
work, use the same connector-first issue, branch, commit, and pull-request
ledger even when no runtime job is selected.

## Evidence and completion

Keep durable decisions, scope changes, and progress in the issue or pull
request. Before marking a pull request ready, report:

- the exact base and head SHAs;
- files and behavior changed;
- the Actions workflow run and individual Rust/Java job outcomes;
- broader checks that ran, including `Core CI` when applicable;
- compatibility, risk, and any remaining work.

Use `Closes` only when the issue is completely delivered; otherwise use
`Advances`. Never claim code was executed from source shown only in chat.
