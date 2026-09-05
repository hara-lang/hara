# Work Bind Continuation

Status: implementation contract for #880. Compiler equivalence is tracked by #808.

## Purpose

`work/bind` is the canonical Work operation for cases where one Work result
determines the next Work value at runtime.

```hara
(work/bind source continuation)
(work/bind options source continuation)
```

The normalized shape is:

```hara
{:op :bind
 :id stable-id
 :source source-work
 :continuation pure-continuation-work
 :maximum-depth optional-positive-integer}
```

It is a structural Work operation. It adds no protocol, host, run, store, or
agent-specific execution path.

## Evaluation law

For input `x`:

```text
v    = execute(source, x)
next = execute(continuation, v)
assert next satisfies IWork
r    = execute(next, v)
```

`continuation` must itself be `:pure` Work. It may select or construct the next
Work value but may not hide another effect while making that decision.

The produced Work executes with the resolved source value as its input and
inherits the current Runtime, run identity, options, authority, cancellation,
deadline, event, checkpoint, and receipt ownership.

## Child identity

The evaluator assigns deterministic child paths:

```text
:bind / :source       / <source-id>
:bind / :continuation / <continuation-id>
:bind / :produced     / <produced-id>
```

No checkpoint belongs to the bind node itself. Ordinary step checkpoints under
source and produced Work remain authoritative.

## Durable replay

For durable execution, runtime-produced Work must have an explicit stable
`:id`. Anonymous produced Work is rejected before it executes.

On resume:

- source steps replay through their existing checkpoints;
- the pure continuation is recomputed;
- it reconstructs the produced Work;
- completed produced-child steps replay by the same deterministic paths;
- only unfinished effect boundaries execute again.

Executable Work values are therefore never persisted as checkpoint results.

## Composition laws

`bind` does not special-case the produced subtree. The recursive evaluator must
preserve every existing Work behavior that the produced value uses.

### Host and Runtime ownership

The default ephemeral `IWorkHost` routes bind through the canonical evaluator.
Runtime-produced Work does not bypass the host's execution adapter or start a
nested host/run.

Produced operations resolve custom structural handlers from the same Runtime
hook table. A dynamically produced extension operation therefore has the same
handler authority and deterministic node lineage as the same static operation.

### Cancellation and deadlines

Every source, continuation, and produced child enters the ordinary
`execute-node` boundary. Cancellation is checked there before the next child is
entered. If the source cancels the current run, neither the continuation nor a
produced effect may execute.

Produced step requests inherit the ordinary leaf envelope, including current
run/root identity, user Work context, item identity, deadline, retry attempt,
and executor authority.

### Cleanup

A produced `ensure` retains ordinary unwinding. Body failure or cancellation
still runs its cleanup under `:work/cleanup?`; bind neither swallows the failure
nor owns a second cleanup policy.

### Collections

A produced collection operation retains normal manifest/item identity. Durable
`each` checkpoints continue to distinguish items through `:item/id`, and resume
replays completed items without re-running their effects.

### Forks

Fork checkpoint copying uses the normal produced-child checkpoint paths. Forking
before a selected step inside a dynamic subtree copies the earlier source and
produced checkpoints and then executes from the selected boundary. No bind
checkpoint or bind-specific fork state exists.

### Async results

Promise-returning source and produced leaves use the existing result
normalization. Managed checkpoints are committed only after those leaf results
resolve, exactly as for static Work.

### Receipts

Receipt/outbox intent remains a consequence of ordinary run and checkpoint
transitions. A bind containing two checkpointed steps emits the same
run-created, run-started, checkpoint, checkpoint, and run-completed intent
families as an equivalent static flow. There is no bind-specific receipt type.

## Dynamic depth

Runtime production is bounded to prevent unbounded dynamic Work expansion.

- default maximum depth: `64`;
- `:maximum-depth` must be a positive integer when supplied;
- the strictest active maximum is inherited by produced descendants;
- depth increments only while entering a `:bind/:produced` child;
- the next produced Work is rejected before execution when the bound is
  exceeded.

## Failures

The portable failure identities are:

```text
:work/bind-source-not-work
:work/bind-continuation-not-work
:work/bind-continuation-not-pure
:work/bind-not-work
:work/bind-invalid-depth
:work/bind-unstable-produced
:work/bind-depth-exceeded
```

Diagnostics may include operation, node path, stable IDs, and configured depth.
They must not embed provider handles or rejected executable values.

## Agent use

`work.agent` is the first consumer. An `IAgentDriver` may model each provider
turn as a checkpointed Work step, then use a pure bind continuation to select
ordinary tool Work and the next provider turn.

Pure tools may execute as pure Work. Non-pure tools are replay-safe Work
factories: model selection invokes the factory only to construct stable
`IWork`, and the Work runtime executes the resulting effect. Direct tool
invocation continues to reject non-pure tools.

This keeps the agent API unchanged:

```text
IAgent
IAgentDriver
IAgentCoordinator
IAgentAuthority
```

Everything executable remains Work.

## Compiler parity

`:bind` is part of the canonical Work algebra. #808 must lower the same source,
pure-continuation, produced-child, identity, depth, replay, and composition
semantics to ordinary Hara forms. No compiler-specific continuation engine is
permitted.
