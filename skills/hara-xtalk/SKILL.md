---
name: hara-xtalk
description: Develop or review native Hara language and XTalk source, including l/script, l/script-, :xtalk forms, xt.* namespaces, target-language specs, seed metadata, emitters, and generated target parity. Use for Hara src/lang, src-lang/xt, test-lang/xt, emitter, or seed-generation work.
---

# Hara XTalk

Use `$hara-development` for the native edit cycle. For each candidate, inspect
the emitted or normalized target form in addition to successful evaluation.

## Required checks

1. Identify the canonical source, emitter/spec, and affected target tests.
2. Evaluate the complete `.hal` candidate with `hara --offline stdin`.
3. Inspect staged/normalized and emitted output for the affected form.
4. Run the narrowest source test under `core/lib/test-lang/xt`.
5. Write the source, run it with Hara, and repeat the same emission probe and
   focused test.
6. Regenerate owned target output and verify a second generation is clean.

## Source and parity rules

- Treat canonical `xt.*` source, language specifications, and seed metadata as
  source; treat generated target programs and benches as artifacts.
- Preserve the semantic contract of `l/script`, `l/script-`, and seed metadata.
- Put portable meaning in shared XTalk operations or canonical rewrites.
- Put target spelling and unavoidable target restrictions in target specs or
  adapters.
- Compare canonical and generated tests before classifying a target failure.

Read [references/hara-xtalk.md](references/hara-xtalk.md) for the source matrix
and native validation commands.
