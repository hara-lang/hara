# Getting started

## Install the hara CLI

Linux (x86_64) and macOS (arm64, x86_64):

```shell
brew install hara-lang/tap/hara
# or: brew install hara-lang/tap/hara-truffle
```

The Rust CLI and Truffle native image are separate Homebrew formulas. Both
provide a native executable and neither needs a JVM at runtime.

The verified release installer remains available without Homebrew:

```shell
curl -fsSL https://www.hara-lang.org/install.sh | sh -- --rust --truffle
```

This installs `hara` and the native-image `hara-truffle` to `~/.local/bin`; neither needs a
JVM at runtime. GitHub Releases is the publishing authority for the downloaded packages and
checksums. Install only one runtime with `--rust` or `--truffle`.
Override the location with `HARA_INSTALL_DIR`, or pin a release with `HARA_VERSION=v0.1.2`.

### Install from a source checkout

The root Makefile provides conventional staged installs. The default builds the
Rust CLI and installs it as `hara`:

```shell
make install                         # ~/.local/bin/hara
make install PREFIX=/usr/local       # /usr/local/bin/hara
```

The JVM/Truffle runtime can be installed separately. Its launcher is named
`hara-truffle`, so it can coexist with the Rust CLI:

```shell
make install-truffle                 # launcher + JAR
make install-all                     # Rust and JVM/Truffle runtimes
make uninstall                       # remove files installed by these targets
```

`PREFIX`, `BINDIR`, `DATADIR`, and `HARA_DATADIR` are overridable. Packaging
jobs can stage an install without embedding the staging path in launchers:

```shell
make install DESTDIR="$PWD/pkgroot" PREFIX=/usr
make check-install
```

The sections below build the Java/Truffle runtime from source instead.

## 1. Install prerequisites

Install JDK 21 and Maven, then verify:

```shell
java -version
mvn -version
```

## 2. Build the Truffle runtime

```shell
mvn -f core/java/pom.xml -Ptruffle package
```

This produces `core/java/target/hara-truffle.jar`.

## 3. Evaluate a form

```shell
./core/hara eval '(let [x 19] (+ x 23))'
```

Expected result:

```text
42
```

## 4. Start the REPL

```shell
./core/hara

# ROOT REPL without a RESP listener
./core/hara --offline
```

The REPL supports multiline forms, persistent history, symbol and Java completion, and inline
documentation. See the [Hara website docs](https://hara-lang.org/docs/) and the
[archived REPL planning document](https://github.com/hara-lang/hara-specs-registry/blob/main/99-archive/planning/tooling/repl.md).

## 5. Run a file or stdin

```shell
./core/hara run core/lib/examples/hello.hal
./core/hara stdin < core/lib/examples/hello.hal
```

The shipped examples are catalogued under `core/lib/examples/catalog.json`; deterministic
examples are checked in normal pull-request CI against the pinned specification authority.

## 6. Run tests

```shell
mvn -q -f core/java/pom.xml test
mvn -q -f core/java/pom.xml -Ptruffle -Dtest=hara.truffle.HaraCoreLanguageConformanceTest test
```

For contributor workflows, test slices, native-image builds, and troubleshooting, read the
[developer documentation](https://hara-lang.org/docs/development/). To build a multi-file project,
continue with the [Hara documentation](https://hara-lang.org/docs/).
