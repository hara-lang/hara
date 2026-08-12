# Hara emit pipeline

```text
source form
  -> input extraction
  -> staging and symbol resolution
  -> canonical rewrite
  -> target rewrite
  -> generic/custom emission
  -> target source
```

## Stage ownership

- Input extraction identifies host values, language blocks, pointers, and
  template splices. It must not choose target syntax.
- Staging expands macros/templates, resolves symbols, records dependencies, and
  preserves canonical operation identity.
- Canonical rewriting converts portable sugar into target-neutral semantic
  operations.
- Target rewriting handles only constraints specific to a target runtime.
- Emission handles tokens, precedence, separators, wrapping, blocks, and
  layout. Custom emitters are for syntax that shared templates cannot express.

## Review checklist

- Is portable meaning established before the target is selected?
- Could a shared grammar entry replace duplicated target entries?
- Does a target rewrite preserve whether the value is a key access, index
  access, expression, statement, or block?
- Are input shape, precedence, missing-value behavior, and assignment behavior
  tested for every custom operation?
- Do canonical and generated target tests exercise the same intended fact?

Evaluate native Hara source with `hara --offline stdin`, run the written source
with `hara --offline run`, and use focused `hara project test` paths for both
canonical and generated behavior.
