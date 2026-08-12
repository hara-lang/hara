---
name: hara-xtalk-compatibility
description: Normalize canonical native Hara XTalk source while preserving JavaScript, Python, Dart, and Lua seed parity. Use for src-lang/xt/lang, test-lang/xt, target compatibility, access semantics, block/value contexts, normalized names, or generated target coverage.
---

# XTalk compatibility

Use `$hara-development` and `$hara-xtalk`. Keep canonical cleanup separate from
target-specific rewrites.

## Workflow

1. Locate the canonical `.hal` source, seed metadata, emitter rule, and target
   artifacts.
2. Evaluate the candidate natively and run its focused canonical test.
3. Normalize the canonical operation only when its meaning is target-neutral.
4. Inspect JavaScript as the first executable reference, then Python, Dart,
   and Lua coverage.
5. Regenerate target benches through the owning Hara generator; never edit
   generated `test-lang/xtbench` files.
6. Run the affected target tests and classify failures as canonical source,
   generator, runtime, or target rewrite before changing code.

Prefer implementations and adapters over suppression. Use a target transform
only when the target shape genuinely differs, and verify intentional coverage
count differences against seed metadata.

Read [references/seedgen-workflow.md](references/seedgen-workflow.md) for the
compatibility matrix and validation sequence.
