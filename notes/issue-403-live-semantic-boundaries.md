# Live semantic evaluator boundaries

Issue: #403

This slice attaches semantic evidence to the opt-in live `EvalFiber` without
introducing another evaluator or replaying an Evaluation Journal.

## Production seam

The existing `forms_cps` and `values_cps` continuation producers already know
which form completed, the returned runtime value, and the lexical environment
that produced it. While an observed fiber is actively executing, they record
that evidence in the fiber's observation context immediately before returning
the existing `Step::Continue` trampoline.

The ordinary `EvalFiber::start` path keeps the same `Step` shape and evaluation
flow. When no observation context is active, recording returns without cloning
the form, result, or environment.

## Portable projection

The bounded live snapshot adds a semantic boundary containing:

- a monotonic sequence number;
- `form/return` or `value/return` producer identity;
- canonical form display and semantic form kind;
- the actual returned value through the existing safe value projection;
- a bounded current lexical frame captured before scope restoration;
- the bounded session environment after the step;
- exact reader form path and byte/line/column span when the completed form has
  one unambiguous match in the original `SpannedForm` tree.

Repeated structurally identical source forms are reported as ambiguous rather
than assigned a guessed path. Synthetic and expanded forms that do not occur in
the reader tree remain explicit unresolved source references. Expansion-origin
metadata is a later slice.

## Initial proof

For:

```hara
(+ 1 (* 2 3))
```

the live boundaries identify `(* 2 3)` at path `[0 2]`, retain its exact source
span, and project its actual result `6` before the outer form returns `7`.

A lexical fixture verifies that resolving `x` inside:

```hara
(let [x 41] (+ x 1))
```

captures the current frame containing `x = 41` even though `let` restores the
session environment before the host inspects the boundary.

## Remaining work

This slice does not yet add macroexpansion origin, explicit namespace/Var
mutation events, call-stack labels beyond captured lexical frames, or bounded
browser session history. Those remain follow-on #403 work over the same live
boundary context.
