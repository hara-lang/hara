# Hara async execution

This area specifies asynchronous function semantics independently from any host.
The layers are deliberately separate:

```text
Promise -> eventual value
VM Fiber -> resumable bytecode execution
Host -> owns fibers internally and exposes their result promises
```

The normative contract is [async-spec.md](async-spec.md). Machine-readable
examples are in [conformance/async.edn](conformance/async.edn).
