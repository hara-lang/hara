# Hara

Hara is the canonical source repository for Hara libraries and source packages.
This checkout currently contains the `hara/foundation` project: its native Hara
(`.hal`) implementation and its behavioral test suite.

The host runtime lives in [Hara Native](https://github.com/hara-lang/hara-native).
Keep the boundary explicit:

| Repository | Owns |
| --- | --- |
| `hara-lang/hara` | canonical HAL, Foundation behavior, source-package composition, package compatibility, and the end-user `hara` command |
| `hara-lang/hara-native` | Rust/JVM/browser hosts, HARP verification and installation, provider bridges, and host-artifact releases |

`hara-native` deliberately does not embed Foundation or any other canonical HAL
library. Conversely, this repository does not release Rust crates, Maven
artifacts, npm packages, or native binaries.

## Current repository state

`project.edn` defines a testable Foundation source project:

```text
src/                         canonical HAL source
src/std/foundation.hal        Foundation facade
src/std/foundation/           Foundation implementation namespaces
test/                         path-matched native test files
project.edn                   source/test catalog for hara-native
```

The project is ready for source development, native project tests, and a local
relocatable executable distribution. Its initial `hara.command/main` command
application uses `std.native.Command` for route parsing and response
validation, and supports `--version`, `help`, and the `headless` RESP-server
declaration; the full legacy command set remains source work. It is **not yet ready for a registry release**: it has no release recipe,
published compatibility policy, or completed end-user command package. Do not
create a release tag or send a publication request until the source-package
release checklist below has been completed.

## Develop locally with Hara Native

Use a released `hara-native` binary, or build the sibling checkout while
developing both repositories:

```text
cargo build --release --manifest-path ../hara-native/core/rust/Cargo.toml --bin hara-native
export HARA_NATIVE="$PWD/../hara-native/core/rust/target/release/hara-native"
"$HARA_NATIVE" --version
```

If `hara-native` is already installed, point `HARA_NATIVE` at that executable
instead. Keep a compatibility-sensitive Hara change tied to an identified
native release or commit; do not treat an arbitrary locally built host as a
release dependency.

Run the whole source suite from this repository root:

```text
"$HARA_NATIVE" test --project .
```

Run a changed test file while iterating:

```text
"$HARA_NATIVE" test --project . --file test/std/foundation/string_test.hal
```

The native project runner reads `:project/source-paths` and
`:project/test-paths` from `project.edn`, then starts a fresh runtime for each
selected `.hal` test file. It does not make Foundation ambient to generic
`eval`, `run`, or `repl`; a consumer gets libraries through a verified source
package.

When a mounted source project provides `std.foundation`, Hara Native bootstraps
it before evaluating ordinary project namespaces. Foundation is therefore
intrinsic to that source-project runtime: after requiring an ordinary project
namespace, a façade may use unqualified `(intern-in [published owner/value])`
without `(require 'std.foundation)`. Continue to require ordinary project
namespaces explicitly.

### Write native tests directly

Keep every HAL source file paired with its path-matched test file. Use direct
`Test/check` cases for comparison-style tests—do not add a local `fact` macro
or another compatibility test DSL:

```clojure
(ns example.math-test)

^{:refer example.math/add}
(Test/check
 [{:name "adds two values"
   :test (fn [] (example.math/add 20 22))
   :expected 42}])
```

Use `Test/run` only for registered test facts. `Test/check` is the native
runner's direct ad-hoc case-vector form. Test names should describe behavior;
test metadata should identify the owning source Var with `^{:refer
namespace/symbol}`. Stateful tests must establish and restore their baseline,
and source/package transformations need inverse or idempotence coverage where
appropriate.

## Build a local Hara executable

`project.edn` declares the companion contract rather than putting Hara command
semantics into Rust:

```clojure
:project/hara-bin "target/hara/bin/hara"
:project/distribution {:launcher "hara"
                       :entry hara.command/main
                       :host "../hara-native/core/rust/target/release/hara-native"
                       :output "target/hara"}
```

Build the declared release host, then call the HAL package boundary from a
Hara evaluation or build automation:

```text
cd ../hara-native/core/rust && cargo build --release --bin hara-native

(require 'std.package.build)
(std.package.build/build-distribution)

HARA_DIST_HOME="$(mktemp -d)" target/hara/bin/hara --version
HARA_DIST_HOME="$(mktemp -d)" target/hara/bin/hara help
```

The empty output directory receives a relocatable three-file boundary:

```text
target/hara/bin/hara       copied native host launcher
target/hara/lib/hara.harp  canonical source package
target/hara/lib/release.edn launcher, entry, identity, version, and SHA-256 digests
```

When renamed to `bin/hara`, the native host finds the adjacent manifest,
verifies the host and HARP digests before loading source, installs the verified
package into `HARA_DIST_HOME`, and calls `hara.command/main` with the complete
argument vector. The host has no built-in Hara commands; `hara.command` owns
the `std.native.Command` application and command behavior in HAL. This local
composition check is not a signed registry release:
registry publication still requires its typed recipe, signed source provenance,
and registry attestation.

### Run the Emacs RESP server

`headless` is a Hara source command which declares the host action
`{:hara/host-action :resp}`. A companion distribution recognizes that declared
action, loads the Hara package plus the selected client project, starts a
source-backed broker, and publishes the endpoint expected by `hara-mode`:

```text
HARA_DIST_HOME="$(mktemp -d)" target/hara/bin/hara \
  --project "$PWD" --root "$PWD" --host 127.0.0.1 --port 0 headless
# HARA RESP 127.0.0.1:<ephemeral-port>
```

The process remains running until Emacs disconnects or terminates it. RESP is
deliberately limited to `127.0.0.1`: it has no remote-authentication surface.
The native host owns the listener, broker, transport validation, and lifecycle;
Hara owns the decision to request that host action. Generic `hara-native eval`,
`run`, and `repl` never start this server or make Foundation ambient.

### Bundle the executable directory

Treat the directory as one release unit. Do **not** ship `bin/hara` by itself:
it needs the adjacent HARP package and manifest.

```text
# Run std.package.build/build-distribution on the target platform first.
# It copies the explicit :project/distribution :host into target/hara/bin/hara.

# Preserve the hara/ directory and executable mode in a platform-labelled archive.
tar -C target -czf target/hara-darwin-arm64.tar.gz hara
tar -tzf target/hara-darwin-arm64.tar.gz

# Smoke-test exactly what a user receives.
bundle_root="$(mktemp -d)"
tar -xzf target/hara-darwin-arm64.tar.gz -C "$bundle_root"
HARA_DIST_HOME="$(mktemp -d)" "$bundle_root/hara/bin/hara" --version
HARA_DIST_HOME="$(mktemp -d)" "$bundle_root/hara/bin/hara" help
```

Choose a filename that identifies the host operating system and architecture
actually used to build `hara-native`; for example `darwin-arm64`, `linux-x86_64`,
or `windows-x86_64`. The current builder copies its own executable, so a
platform release is composed by that platform's host binary. A future
cross-platform release workflow can supply prebuilt target launchers, but must
retain the same `bin/` and `lib/` layout and regenerate the manifest digests.

The archive is a transport wrapper around the directory, not a second package
format. `release.edn` detects accidental modification of either enclosed file
at startup; public distribution still needs the signed source and registry
attestation process described below.

The optional legacy sealed executable remains separate: call
`std.package.build/seal-project` when a single payload-bearing executable is
required. It writes its primary and specs HARPs beneath `target/hara-sealed/`;
the normal `target/hara/` companion never mounts or ships the specs package.

Before committing a HAL change, run its focused test in a fresh native process
and then run the full project suite. The repository workflow requires each
implementation function to receive a behavioral test rather than a type-only
or smoke assertion.

## Work at the source/host boundary

Choose the owning repository before making a change:

| Change | Primary repository | Required handoff |
| --- | --- | --- |
| Foundation API, HAL parser-facing source usage, or source package contents | this repository | test with the intended `hara-native` version; publish a source package only after its release contract is complete |
| Runtime execution, HARP integrity, CLI host behavior, JVM, browser, or providers | `hara-native` | use its focused host validation and native-release process |
| New source feature requiring host/ABI support | both, in dependency order | release Hara Native first; then pin and test against its published artifact in the Hara source release |

Do not add canonical HAL, an embedded Foundation bundle, or a source-checkout
dependency to `hara-native`. Do not point a Hara source release at an unmerged
or unpublished native commit. The published source package must record the
exact native artifacts it supports.

For host work, follow the [Hara Native developer guide](https://github.com/hara-lang/hara-native/blob/main/DEVELOPING.md).
Its fast Rust loop is `make test-boundary` plus `make test-rust`; its
cross-host ABI layer is `make test-conformance`. Those tests belong in Hara
Native and do not replace this repository's HAL suite.

## Source-package publishing (when this repository is release-ready)

GitHub Packages is the source of truth for Hara releases. A release is not an
upload of a local `.harp` file and it does not use the Hara Identity service.
The source repository proves a signed tag; the protected
[`hara-lang/hara-packages`](https://github.com/hara-lang/hara-packages)
workflow rebuilds the tag and is the only writer to GHCR.

The paired immutable artifacts for this repository are:

```text
ghcr.io/hara-packages/hara-lang.hara:<version>        source HARP
ghcr.io/hara-packages/hara-lang.hara.specs:<version>  specification HARP
```

The public Packages API remains the transport for clients. It returns locks
with the archive SHA-256 and the exact `:oci/repository` and `:oci/manifest`;
clients do not need a GitHub token or an Identity-policy checkout.

1. Complete the package boundary in this repository:
   - set the immutable `:project/version` and the intended package coordinate;
   - add `:project/recipe` pointing to a typed recipe with `:recipe/format`,
     `:recipe/adapter`, `:recipe/toolchain`, `:recipe/inputs`, and
     `:recipe/outputs`;
   - keep the complete specification corpus under `spec/content/`; its
     artifact-only `spec/project.edn` is built through
     `std.package.build/build-specs`;
   - pin the native revision that performs the rebuild in the release
     environment variable `HARA_NATIVE_PUBLICATION_REVISION`.
2. Use that target Hara Native binary to run the HAL suite and inspect both
   reproducible local inputs:

   ```text
   "$HARA_NATIVE" test --project .
   "$HARA_NATIVE" bundle build . --output /tmp/hara-source.harp
   "$HARA_NATIVE" bundle verify /tmp/hara-source.harp
   "$HARA_NATIVE" bundle build spec --output /tmp/hara-specs.harp
   "$HARA_NATIVE" bundle verify /tmp/hara-specs.harp
   ```

3. From a clean, reviewed `main`, push the exact source commit and create a
   signed version tag. The tag name normally equals `:project/version`:

   ```text
   git status --short
   git push origin main
   git tag -s <version> -m "hara source <version>"
   git verify-tag <version>
   git push origin <version>
   git ls-remote --tags origin refs/tags/<version>
   ```

4. The tag runs `package-publication-request.yml`. It verifies the signed tag,
   records the Hara Native revision and `spec` Git tree, signs a receipt with
   GitHub OIDC, and opens or updates a receipt pull request in `hara-packages`.
   The source workflow has no GHCR credential.
5. Review and merge that receipt pull request. The protected central workflow
   checks the source files against the receipt, rebuilds and verifies both
   HARPs, publishes their immutable version and digest tags, makes them public,
   and reads the manifests back from GHCR. A merged receipt is the authority;
   a locally built HARP or a source tag alone is not a published package.

Never reuse or move a published version tag, upload an archive with `curl`,
copy an archive into registry storage, or place a GHCR credential in this
repository. GitHub environment protection and the central workflow govern
release authority.

## Native host releases are separate

A Hara Native release publishes host artifacts—not this repository's HAL:
Rust crates and CLI archives, the JVM artifact, browser/npm packages, and the
container image. Its release flow is:

1. complete and validate the host change on `hara-native/main`;
2. promote it by merge-commit pull request to `hara-native/release`;
3. let the native release preflight validate the host and public artifacts;
4. have a maintainer dispatch the protected native release promotion from
   `release`.

That workflow creates the native GitHub release and verifies the public
registries. Hara source changes do not automatically require a native release,
and a native host fix does not automatically require a new Hara source package.
When a release changes their compatibility boundary, publish the native
artifact first and make the subsequent Hara source release explicitly depend
on that published version.

See [Hara Native releases](https://github.com/hara-lang/hara-native/blob/main/RELEASES.md)
for the protected-branch, preflight, recovery, and credential rules.

## Contribution flow

1. Branch from current `main` and keep the change limited to its owning layer.
2. For HAL work, evaluate the candidate, run the focused native test, write the
   smallest edit, and rerun it from disk in a fresh Hara Native process.
3. Run the complete project suite for any behavioral/source change.
4. Review the complete diff; commit only the intended files and open a pull
   request against `main`.
5. For a release, follow the publication or native-release checklist above;
   merging a pull request, creating a tag, and submitting a registry request
   are separate verified states.
