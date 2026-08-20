# Minimal Work Agent API

Status: implementation contract for #861, #862, #863, with dynamic Work
continuation under #880.

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
`:all`, `:bind`, graph, and managed Work. No `:agent/*` structural evaluator
operation is introduced.

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

Pure and effectful tools share one model-visible schema but have different
execution laws.

A `:tool/effect :pure` implementation receives decoded arguments and may return
a direct value or Promise. The driver lowers the call to pure Work and keeps the
existing structured tool-result behavior for success, unknown tools, and pure
exceptions.

A non-pure tool implementation is a Work factory. The model may select it, but
direct invocation still returns `:tool/requires-work`. The Work-aware driver
invokes the function only to construct an explicitly identified `IWork` value.
That Work then executes through the canonical evaluator, executor/store, host,
checkpoint, cancellation, event, and receipt boundaries.

A non-pure tool factory that returns a non-Work value fails with
`:work/agent-tool-not-work`. Effectful Work without an explicit stable `:id`
fails with `:work/agent-tool-unstable-work` before execution.

## Dynamic model/tool continuation

#880 supplies the generic `work/bind` operation used by provider drivers.
A provider interaction can therefore be represented as ordinary Work:

```text
model step
  -> normalized provider response
  -> pure bind continuation
       -> final-result Work
       -> or tool Work
            -> pure continuation
            -> next model Work
```

Each model request is a normal `:step`. On durable resume, a completed model
request replays from its checkpoint rather than repeating the provider call.
Completed effectful tool steps replay the same way. The pure continuation is
recomputed and reconstructs the same identified produced Work subtree.

This is not an agent lifecycle. The same run/root/version and Work frame lineage
remain authoritative throughout the interaction.

## Ephemeral provider

`work.agent.ephemeral` provides in-memory reference implementations of the
agent capability protocols. Its mutable coordination state is provider-local
only; Work remains responsible for run lifecycle and durability.

## OpenAI driver

`work.agent.driver.openai` retains provider-neutral OpenAI Responses and tool
translation code but implements `IAgentDriver`.

Driving an intent returns a `:bind` Work tree. Each OpenAI Responses request is
an identified Work step. A pure bind continuation inspects the normalized
response. Final responses lower to pure result Work. Function calls lower to an
identified `:all` of tool Work, followed by another bind that constructs the
next model turn.

Pure tools remain lightweight. Effectful tool factories produce ordinary Work
and are never directly executed by the tool dispatcher.
