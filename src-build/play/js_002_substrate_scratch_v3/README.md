# js-002 substrate scratch-v3

This `src-build/play` project turns `postgres.sample.scratch-v3` into a set of Tahto DSL `xt.substrate` examples.

The generated browser page includes four slices:

- active currency catalogue;
- user profile by account;
- wallet and asset projection;
- the same currency model through the SQLite caching source.

`main.hal` owns the platform-neutral schema bindings, source topology, dataview descriptors, substrate connection, model attachment, page proxy call, and event envelope. `app.hal` only renders those contracts. The native build definition is in `build.hal`.

## Build

From the Hara repository root:

```bash
../hara-native/core/rust/target/release/hara-native test --project . --file test/play/js_002_substrate_scratch_v3/build_test.hal
```

The test checks that the native `work.flow.make` project definition is loaded,
has the expected tag, and exposes the live make host. The final browser artifact
still depends on the archived `xt.substrate` target libraries, which are not
bundled in the current Hara checkout.

```clojure
(require '[play.js-002-substrate-scratch-v3.build :as build])
(build/project)
```

Artifacts are written to:

```text
.build/play-js-002-substrate-scratch-v3/public
```

Serve the generated page with:

```bash
cd .build/play-js-002-substrate-scratch-v3
make start
```

The live substrate functions exported by the generated module are `connect` and `attach-demo`. They use the generated `scratch_v3` schema and application lookup from `postgres.gen` through `pg/bind-schema` and `pg/bind-app`.
