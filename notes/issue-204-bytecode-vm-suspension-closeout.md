# Bytecode VM suspension and resumability closeout

Issue: `hara-lang/hara#204`

Normative companion: `hara-lang/hara-specs-registry#8`

## Implemented execution model

The Rust bytecode VM parks one owned `Machine` inside `VmFiber`. The parked
machine retains its current function and instruction pointer, operand stack,
current frame, saved caller frames, lexical locals, hidden exception/finally
slots, async scheduler, pending children, and optional JIT state.

A run has four observable outcomes:

- `Returned(Value)`;
- `Failed(VmError)`;
- `Suspended(Promise)` at a pending `Await`;
- `Yielded(Value)` at a guest `Yield`.

Resumption continues the same machine. It does not reparse source, recompile
bytecode, rebuild lexical environments, or create evaluator continuation
objects.

## Await and exception behavior

`Await` remains the current instruction while its Promise is pending. A
fulfilled Promise replaces the Promise operand with its value and advances.
A rejected Promise enters the ordinary VM error path at the await site, so
static handler tables, cross-frame unwind, catch ordering, hidden pending
slots, and `finally` replacement semantics remain shared with synchronous
errors.

The public integration suite covers both directions:

- fulfillment below nested calls, followed by `finally` and caller
  continuation;
- rejection below nested calls, followed by `finally` and an outer catch.

## Yield behavior

`Yield` exposes one guest value without unwinding or advancing. The driver's
resume value is pushed back into the parked machine, becomes the value of the
yield expression, and execution continues through the existing frame stack.

## Async functions and cancellation

A prototype marked `^:async` always returns one stable result Promise. An
already-settled await keeps the fast path but still preserves the Promise
return shape. A pending child is owned by the VM scheduler and settles that
same result Promise when resumed.

Cancelling the result Promise invokes the child cancellation hook, which
notifies the host Promise currently awaited by the child and rejects the
result with the shared structured cancellation identity.

Cancellation notification is a host control signal rather than an implicit
guest throw injected at an arbitrary instruction. Fulfilled and rejected
await resumption use normal catch/finally semantics; broader structured
cancellation cleanup remains a separate concurrency contract.

## Coexistence boundary

`EvalFiber` remains the tree evaluator's CPS execution state. `VmFiber` is the
prepared-bytecode execution state. They share Values, Promises, providers,
namespaces, catch matching, and error identity, but do not share continuation
objects or silently fall back to one another.

The parked VM is process-local. It may hold native values, Vars, Promises,
provider callbacks, and scheduler references, so it is not a HALC, HBC0, HTA,
or durable workflow artifact.

## Verification

`core/rust/tests/vm_suspension.rs` exercises the contract through public APIs:

- pending await and `VmFiber::poll`;
- nested-frame fulfillment plus `finally`;
- nested-frame rejection plus `finally` and outer catch;
- yield and driver-supplied resume values;
- settled async result Promise shape;
- cancellation propagation to a pending host Promise.

Focused commands:

```sh
cargo test --manifest-path core/rust/Cargo.toml --test vm_suspension
cargo test --manifest-path core/rust/Cargo.toml --lib vm::
cargo build --manifest-path core/rust/Cargo.toml --target wasm32-unknown-unknown --lib
```

The remaining promotion evidence is a browser-host Promise integration lane
and comparative allocation/retained-state measurements against `EvalFiber`.
Those are evidence for promotion and optimization; they do not change the
process-local semantics pinned here.
