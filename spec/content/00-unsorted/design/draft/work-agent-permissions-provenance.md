# Work-agent permissions and provenance

Status: **design draft**

## Intent

`Work` should remain a small, portable algebra for describing execution. Agent
and tool code should be able to compose that algebra without quietly acquiring
authority from the composition itself.

This document proposes a layer above the Work algebra that answers two
different questions:

1. **Permissions:** may this principal, in this run, perform this operation?
2. **Provenance:** what plan, identity, input, delegation, implementation, and
   decision produced this operation and its result?

The layer is deliberately separate from the plan. A plan may declare what it
requires, but the executor grants authority and records evidence. Loading an
agent, resolving a target, or decoding a plan must not grant permission.

## Current seam

The current `work.agent` port has the necessary composition boundary:

```text
agent/drive
  -> work.core bundle
       :work/plan       portable Work algebra
       :work/targets    process-local target functions
```

`work.core/prepare` creates a fresh native registry and runtime. During
preparation, `work.core/bind-targets` installs local targets. A continuation may
return another Work bundle; its targets are promoted into the same registry and
only its portable plan continues through the native evaluator.

This means permissions and provenance have three natural enforcement points:

1. **Composition:** inspect and annotate a bundle before it is prepared.
2. **Admission:** validate the requested authority before targets are bound.
3. **Execution:** emit evidence and re-check authority at each effectful
   boundary, including dynamically promoted child bundles.

The current implementation does not yet enforce this layer. The structures and
rules below are the proposed contract.

## Design principles

### The algebra is authority-neutral

`Work/pure`, `Work/step`, `Work/bind`, `Work/chain`, `Work/each`, and related
constructors describe computation. They do not contain credentials or confer
authority.

A plan can carry a declaration such as `:permission/required`, but that is a
request, not a grant. The host policy remains authoritative.

### Deny by default

An operation with no matching grant is rejected before its target is invoked.
Unknown capabilities, unknown target identities, malformed provenance, expired
delegations, and unverifiable digests fail closed.

### Authority only narrows through delegation

A child agent, nested tool, or dynamically returned bundle receives an
attenuated grant derived from its parent invocation. It may not add a new
capability merely by returning a Work bundle.

The effective grant is conceptually:

```text
child grant = parent grant ∩ child request ∩ host policy
```

If a parent has no authority for an operation, it cannot delegate that
authority to another agent.

### Provenance is evidence, not authority

An agent may report its identity, source, model, or intended purpose, but those
claims do not authenticate it. Trusted identity, package, artifact, and host
records must be resolved outside the untrusted plan and attached as verified
evidence.

### Portable data, local authority

The portable plan and provenance descriptors contain ordinary data and stable
identifiers only. They must not contain API keys, sockets, native handles,
closures, classloaders, or other live authority. Process-local targets are
resolved separately through an admitted registry.

## Proposed bundle shape

The existing bundle remains the execution boundary. A future policy layer may
add a manifest beside the plan and targets:

```clojure
{:work/plan <canonical Work plan>
 :work/targets <process-local target table>
 :work/manifest
 {:work/id "workflow/assessment"
  :work/plan-digest "sha256:..."
  :work/entrypoint "workflow/start"
  :work/requirements
  [{:work/target "agent/research/model"
    :permission/required #{:network-connect}
    :effect :model}
   {:work/target "tool/publish"
    :permission/required #{:external-write}
    :effect :external}]
  :work/provenance
  {:source {:repository "..."
            :commit "..."}
   :builder {:id "..."
             :digest "sha256:..."}
   :parents []}
  :work/delegation
  {:principal/id "supervisor"
   :purpose "assessment"
   :expires-at "..."}}}
```

The exact schema is open. The important distinction is that `:work/manifest`
declares requirements and lineage; it does not grant `:network-connect` or
`:external-write`.

## Permission model

Permissions should be expressed as stable capabilities rather than target
names. Target names identify implementation points; capabilities identify the
authority being requested.

Examples include:

```text
:model-invoke       call a configured model provider
:network-connect    connect to an approved network endpoint
:filesystem-read    read an approved path or object
:filesystem-write   write an approved path or object
:external-write     perform an irreversible external mutation
:agent-delegate     invoke a child agent
:authority-decide   issue an approval or signature
```

A grant should bind more than a capability label. It should normally include:

```clojure
{:permission/capability :network-connect
 :permission/resource "api.openai.com"
 :permission/actions #{:request}
 :permission/principal "supervisor"
 :permission/purpose "assessment"
 :permission/parent-run "run/123"
 :permission/expires-at "..."
 :permission/id "grant/456"}
```

Resource, action, principal, purpose, parent run, and expiry constraints prevent
a broad capability from becoming ambient authority.

### Effect classes

The agent/tool effect classification should map to permission requirements:

| Effect | Default meaning | Typical requirement |
| --- | --- | --- |
| `:pure` | deterministic local transformation | none |
| `:model` | model/provider interaction | `:model-invoke` and usually `:network-connect` |
| `:delegation` | invocation of another agent | `:agent-delegate` |
| `:external` | mutation outside the Work runtime | resource-specific write capability |
| `:authority` | approval, signature, or policy decision | `:authority-decide` |

The current `work.agent/tool` already distinguishes `:pure` from non-pure
tools. The policy layer should make that declaration checked metadata rather
than relying on the tool implementation to self-report honestly.

## Provenance model

Provenance should be recorded at both the plan level and the execution level.

### Plan provenance

Plan provenance identifies the immutable or reproducible inputs used to create a
bundle:

```clojure
{:provenance/plan
 {:plan/digest "sha256:..."
  :plan/format "work-v1"
  :source [{:repository "..."
            :commit "..."
            :path "src/..."}]
  :dependencies [{:package "..."
                  :version "..."
                  :digest "sha256:..."}]
  :builder {:id "..."
            :version "..."
            :digest "sha256:..."}
  :parent-plans ["sha256:..."]}}
```

The digest must be computed over canonical portable data. Process-local target
functions are represented by stable target identities and implementation
digests, never serialized as closures.

### Run provenance

Each evaluation or submission receives a run identity and emits an ordered
journal:

```clojure
{:run/id "run/123"
 :run/parent "run/122"
 :run/principal "supervisor"
 :run/plan-digest "sha256:..."
 :run/input-digest "sha256:..."
 :run/events
 [{:event/id "event/1"
   :event/parent nil
   :event/target "agent/research/model"
   :event/effect :model
   :event/required #{:model-invoke}
   :event/grants ["grant/456"]
   :event/input-digest "sha256:..."
   :event/output-digest "sha256:..."
   :event/status :ok}
  {:event/id "event/2"
   :event/parent "event/1"
   :event/target "tool/publish"
   :event/effect :external
   :event/status :denied
   :event/error :permission/denied}]}
```

The journal should form a hash-linked or otherwise tamper-evident sequence.
Inputs and outputs should be summarized by digest by default; sensitive values
must not be copied into logs merely to make a trace convenient.

## Multi-agent delegation

The intended supervisor flow is:

```text
principal
  -> supervisor agent
       -> delegated research agent
       -> delegated critic agent
       -> supervisor synthesis
       -> authority gate
       -> external action
```

An effectful delegation tool may return `agent/drive` for a child agent. Before
the child bundle is promoted into the parent runtime, the policy layer should:

1. identify the parent run and delegating principal;
2. resolve the child agent and its verified implementation provenance;
3. calculate the child capability intersection;
4. attach a delegation record with purpose, scope, expiry, and parent event;
5. admit the child plan and targets;
6. record the child result as an event linked to the delegation.

This preserves the current dynamic Work composition while preventing dynamic
target promotion from becoming an authority escalation path.

## Concrete example: governed release assessment

Consider the same supervisor workflow, with one additional requirement: the
agents may assess a release, but no model may publish it directly.

The request is:

```text
Should release 42 be published?
```

The intended execution is:

```text
supervisor
  -> research specialist: read release notes and issue tracker
  -> critic specialist: identify risks in the research
  -> supervisor: produce a recommendation
  -> release board: approve or reject publication
  -> publisher: publish only an approved release
```

### Grants

The host policy grants the following scopes:

| Principal | Capabilities | Resource constraint | Can do |
| --- | --- | --- | --- |
| `:supervisor` | `:model-invoke`, `:agent-delegate` | assessment 42 | reason and delegate |
| `:researcher` | `:model-invoke`, `:network-connect` | `docs.example.com` | inspect approved sources |
| `:critic` | `:model-invoke` | none | review supplied material |
| `:release-board` | `:authority-decide` | release 42 | approve or reject |
| `:publisher` | `:external-write` | release 42 | publish after approval |

The supervisor has no `:external-write` grant. Even if the model asks for a
`publish` tool, admission must reject it. The publisher receives its write
capability only in a separately admitted, approval-bound invocation.

### Agent and tool declarations

The following is illustrative HAL using the current `work.agent` surface plus
the proposed policy metadata. `governed-tool` is the future admission wrapper;
the existing `agent/tool` already preserves additional tool fields in its
normalized value.

```clojure
(ns release.assessment
  (:require [work.core :as work]
            [work.agent :as agent]))

(def researcher
  {:agent/id :researcher
   :agent/driver
   (agent/openai-driver
    {:client client
     :instructions
     "Read only the approved sources and return concise findings."})})

(def critic
  {:agent/id :critic
   :agent/driver
   (agent/openai-driver
    {:client client
     :instructions
     "Review the supplied research and identify release risks."})})

(defn governed-tool
  [tool principal required purpose]
  (assoc tool
         :permission/principal principal
         :permission/required required
         :permission/purpose purpose))

(def research-tool
  (governed-tool
   (agent/tool
    "research"
    "Ask the research specialist to inspect approved sources."
    {:type "object"
     :properties {"task" {:type "string"}}
     :required ["task"]}
    (fn [arguments]
      (agent/drive researcher (get arguments "task")))
    {:effect :delegation})
   :supervisor
   [:agent-delegate]
   "release-assessment-42"))

(def critic-tool
  (governed-tool
   (agent/tool
    "critic"
    "Ask the critic specialist to review research."
    {:type "object"
     :properties {"task" {:type "string"}}
     :required ["task"]}
    (fn [arguments]
      (agent/drive critic (get arguments "task")))
    {:effect :delegation})
   :supervisor
   [:agent-delegate]
   "release-assessment-42"))

(def supervisor
  {:agent/id :supervisor
   :agent/driver
   (agent/openai-driver
    {:client client
     :instructions
     "Assess release 42. Delegate research and criticism, then state a recommendation. Never publish."
     :tools [research-tool critic-tool]})})

(def assessment
  (agent/drive supervisor "Should release 42 be published?"))

(deref (work/evaluate assessment nil))
```

The two delegation tools return child Work bundles. Current Work bundle
promotion makes those children executable in the same runtime. Policy admission
adds the missing checks:

```clojure
{:delegation/parent :supervisor
 :delegation/child :researcher
 :delegation/purpose "release-assessment-42"
 :delegation/requested #{:model-invoke :network-connect}
 :delegation/resource "docs.example.com"
 :delegation/effective #{:model-invoke :network-connect}
 :delegation/parent-event "event/supervisor-research"}
```

The effective grant is allowed only because the host policy explicitly permits
the researcher scope and the supervisor is allowed to delegate for this
purpose. The child cannot turn that grant into `:external-write`.

### Approval-bound publication

Publication is a separate Work boundary. It is not another tool exposed to the
supervisor's model:

```clojure
(def approval-request
  {:action :publish
   :release "42"
   :assessment assessment
   :assessment-digest "sha256:..."
   :plan-digest "sha256:..."
   :policy-revision "release-policy-v3"})

(def approval
  (agent/authorize
   {:agent/id :release-board
    :agent/authority release-board-authority}
   approval-request))
```

The host admits `approval` under `:authority-decide`. If the board approves,
the publisher receives a narrow, one-time grant:

```clojure
{:permission/capability :external-write
 :permission/resource "release/42"
 :permission/principal :publisher
 :permission/purpose "publish-approved-release-42"
 :permission/approval "approval/789"
 :permission/plan-digest "sha256:..."
 :permission/input-digest "sha256:..."
 :permission/expires-at "..."}
```

The publisher's Work bundle is then admitted and submitted. If the assessment
changes, the plan or input digest changes, or the approval expires, the same
publish target is denied and is never invoked.

### Resulting provenance

A successful journal would contain a linked sequence similar to:

```clojure
[{:event/id "event/1"
  :event/target "work.agent.openai/:supervisor/model"
  :event/principal :supervisor
  :event/required #{:model-invoke}
  :event/grants ["grant/supervisor"]
  :event/status :ok}
 {:event/id "event/2"
  :event/parent "event/1"
  :event/target "tool/research"
  :event/principal :supervisor
  :event/required #{:agent-delegate}
  :event/child-run "run/researcher"
  :event/status :ok}
 {:event/id "event/3"
  :event/parent "event/2"
  :event/target "work.agent.openai/:researcher/model"
  :event/principal :researcher
  :event/required #{:model-invoke :network-connect}
  :event/resource "docs.example.com"
  :event/delegation "event/2"
  :event/status :ok}
 {:event/id "event/4"
  :event/parent "event/1"
  :event/target "tool/critic"
  :event/principal :supervisor
  :event/child-run "run/critic"
  :event/status :ok}
 {:event/id "event/5"
  :event/parent "event/1"
  :event/target "authority/publish"
  :event/principal :release-board
  :event/required #{:authority-decide}
  :event/approval "approval/789"
  :event/status :ok}
 {:event/id "event/6"
  :event/parent "event/5"
  :event/target "release/publish"
  :event/principal :publisher
  :event/required #{:external-write}
  :event/resource "release/42"
  :event/approval "approval/789"
  :event/status :ok}]
```

If the supervisor attempts to publish directly, the evidence should instead be:

```clojure
{:event/target "release/publish"
 :event/principal :supervisor
 :event/required #{:external-write}
 :event/granted #{:model-invoke :agent-delegate}
 :event/status :denied
 :event/error :permission/denied}
```

There is no target invocation, external request, or partial publication in the
denied case. This is the key distinction between a provenance log added after
execution and a permission layer that governs the algebra.

### What is current and what is proposed

The example uses current behavior for:

- `agent/drive` returning a Work definition bundle;
- effectful tools returning child bundles;
- dynamic target promotion during `Work/bind`;
- `work/evaluate` or `work/submit` as the execution boundary;
- `agent/authorize` as an authority-shaped Work step.

The following pieces are proposed and need implementation:

- `:permission/*` and `:delegation/*` declaration validation;
- policy admission before target binding;
- attenuation of child grants;
- per-target execution checks;
- run/event identities and digest computation;
- approval-bound publisher grants;
- tamper-evident journal storage and final attestations.

## Algebra-level rules

The policy layer should attach rules to the existing Work operations rather than
replace them.

| Work operation | Policy/provenance rule |
| --- | --- |
| `Work/pure` | Requires no external capability; records input/output lineage when evaluated. |
| `Work/step` | Resolves a stable target descriptor and checks its declared effect and capabilities before invocation. |
| `Work/bind` | Carries the same run context into the continuation; a returned bundle is admitted before its plan executes. |
| `Work/chain` | Preserves ordered parent/child lineage across each composed bundle. |
| `Work/each` | Gives every element the parent grant and records element identity/order. |
| `Work/choose` | Records the selected branch and the predicate/decision provenance. |
| `Work/await` | Records the awaited resource and verifies that resumption uses the same run/delegation scope. |
| `Work/submit` | Creates the host-managed run boundary, cancellation identity, journal, and final attestation. |

The evaluator should not silently continue after a policy failure. A denied
operation is a structured terminal result with the target, required
capabilities, principal, run, and policy revision identified.

## Authority gates

`agent/authorize` and `agent/sign` are useful boundaries for operations that
should not be decided solely by a model. A typical release flow is:

```text
research -> critique -> proposed action
          -> authorize(action)
             -> if approved: publish(action)
             -> otherwise: terminal denial
```

The approval should be bound to at least:

```text
principal + action + normalized input digest + plan digest + policy revision
```

An approval for one action or one input must not be replayable for another.
Signatures should be treated as attestations over those bindings, not as a
general-purpose capability that can be detached from its purpose.

## Admission and execution lifecycle

The proposed lifecycle is:

```text
compose
  -> canonicalize plan and input
  -> compute plan/input digests
  -> resolve implementation provenance
  -> derive requested capabilities
  -> host policy admission
  -> bind admitted local targets
  -> execute with run context
  -> admit dynamic child bundles before continuation
  -> emit journal and final attestation
  -> reset runtime and close authority handles
```

Admission must happen before invoking a target. Execution-time checks remain
necessary because a `Work/bind` continuation can construct a new bundle from a
runtime value.

`work.core/reset` should remain the deterministic cleanup boundary. A prepared
runtime, its registry, delegation records, and in-memory journal should be
resettable or closable after success, failure, cancellation, or partial
initialization.

## Recommended API direction

The first public layer can remain small:

```clojure
(work/manifest bundle)
(work/requirements bundle)
(work/admit bundle principal policy)
(work/prepare admitted-bundle)
(work/run-context prepared)
(work/journal run)
(work/attestation run)
```

The implementation should keep policy resolution and journal storage behind
explicit interfaces. A policy provider may be local, database-backed, or host
managed, but the Work evaluator should receive a concrete admitted context and
never reach for ambient credentials.

The likely internal boundaries are:

- `work.agent`: agent identity, delegation, tools, and agent-level declared
  requirements;
- `work.core`: bundle manifests, admission, target descriptors, run context,
  and reset lifecycle;
- native `Work`: execution, cancellation, deadlines, and host-managed runs;
- identity/package/artifact layers: verified principals, source provenance,
  package digests, and attestations.

## Security invariants

A conforming implementation should preserve these invariants:

1. A plan never contains credentials or live host authority.
2. Loading a plan, agent, tool, or target grants no permission.
3. Every non-pure target has a declared effect and required capability set.
4. Every effectful invocation is checked against the active principal, resource,
   purpose, policy revision, and expiry.
5. A child bundle cannot widen its parent grant.
6. Dynamic target promotion cannot bypass admission.
7. Provenance links every result to a plan digest, input digest, implementation
   identity, parent event, and policy decision.
8. Approval and signature records are bound to the exact action and input.
9. Denials happen before target invocation and are observable as structured
   evidence.
10. Reset and close restore the runtime and authority state even after failure.

## Initial acceptance criteria

The first implementation slice should demonstrate:

- a pure multi-agent workflow with a complete plan and provenance digest;
- a delegated child agent whose grant is narrower than its supervisor's grant;
- rejection of an undeclared or expired model/tool capability before transport
  invocation;
- rejection of a dynamically returned bundle that requests broader authority;
- an authority gate whose approval is bound to the plan and input digests;
- a journal containing ordered parent/child events and structured denials;
- no credentials or closures in the encoded plan or provenance record;
- plan/provenance round trips and idempotent runtime reset;
- a final attestation that can be independently verified from the recorded
  evidence.

## Relationship to existing specifications

This design should reuse, rather than duplicate, the existing platform trust
boundaries:

- [native capability and host boundary](../../../01-lang/003-native/draft/native-spec.edn)
  for host-facing capability vocabulary;
- [identity](../../../02-platform/000003-identity/draft/hara-identity.edn)
  for principals, delegation, rotation, and revocation;
- [immutable artifacts](../../../02-platform/000004-artifact/draft/hara-artifact.edn)
  for digests and attestations;
- [packages](../../../02-platform/000006-package/draft/hara-package.edn)
  for source and implementation provenance;
- [extensions](../../../02-platform/000007-extension/draft/hara-extension.edn)
  for declared host capabilities and isolated providers.

The Work-agent layer should describe execution requirements and lineage. It
should not become a second identity, package-signing, or host-capability system.
