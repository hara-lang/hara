# Hara project and package contract

The authoritative document is [`hara-package.edn`](hara-package.edn).
`project.edn` is the single contributor-authored manifest. Reconciliation
generates `project.lock.edn`; package building generates the root `package.edn`
inside a deterministic `.harp`.

## Wasm imports and host flavors

Package metadata separates portable Wasm modules from host-native artifacts.
`:wasm-imports` contains digest-bound direct Wasm and generated HTA modules.
Direct Wasm loads through `(:import ...)`; generated HTA loads through
`(:require ...)`. `:flavors` contains host implementations such as `:jvm` and
`:dotnet`; `:wasm` is not a flavor. Host class or type imports are written
inside the matching flavor clause, for example
`(:flavor :jvm [java.lang String])`.

## Runtime-aware projects

Projects may declare orthogonal `:project/runtime-profiles` for `:jvm` and
`:rust`. The executing host selects its own project profile automatically;
build profiles under `:project/profiles` remain an independent concern.

An effective runtime project is formed from the shared project paths and Hara
package dependencies plus the selected runtime profile's additions. JVM
profiles additionally own Maven dependencies, Java source roots, and a class
target. Rust profiles may own Hara packages that carry Wasm extensions.

The contract hard-cuts the former top-level `:jvm/source-paths`,
`:jvm/dependencies`, and `:jvm/target-path` keys. Validators report the
corresponding `:project/runtime-profiles :jvm` replacement.

## Prebuilt route artifacts

The generated `package.edn` may add route-specific prebuilt artifacts to the
existing portable HAL/HIR resources. `:wasm-imports` maps logical Wasm modules
to their artifact declarations. `:flavors` maps unqualified host flavors to
their artifacts; `:wasm` is not a flavor.

One package identity may therefore declare both a direct Wasm import and an
explicit JVM flavor:

```clojure
{:wasm-imports
 {:provider
  {:variant/artifact
   {:artifact/type :wasm
    :artifact/path "artifacts/provider.wasm"
    :artifact/sha256 "sha256:..."
    :artifact/target "wasm32-wasi-preview1"
    :artifact/abi "core.v1"
    :artifact/entry-point "provider_init"}
   :variant/required-capabilities #{}
   :variant/host-calls #{}
   :variant/exports #{:provider/open}}}
 :flavors
 {:jvm
  {:variant/artifact
   {:artifact/type :jar
    :artifact/path "artifacts/provider.jar"
    :artifact/sha256 "sha256:..."
    :artifact/target "java-21"
    :artifact/abi "hara.provider.jvm.v1"
    :artifact/entry-point "example.provider.HaraProvider"}
   :variant/required-capabilities #{:db/connect}
   :variant/host-calls #{}
   :variant/exports #{:provider/open :provider/close}
   :variant/dependencies
   {:maven {org.example/provider-runtime {:version "1.0.0"}}}}}}
```

The language loading route selects the artifact kind:

```text
(:import provider)                         -> :wasm-imports/:provider :wasm
(:require provider)                        -> :wasm-imports/:provider :hta
(:flavor :jvm [java.lang String] ...)      -> :flavors/:jvm :jar
```

`:import` accepts only direct `:wasm`; `:require` accepts portable Hara or a
generated `:hta`; and JVM libraries load only from an explicit
`(:flavor :jvm [...])` clause. No route falls back to an incompatible artifact
or implicitly selects a JVM JAR. Pure portable packages may omit both maps.

Every artifact path must name an entry in `:files`, and its
`:artifact/sha256` must equal the file entry digest. Packages with a Wasm import
or host flavor also record immutable `:repository` and `:commit` values under
`:package :provenance`. Installation verifies and extracts bytes only; target,
ABI, capabilities, host-call allowlists, and digests are checked before any
JAR classloader, Wasm/HTA instance, provider registration, or package code
exists. Consumer installation never invokes Cargo, Maven, `javac`, `rustc`, or
another build tool.

JVM flavor registration is trusted, isolated, dependency-scoped, and atomic.
Wasm/HTA loading has explicit async completion and cancellation, session-scoped
handles, idempotent load and close, and rollback after initialization failure.
Portable descriptors are ordinary data and may not contain credentials,
classloaders, raw sockets, native pointers, or live handles.

[`conformance/projects.edn`](conformance/projects.edn) contains the shared
normalization, selection, digest, ABI, target, capability, host-call,
lifecycle, isolation, descriptor, and rejection corpus intended for HAL, Rust,
and JVM package loaders.

## Published typed-schema catalogs

An archive may declare one published typed-schema catalog in `package.edn`:

```edn
:schema/catalog {:format "std.typed.catalog/1"
                 :path "catalog/std-typed-catalog.json"
                 :sha256 "sha256:<digest>"}
```

The path and digest must also occur in `:files`. The package builder and every
package reader pass the complete catalog bytes to Hara's canonical
`std.typed.catalog.document` verifier before the archive can be built,
published, installed, activated, or used for provider registration. The
registry owns the descriptor and provenance contract; Hara owns catalog
normalization, hashes, dependency closure, components, and deterministic
verification reports. Omitting `:schema/catalog` is valid for packages that
do not publish a typed-schema catalog.
