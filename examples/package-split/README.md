# Package split example

This is a small Hara consumer project. It proves that the Foundation source
can be split using `config/packages.edn`, built into HARP archives, installed
into a package store, and consumed without importing the Foundation umbrella
project.

## What the project declares

The consumer has exactly two direct package dependencies in
[`project.edn`](project.edn):

```edn
"hara:hara/lang.core" {:version "=0.1.16"}
"hara:hara/code.test"  {:version "=0.1.16"}
```

The package coordinates above are not namespace imports. They identify package
artifacts. The source imports the public Hara namespaces exported by those
artifacts:

```clojure
(ns package.split.main
  (:require [lang.core :as l]
            [code.test :as t]))
```

The source calls `lang.core/rt-create` and `code.test/exactly`. The test checks
their exact result, so a missing package, missing facade, or incorrect public
surface causes the test to fail.

## How the split works

The source-package pipeline has four boundaries:

```text
config/packages.edn
        │ semantic package ownership
        ▼
code.deploy plan/stage
        │ child project.edn files and dependency graph
        ▼
code.deploy build
        │ verified HARP archives and index.edn
        ▼
package sync + HARA_DIST_HOME
        │ installed direct and transitive packages
        ▼
package.split.main
        │ lang.core and code.test namespace imports
        ▼
passing native Hara test
```

`config/packages.edn` maps semantic package names to namespace families. For
example, the `hara/lang.core` entry owns the `lang.core` implementation and
facade, while `hara/code.test` owns the `code.test` implementation and facade.
Each namespace family has one owner.

`code.deploy` reads those ownership declarations together with the source
files. It uses namespace linkage and explicit catalogue dependencies to create
a dependency-first set of child projects. A child project contains its owned
package source plus the source context needed to compile it, but its package
identity remains separate.

The generated package coordinates use the `hara:hara/...` form. Thus:

```text
hara/lang.core → hara:hara/lang.core
hara/code.test → hara:hara/code.test
```

The consumer declares only those two coordinates. Packages such as
`std.foundation`, `std.lib`, `std.typed`, `work`, and `code.framework` are
resolved transitively from the package metadata.

## Pull versus sync

The package commands have intentionally different boundaries:

| Command | Downloads and verifies archives | Installs packages | Writes `project.lock.edn` |
| --- | ---: | ---: | ---: |
| `package pull` | yes | no | no |
| `package sync` | yes | yes | yes |

`pull` is useful when you want to populate the digest-addressed archive cache
without changing the project or its lockfile. `sync` is the command needed by
this example because the native test must resolve packages from an installed
store.

## Run the proof

Run these commands from the Hara repository root:

```text
export HARA_NATIVE=/path/to/workspace/technology/hara-native/core/rust/target/release/hara-native
test -x "$HARA_NATIVE"

# Build a current source-owned Hara CLI when target/hara/bin/hara is stale.
HARA_DIST="$(mktemp -d)"
"$HARA_NATIVE" distribution build . --output "$HARA_DIST"
HARA_CLI="$HARA_DIST/bin/hara"

# Inspect the ownership and dependency graph, then build every HARP package.
"$HARA_CLI" deploy plan --root .
"$HARA_CLI" deploy build --root .

# Use a clean package store so local installations cannot hide missing edges.
HARA_STORE="$(mktemp -d)"
HARA_DIST_HOME="$HARA_STORE" "$HARA_CLI" package sync \
  --root examples/package-split \
  --registry "$PWD/target/deploy/index.edn"

# Resolve the consumer only from the installed package artifacts.
env -u HARA_WORKSPACE_ROOT -u HARA_PROJECT_ROOT \
  HARA_DIST_HOME="$HARA_STORE" "$HARA_NATIVE" test \
  --project examples/package-split

# Remove generated consumer state and temporary stores after the proof.
rm -f examples/package-split/project.lock.edn
rm -rf "$HARA_STORE" "$HARA_DIST"
unset HARA_NATIVE HARA_CLI HARA_STORE HARA_DIST HARA_DIST_HOME
```

The expected final output includes:

```text
PASS .../examples/package-split/test/package/split/main_test.hal
SUMMARY files=1 passed=1 failed=0 error=0
```

## What this proves

The example proves that:

- package ownership in `config/packages.edn` can be materialised as separate
  HARP packages;
- `lang.core` and `code.test` can be direct consumer dependencies;
- their Foundation dependencies are resolved transitively;
- the public facades load from installed artifacts; and
- the native Hara test runner does not need the Foundation umbrella package or
  a local workspace checkout.

The generated archives under `target/deploy/` and the consumer lockfile are
validation outputs. They are not source files for this example.
