---
name: hara-dev-spec-writer
description: Design, simplify, review, or implement native Hara language specifications, grammar entries, resolve/rewrite/emit stages, XTalk canonicalization, and target-language parity. Use when working on Hara specs or emitters and deciding whether behavior belongs in shared grammar, canonical rewriting, target rewriting, or emission.
---

# Hara language specification writer

Use `$hara-development`; also use `$hara-xtalk` for portable language or seed
changes.

## Design contract

- Keep a target spec close to the shared grammar: helpers, features, template,
  grammar, then metadata/book/init.
- Put portable meaning in shared grammar or canonical XTalk rewrites.
- Put target spelling and genuine target restrictions in target specs.
- Resolve names and dependencies before target rewriting.
- Keep custom emitters small and preserve operation identity through staging.
- Edit the owning source rather than generated target output.

## Validation

For each change, inspect the owning grammar/spec and focused tests, evaluate the
complete native Hara candidate, and probe staged and emitted output for an
expression, a statement, and an edge case. After writing, run the actual file,
repeat those probes, and run the affected canonical and generated target tests.

When targets diverge, identify the first differing stage: input, staging,
canonical rewrite, target rewrite, or final emission.

Read [references/pipeline.md](references/pipeline.md) for the pipeline boundary
and review checklist.
