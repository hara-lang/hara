# Minimal Work Agent API

Status: implementation contract for #861, #862, and #863.

## Model

```text
Agent
  = ordinary map

IAgent
  = projects a protocol-backed value to an Agent map

IWorkAgentDriver
  = turns intent into Work

IWorkAgentCoordinator
  = coordinates agents and tasks

IWorkAgentAuthority
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
IWorkAgentDriver
IWorkAgentCoordinator
IWorkAgentAuthority
```

`IAgent/agent-spec` returns the canonical Agent map.
`IWorkAgentDriver/agent-drive` returns an `IWork` value and never executes it.
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

## Ephemeral provider

`work.agent.ephemeral` provides in-memory reference implementations of the
agent capability protocols. Its mutable coordination state is provider-local
only; Work remains responsible for run lifecycle and durability.

## OpenAI driver

`work.agent.driver.openai` retains the provider-neutral OpenAI Responses and
tool translation code but implements `IWorkAgentDriver`. Driving an intent
returns ordinary Work; model/tool effects execute only when that Work is run by
the canonical Work runtime/host.
