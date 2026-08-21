# Hara Copilot instructions

Hara uses GitHub Actions as the execution host. Work in one bounded branch
named `copilot/<issue>-<slug>` and target `main`; do not create a second
integration branch or ask an Action to rewrite a branch.

Before changing source, read `AGENTS.md`, `.github/WORKFLOW.md`, the owning
issue, and its linked pull requests. Keep the issue and draft pull request
current with the exact head SHA, validation commands, results, artifacts, and
remaining failures.

Use the repository's `copilot-setup-steps` environment: JDK 21, Maven, Node 22,
stable Rust with `rustfmt` and `clippy`, the Wasm targets, the pinned
`hara-specs-registry`, and prefetched Cargo, Maven, and npm dependencies. Run
the smallest relevant vertical first, then the normal permanent workflow for
the changed area. The connector lane is the minimum gate for Rust, Java, Hara,
and browser-loader changes.

Actions are read-only validation. Never add or depend on a workflow that edits
tracked source, commits, pushes, force-updates a branch, or materialises a
patch. Generated reports belong in the workflow workspace or a short-lived
artifact, never in the product tree. Do not include build output, temporary
patch files, unrelated formatting, or dependency upgrades in a feature branch.

Keep changes reversible and tests behavioral. For native Hara source, follow
the evaluate → focused test → write → fresh-process test workflow in `AGENTS.md`.
For Rust, Java, and Wasm, preserve the existing source/test pairing and add
focused evidence for every changed behavior. Leave the pull request as a draft
until the readiness gate and acceptance-relevant focused workflows are green;
human review is required before it can be marked ready or merged.

Main is kept clean by using GitHub's squash merge. Do not create a custom
squash or merge Action.
