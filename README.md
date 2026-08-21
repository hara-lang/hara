# Hara

Hara is a small Lisp runtime and toolchain for building, inspecting, and
changing live systems. This guide is for someone working from a source checkout:
build a runtime, install it locally, use it from Emacs, and understand how an
official release is published.

If you are looking for the architecture, repository map, package workspaces,
or the relationship between the Hara repositories, see the
[repository guide](docs/repository-guide.md).

## Choose a runtime

You do not need every Hara runtime for ordinary development.

| Runtime | Command after installation | Use it for |
| --- | --- | --- |
| Rust | `hara` | The normal native CLI and recommended local default |
| Rust lite | `hara-lite` | A smaller recovery/development CLI with evaluator and REPL commands |
| Truffle JVM | `hara-truffle` | The Java/Truffle runtime, JVM tests, and JVM-specific debugging |
| Truffle native image | `hara-truffle-native` | A standalone GraalVM-built Truffle executable |
| Browser Wasm | no terminal command | Browser, Studio, and embedding work |

Start with the Rust runtime. Build another variant only when the work you are
doing needs it.

## 1. Prepare the checkout

Hara itself lives in `hara-lang/hara`. The Truffle build and several
conformance suites also expect `hara-specs-registry` beside it.

```shell
git clone https://github.com/hara-lang/hara.git
git clone https://github.com/hara-lang/hara-specs-registry.git
cd hara
```

In the Greenways workspace used by this repository, those paths are already:

```text
technology/hara
technology/hara-specs-registry
extensions/hara-emacs
```

### Toolchains

Install only the toolchains needed by the variants you intend to build:

| Variant | Required tools |
| --- | --- |
| Rust and Rust lite | stable Rust and Cargo |
| Truffle JVM | JDK 21 and Maven |
| Truffle native image | GraalVM 25 with `native-image`, plus Maven |
| Browser Wasm | Rust target `wasm32-unknown-unknown`, `wasm-bindgen`, Node.js 22, and npm |

Confirm the basics before starting:

```shell
cargo --version
java -version
mvn --version
```

For Wasm development, add the Rust target and install the `wasm-bindgen` CLI
whose version matches `core/rust/Cargo.lock`.

## 2. The quickest local build and install

From the repository root:

```shell
make install
```

This builds the optimised Rust CLI and installs:

```text
~/.local/bin/hara
~/.local/share/hara-lite/project.edn
~/.local/share/hara-lite/lib/
```

Make sure `~/.local/bin` is on `PATH`, then smoke-test the installed binary:

```shell
hara --version
hara eval '(+ 19 23)'
```

The expected evaluation result is `42`.

To install under a different prefix:

```shell
make install PREFIX=/opt/hara
```

To test the install layout without writing into the real prefix:

```shell
make check-install
```

To remove files installed by these Make targets:

```shell
make uninstall
```

## 3. Build and install every local variant

### Rust CLI

Build without installing:

```shell
make build-rust
core/rust/target/release/hara eval '(+ 19 23)'
```

Build and install as `hara`:

```shell
make install-rust
```

The official release build enables the `native-jit` feature. To reproduce that
exact binary locally:

```shell
cargo build --release \
  --manifest-path core/rust/Cargo.toml \
  --features native-jit \
  --bin hara
```

### Rust lite CLI

Rust lite deliberately exposes only evaluator and REPL operations. It remains
usable when higher-level Hara tooling is being repaired.

```shell
make build-rust-lite
core/rust/target/release/hara-lite --project core eval '(+ 19 23)'
make install-rust-lite
hara-lite eval '(+ 19 23)'
```

The installed binary finds its portable Hara project under
`~/.local/share/hara-lite`. Set `HARA_LITE_PROJECT` only when you intentionally
want to use another copy.

### Truffle on the JVM

Build the executable JAR:

```shell
make build-truffle
java -jar core/java/target/hara-truffle.jar eval '(+ 19 23)'
```

Build and install the JAR plus a launcher:

```shell
make install-truffle
hara-truffle eval '(+ 19 23)'
```

The installed layout is:

```text
~/.local/bin/hara-truffle
~/.local/share/hara/hara-truffle.jar
```

Use `HARA_JAVA` to select a Java executable and `HARA_RUNTIME_JAR` to point the
launcher at another JAR.

### Truffle native image

This build needs GraalVM's `native-image` command, not only a normal JDK.

```shell
make build-truffle-native
core/target/hara-truffle eval '(+ 19 23)'
make install-truffle-native
hara-truffle-native eval '(+ 19 23)'
```

The local Make target uses the name `hara-truffle-native` so it can coexist
with the JVM launcher. Official release archives use the shorter binary name
`hara-truffle` because they contain the native image, not the JVM launcher.

### Raw and browser Wasm

Build all raw Wasm variants:

```shell
scripts/runtime/build-hara-wasm-raw all
```

The variants are:

- `hara-wasm-core` — the smallest raw evaluator;
- `hara-wasm-vm` — the default bytecode-VM build;
- `hara-wasm-trace` — development tracing enabled.

Install browser workspace dependencies once, then build the browser packages:

```shell
npm ci --prefix core/rust/web
scripts/runtime/build-hara-browser all
```

Browser packages are written beneath
`core/rust/web/packages/browser/dist/`. They are build artifacts for embedding
and Studio development; they are not terminal executables and the repository
currently has no automatic npm publication workflow for `@hara-lang/browser`.

## 4. Validate a local build

Use the smallest check that covers your change while iterating:

```shell
cargo test --manifest-path core/rust/Cargo.toml
mvn -f core/java/pom.xml -Ptruffle test
scripts/runtime/run-lib-tests
```

Before treating a runtime change as release-ready, run the aggregate core
checks:

```shell
make -C core check-all
```

Useful narrower Make targets are listed by:

```shell
make -C core help
```

For any saved `.hal` change, follow the repository's fresh-process workflow:
evaluate the complete candidate, run its focused test, write the edit, run the
written file in a new Hara process, and repeat the focused test.

## 5. Use the local build from Emacs

The workspace's `hara-emacs` checkout automatically prefers runtime artifacts
built in this repository. A simple configuration is:

```elisp
(use-package hara-mode
  :load-path "/path/to/hara-extensions/hara-emacs"
  :mode ("\\.hal\\'" . hara-mode))
```

Build the runtime you want, set `HARA_BACKEND` before Emacs starts, then open a
`.hal` file:

```shell
make build-rust-lite
export HARA_BACKEND=rust-lite
emacs
```

Supported backend names in the package launcher are `rust`, `rust-lite`,
`truffle`, and `native`. In Emacs, `M-x hara-jack-in` starts or reuses the
project server and `M-x hara-repl` opens that project's REPL.

See the
[hara-emacs guide](https://github.com/hara-lang/hara-extensions/tree/main/hara-emacs)
for the daily evaluation, testing, source/test navigation, and `code.manage`
commands.

## 6. Understand official publication

There is an important distinction:

- your local computer prepares, validates, reviews, and authorises a release;
- GitHub Actions builds platform artifacts and publishes them from an immutable
  commit or tag.

Do not upload hand-built local binaries as official release artifacts. The CI
matrix is the authority for platform builds, checksums, smoke installs, and
downstream package updates.

### What the main release publishes

A normal `vX.Y.Z` release runs `.github/workflows/release.yml` and produces:

- Rust CLI archives for Linux x86-64, macOS x86-64, and macOS arm64;
- Truffle native-image archives for Linux x86-64, macOS x86-64, and macOS arm64;
- the versioned Studio runtime archive and checksum;
- a combined `SHA256SUMS` file;
- a GitHub prerelease while the version is `0.x`;
- binary Homebrew formulas in `hara-lang/homebrew-tap`;
- the source-built formula used by the Greenways tap when its token is present.

The workflow then installs the public artifacts on clean Linux and macOS
runners and verifies `(+ 19 23)` returns `42`.

### Prepare a normal reviewed release

1. Choose `X.Y.Z` and update every versioned surface. The authoritative check
   lists those surfaces and fails if they disagree:

   ```shell
   node scripts/runtime/check-release-version.mjs X.Y.Z
   ```

2. Run the relevant focused suites and `make -C core check-all`.

3. Merge the version change to `main`. Record the exact merged commit SHA.

4. Add one reviewed manifest at `.github/releases/vX.Y.Z.json`:

   ```json
   {
     "schema": "hara-release-cut/0-alpha",
     "version": "X.Y.Z",
     "tag": "vX.Y.Z",
     "commit": "FULL_40_CHARACTER_COMMIT_SHA",
     "workflow": "release.yml"
   }
   ```

5. Open and merge the manifest pull request. The
   `cut-reviewed-release.yml` workflow verifies that the commit is on `main`,
   creates the immutable annotated tag, starts `release.yml`, waits for it, and
   verifies the complete public asset set.

6. Read back the GitHub release, the release workflow result, and the Homebrew
   updates. Homebrew publication is intentionally `continue-on-error`, so a
   green runtime release does not by itself prove both taps were updated.

This reviewed-manifest route is the preferred full-release path. Although
`release.yml` can react to a pushed `v*` tag, manually cutting the full release
tag bypasses the repository's review gate.

### Publish the Rust lite prerelease

Rust lite has a separate tag and workflow. Its tag describes a lite release of
the current Cargo version, for example `v0.1.6-lite.1`:

```shell
git tag --annotate v0.1.6-lite.1 --message "hara-lite v0.1.6-lite.1"
git push origin v0.1.6-lite.1
```

`.github/workflows/release-lite.yml` builds all three supported Rust platform
archives and creates a GitHub prerelease. Confirm that `SHA256SUMS` and every
expected `hara-rust-lite-*` archive are present before announcing it.

### Publish the Rust crates

Crates.io publication is separate from the CLI release because crates are
immutable. After the versioned commit or tag is reviewed, dispatch:

```shell
gh workflow run publish-rust-crates.yml --ref main --field ref=vX.Y.Z
```

The workflow materialises the HAL source archive and publishes, in dependency
order:

1. `hara-abi`
2. `hara-hta`
3. `hara-wasm`
4. `hara-vm`
5. `hara-compiler`

It waits for each dependency to appear in the crates.io index before publishing
the next one, then retains the exact `.crate` archives and checksums as a
workflow artifact.

### Publish the Maven snapshot

The Java artifact is currently a snapshot, not a Maven Central release. Its
version comes from `core/java/pom.xml` and must end in `-SNAPSHOT`.

```shell
gh workflow run publish-maven-snapshot.yml --ref main
```

The workflow builds the Wasm fixtures, runs the Maven tests, and deploys
`org.hara-lang:hara.lang` to the configured Central Portal snapshot repository.

### Publish `@hara-lang/hta`

The HTA npm package is released independently. Update and validate
`core/rust/web/packages/hta/package.json`, merge that commit, then push an
annotated tag matching the package version exactly:

```shell
git tag --annotate hta-vX.Y.Z --message "@hara-lang/hta X.Y.Z"
git push origin hta-vX.Y.Z
```

The `publish-hta.yml` workflow runs the source and packed-consumer tests. It
publishes only when that exact npm version does not already exist; otherwise it
requires the registry integrity to match the locally packed artifact.

## Publication map

| Deliverable | Local build | Official publication trigger |
| --- | --- | --- |
| Rust CLI | `make build-rust` | reviewed `vX.Y.Z` release manifest |
| Truffle native image | `make build-truffle-native` | same reviewed `vX.Y.Z` release |
| Studio runtime | website assembly script in release CI | same reviewed `vX.Y.Z` release |
| Rust lite CLI | `make build-rust-lite` | pushed `vX.Y.Z-lite.N` tag |
| Rust crates | `cargo package` | manual `publish-rust-crates.yml` dispatch |
| Truffle JVM snapshot | `make build-truffle` | manual `publish-maven-snapshot.yml` dispatch |
| `@hara-lang/hta` | npm workspace build/test | pushed `hta-vX.Y.Z` tag |
| Browser Wasm package | `scripts/runtime/build-hara-browser all` | no npm publication workflow currently |

## More documentation

- [Repository structure and architecture](docs/repository-guide.md)
- [Getting started](GETTING_STARTED.md)
- [Contributing](CONTRIBUTING.md)
- [Architecture](ARCHITECTURE.md)
- [Hara for Emacs](https://github.com/hara-lang/hara-extensions/tree/main/hara-emacs)
- [License inventory](LICENSES/README.md)

Hara-owned source is licensed under the [Apache License 2.0](LICENSE).
