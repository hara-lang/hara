# Historical Foundation review imports

`index.edn` moves the earlier Clojure-core compatibility review and the
completed `baa75a` external-dependency review into the current
specifications-owned conformance layout.

The import is **review evidence**, not a current Hara contract and not runtime
status. It preserves immutable source coordinates, historical classifications,
semantic overrides, disposition counts, and the reusable `clojure.string`
mapping. It deliberately does not copy historical conclusions directly into a
current family contract.

Promotion follows this sequence:

```text
historical review import
  + current f55f family origin observation
  + current Hara owner and specification evidence
  -> reviewed family conformance disposition
  -> execution by every applicable registry-declared adapter
  -> published runtime status
```

For `clojure.core`, an imported `same-exact`, renamed, or changed classification
is a candidate disposition. The consuming family still retains the exact source
reference, `:refer-clojure` context, shadowing result, and behavioral case.

For an external namespace route, an unchanged Foundation source blob may carry
the historical review forward for current review. A changed blob remains
historical evidence only. Reader objects, JVM classes, Vars, closures, and other
host-owned values remain manual or capability-specific unless a portable
contract is proven.

The active family publications remain under `../libraries/`. Their
`origin.edn` files own immutable Foundation observations, `metadata.edn` owns
pins and generated identities, and `conformance.edn` owns accepted Hara
dispositions. Runtime reports are separate.
