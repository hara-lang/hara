# Work Capability ABI

Status: implementation contract for the `work.*` algebra migration tracked by
#803 and #804.

## Purpose

The Work kernel separates immutable computation descriptions from execution,
persistence, and live process ownership.

```text
IWork
  describes Work
       |
       v
Work evaluator or compiler
       |
       +--------------------+
       |                    |
       v                    v
IWorkExecutor            IWorkStore
execute one leaf         query and journal state
       |                    |
       +----------+---------+
                  v
            Runtime value
                  |
                  v
      optional IWorkHost / IWorkRun
```

The evaluator owns the structural meaning of Work operations. Executors never
interpret Work composition, and stores never execute Work.

## Native protocol family

The public native Work family is:

```text
IWork
IWorkExecutor
IWorkStore
IWorkRef
IWorkRun
IWorkHost
```

No `IWorkRuntime` or `IWorkMachine` protocol is defined. Runtime configuration
is ordinary Hara data.

## IWorkExecutor

```clojure
(defprotocol IWorkExecutor
  (work-execute [executor request]))
```

`work-execute` performs one leaf request. It may return a direct value or an
`IPromise`.

A leaf request is a map. The stable vocabulary is expected to include:

```clojure
{:run/id optional-run-id
 :work/root work-root
 :work/boundary :pure-or-step
 :node/id node-id
 :node/path node-path
 :item/id optional-item-id
 :work/target target
 :work/input input
 :work/context context
 :work/attempt attempt
 :work/deadline optional-deadline}
```

This initial ABI slice defines the protocol identity and arity. The canonical
request validation and target profiles are introduced with the store-free Work
evaluator.

`IWorkExecutor` does not extend `IComponent`. A concrete process pool, remote
worker, or sandbox executor may separately implement lifecycle protocols.

## IWorkStore

```clojure
(defprotocol IWorkStore
  (work-query [store query])
  (work-transact [store transition]))
```

`work-query` performs a typed read. The baseline managed-execution query family
covers:

```text
run load and list
committed event history
checkpoint load and list
```

A query is an immutable map whose discriminator is supplied by
`:work/query`. Existing provider operation maps remain compatible until the
store-adapter migration establishes the canonical query profiles.

`work-transact` applies one revision-fenced journal transition. A transition
may atomically contain:

```text
run creation or updates
checkpoint commits
committed events
```

The store must preserve these laws:

- a failed expected-revision check performs no partial writes;
- a checkpoint identity cannot be committed with a different value;
- a checkpoint and its corresponding completion event commit atomically;
- observers see only events returned by the store as committed;
- committed event order is stable for one run.

Transactional outbox, claims, leases, delayed scheduling, receipt publication,
and distributed fencing are optional capability suites rather than mandatory
`IWorkStore` methods.

`IWorkStore` does not extend `IComponent`. A concrete database client may
separately implement lifecycle protocols.

## Runtime

Runtime is a reusable immutable map or record that assembles capabilities and
policy:

```clojure
{:work/executor executor
 :work/store store-or-nil
 :work/registry registry-or-nil
 :work/policy policy-map
 :work/hooks hooks-map}
```

It has no native protocol and owns no thread, process, stream, run identity, or
cancellation lifecycle.

Bare evaluation requires an executor and may omit the store. Managed execution
adds store-backed step replay and journal transitions. The same Runtime value
may be reused across many live runs.

## Host boundary

`IWorkHost`, `IWorkRun`, and `IWorkRef` remain orthogonal to the evaluator,
executor, and store ABIs.

A host owns live concerns:

```text
run identity
status and asynchronous result
cancellation and deadlines
structured child runs
live event streams
finalisation
```

The native host accepts an execution adapter. It does not define the meaning of
Work operations and does not become a store.

## Compatibility phase

During the migration:

- existing executor and store provider descriptors remain accepted;
- existing Work maps and operation keywords retain their shapes;
- existing run, event, checkpoint, and PostgreSQL record shapes remain stable;
- adapters bridge provider operation maps to the native capability protocols;
- Runtime constructors may continue returning the existing struct while also
  accepting the canonical map contract.
