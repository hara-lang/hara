---
name: hara-development
description: Develop, edit, debug, or review native Hara `.hal` source and Hara language libraries. Use for any `.hal` change, Hara parser or runtime failure, source-gate failure, or Hara project test workflow. Establishes the native evaluate-test-write-run-test cycle used by more specialized Hara skills.
---

# Native Hara development

Use the Hara CLI as the source authority. Evaluate in fresh native processes;
do not use persistent language sessions, external source linters, or automatic
source-repair tools.

## Edit cycle

For each coherent `.hal` change:

1. Read the owning source and its narrowest test.
2. Construct the complete proposed file, not only the changed lines.
3. Evaluate that candidate with the project runtime using
   `hara --project <root> --offline stdin` when `project.edn` owns the source,
   or `hara --offline stdin` for a standalone file.
4. Run the narrowest applicable assertion or
   `hara --project <root> --offline project test <path>`.
5. Write a small edit with the native editor or `apply_patch`.
6. Evaluate the written file with `hara --offline run <file>`.
7. Repeat the focused test, then run a broader project check only when the
   change warrants it.

Each invocation creates a fresh Hara execution context. If parsing fails,
correct the proposed source and evaluate it again.

## Source ownership

- Edit canonical `.hal` sources rather than generated Rust snapshots, target
  programs, SQL, RPC modules, or bytecode artifacts.
- Find the generator or language specification before changing generated
  output.
- Preserve the source layout described by the repository `AGENTS.md`.
- Use an applicable domain skill in addition to this one for PostgreSQL,
  XTalk, seed generation, or language specifications.

## Gate behavior

The source gate reconstructs a complete proposed file, applies the registry
source rules, and evaluates it before a write. After a write it evaluates the
actual file. The shell gate blocks common ways of bypassing this cycle.

Read [references/gates.md](references/gates.md) when diagnosing a hook denial
or changing the gate implementation.
