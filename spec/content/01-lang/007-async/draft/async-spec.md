# Hara async semantics

Status: draft normative contract.

## Function and await semantics

| Form | Required result |
| --- | --- |
| Ordinary function call | A value |
| `^:async` function call | A promise |
| `co/await` of a fulfilled promise | Its fulfilled value |
| `co/await` of a rejected promise | Throw the rejection through normal exception handling |
| `co/await` of a pending promise | Suspend the current VM fiber |
| Async function returns a value | Resolve its result promise to the value |
| Async function returns a promise | Adopt the returned promise |
| Async function throws | Reject its result promise |
| Async function is cancelled | Reject its result promise with structured cancellation data |

Calling an async function never implicitly awaits it. An async function is valid
even when every path completes synchronously.

Promise is a native value and MUST NOT contain fiber identifiers, host pointers,
reporting fields, or Nginx state. Cancellation is represented as a structured
promise rejection, not as a fourth promise settlement state.

## Static suspension restriction

`std.foundation.coroutine/await` is legal only inside:

1. a function declared with `^:async`; or
2. the direct function literal supplied to `std.foundation.coroutine/create`.

Every nested function establishes a new suspension context. Permission from an
outer function is not inherited. The compiler MUST recognize `await` through
the resolved canonical Var, after namespace and alias resolution, rather than
by matching source spelling.

Passing a known async function to `co/create` SHOULD produce a warning because
calling the function already returns its result promise. Discarding a known
async result MAY produce a warning. An async function without suspension MUST
remain legal.

Normalized analyzer signatures use the shape:

```clojure
{:kind :fn
 :inputs [:request]
 :output [:promise :response]
 :effects #{:suspend}}
```

## Runtime invariants

Awaiting a pending promise saves the complete frame stack, instruction pointer,
operand and local slots, namespace and dynamic bindings, exception handlers,
pending promise, resume target, and cancellation state. Settlement queues a
resume operation; it MUST NOT recursively re-enter a running VM.

An async call creates a child fiber and a stable result promise, drives the child
immediately, and returns the promise to its caller. Synchronous completion
settles the result promise without retaining host-visible state. The embedding
runtime privately retains a pending child, cancellation ownership, and active
host calls; request/response APIs expose only promises.

Rejected and cancelled awaits enter ordinary exception unwinding, including
`try`, `catch`, and `finally`.
