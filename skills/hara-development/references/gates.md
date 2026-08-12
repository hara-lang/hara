# Hara source gates

## Decision surface

| Operation | Pre-write | Post-write |
| --- | --- | --- |
| Read/search `.hal` | Allowed | Not applicable |
| Edit/Write/apply_patch | Reconstruct full candidate, apply policy, evaluate with `hara --offline stdin` | Evaluate actual file with `hara --offline run` |
| Delete `.hal` | Confirm patch shape; no source evaluation | Confirm the file is absent |
| Shell write to `.hal` | Denied for recognized write forms | Not applicable |
| Hara build/test/run | Allowed | Not applicable |

The source gate enforces syntax/evaluation and the registry source conventions.
It does not select the correct behavioral test, prove emitted target parity, or
repair source. Those remain part of the skill workflow.

## Source conventions

The installable `source-gates.json` snapshot is generated from the normative
registry document. It currently rejects dynamic `requiring-resolve`, references
to `clojure.*` namespaces, explicit imports of Foundation builtin libraries,
and `lang.*` source under the noncanonical root.

## Failure handling

- Correct parser and evaluation failures in the proposed source, then retry.
- Set `HARA_BIN` only when project discovery cannot locate the intended CLI.
- Treat a missing or malformed policy snapshot as a gate failure.
- Do not bypass a gate with a shell or general-purpose interpreter.

The shell gate prevents common accidental bypasses. It is not a substitute for
filesystem sandboxing or code review.
