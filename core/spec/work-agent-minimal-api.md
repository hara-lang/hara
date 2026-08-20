# Minimal Work Agent API

Status: implementation contract for #861, #862, and #863.

## Model

```text
Agent
  = ordinary map

IAgent
  = projects a protocol-backed value to an Agent map

IAgentDriver
  = turns intent into Work

IAgentCoordinator
  = coordinates agents and tasks

IAgentAuthority
  = authorizes and signs actions

Everything else
  = Work
```

Agent is data. Runtime remains Work capability composition. Providers are
replaceable, and protocols exist only at capability boundaries.

A minimal agent is:

```clojure
{:agent/id :compiler
 :agent/driver compiler-driver}
```

A richer agent composes capabilities without changing the algebra:

```clojure
{:agent/id :chief-research-agent
 :agent/driver llm-agent-driver
 :agent/coordinator ignatius
 :agent/authority hestia
 :agent/capabilities #{...}}
```

## Protocols

The public agent capability family is exactly:

```text
IAgent
IAgentDriver
IAgentCoordinator
IAgentAuthority
```

`IAgent/agent-spec` returns the canonical Agent map.
`IAgentDriver/agent-drive` returns an `IWork` value and never executes it.
Coordinator and authority requests/results are ordinary immutable Hara values.

There is no agent runtime, store, host, run, ref, observer, machine, or second
evaluator algebra.

## Lowering

`work.agent` is a frontend over canonical Work. Driver output and helper
operations lower immediately to existing Work forms such as `:step`, `:chain`,
`:all`, graph, and managed Work. No `:agent/*` structural evaluator operation
is introduced.

Live identity, status, results, cancellation, events, checkpoints, retries,
persistence, and execution remain owned by the existing Work family:

```text
IWork
IWorkExecutor
IWorkStore
IWorkHost
IWorkRun
IWorkRef
```

## Tool effect boundary

An agent driver may invoke a tool callable in-process only when the tool is
explicitly `:tool/effect :pure`. Pure tools share the enclosing driver leaf and
do not create another lifecycle.

A non-pure tool must not be invoked directly inside the driver leaf. Doing so
would hide an external side effect inside the enclosing step's retry and
checkpoint boundary, allowing that effect to be repeated when the outer step
is retried or resumed.

Until #880 provides generic runtime-produced child Work, non-pure tools are
rejected before model exposure and by the direct tool dispatcher. The rejected
callable is never executed. #880 belongs to Work itself: it must allow a
dynamically selected effect to lower to ordinary child `IWork` in the same run
with normal checkpoint, cancellation, event, and receipt semantics. It must not
add another agent protocol or an `:agent/*` evaluator operation.

## Ephemeral provider

`work.agent.ephemeral` provides in-memory reference implementations of the
agent capability protocols. Its mutable coordination state is provider-local
only; Work remains responsible for run lifecycle and durability.

## OpenAI driver

`work.agent.driver.openai` retains the provider-neutral OpenAI Responses and
tool translation code but implements `IAgentDriver`. Driving an intent
returns ordinary Work; model interaction and pure tool calls execute only when
that Work is run by the canonical Work runtime/host. Effectful tool calls remain
explicit Work concerns under #880 rather than hidden driver side effects.
