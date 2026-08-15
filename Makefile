SHELL := /bin/sh

PREFIX ?= $(HOME)/.local
DESTDIR ?=
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share
HARA_DATADIR ?= $(DATADIR)/hara

CARGO ?= cargo
MVN ?= mvn
INSTALL ?= install

RUST_MANIFEST ?= core/rust/Cargo.toml
RUST_BINARY ?= core/rust/target/release/hara
TRUFFLE_POM ?= core/java/pom.xml
TRUFFLE_JAR ?= core/java/target/hara-truffle.jar

.PHONY: all build build-rust build-truffle \
  install install-rust install-rust-files \
  install-truffle install-truffle-files install-all \
  uninstall check-install help

all: build

build: build-rust

build-rust:
	$(CARGO) build --release --manifest-path "$(RUST_MANIFEST)" --bin hara

build-truffle:
	$(MVN) -f "$(TRUFFLE_POM)" -Ptruffle -DskipTests package

install: install-rust

install-rust: build-rust
	@$(MAKE) --no-print-directory install-rust-files

install-rust-files:
	@test -x "$(RUST_BINARY)" || { \
	  printf 'Hara Rust binary not found or not executable: %s\n' "$(RUST_BINARY)" >&2; \
	  exit 1; \
	}
	$(INSTALL) -d "$(DESTDIR)$(BINDIR)"
	$(INSTALL) -m 755 "$(RUST_BINARY)" "$(DESTDIR)$(BINDIR)/hara"
	@printf 'Installed Hara Rust runtime: %s\n' "$(BINDIR)/hara"

install-truffle: build-truffle
	@$(MAKE) --no-print-directory install-truffle-files

install-truffle-files:
	@test -r "$(TRUFFLE_JAR)" || { \
	  printf 'Hara Truffle JAR not found or not readable: %s\n' "$(TRUFFLE_JAR)" >&2; \
	  exit 1; \
	}
	$(INSTALL) -d "$(DESTDIR)$(BINDIR)" "$(DESTDIR)$(HARA_DATADIR)"
	$(INSTALL) -m 644 "$(TRUFFLE_JAR)" "$(DESTDIR)$(HARA_DATADIR)/hara-truffle.jar"
	@{ \
	  printf '%s\n' '#!/bin/sh'; \
	  printf '%s\n' 'set -eu'; \
	  printf '%s\n' 'JAR="$${HARA_RUNTIME_JAR:-$(HARA_DATADIR)/hara-truffle.jar}"'; \
	  printf '%s\n' 'JAVA="$${HARA_JAVA:-java}"'; \
	  printf '%s\n' 'if [ ! -r "$$JAR" ]; then'; \
	  printf '%s\n' '  printf "Hara runtime JAR not found: %s\\n" "$$JAR" >&2'; \
	  printf '%s\n' '  exit 2'; \
	  printf '%s\n' 'fi'; \
	  printf '%s\n' 'if ! command -v "$$JAVA" >/dev/null 2>&1; then'; \
	  printf '%s\n' '  printf "Java runtime not found: %s\\n" "$$JAVA" >&2'; \
	  printf '%s\n' '  exit 2'; \
	  printf '%s\n' 'fi'; \
	  printf '%s\n' 'exec "$$JAVA" -jar "$$JAR" "$$@"'; \
	} > "$(DESTDIR)$(BINDIR)/hara-truffle"
	chmod 755 "$(DESTDIR)$(BINDIR)/hara-truffle"
	@printf 'Installed Hara Truffle launcher: %s\n' "$(BINDIR)/hara-truffle"
	@printf 'Installed Hara Truffle runtime: %s\n' "$(HARA_DATADIR)/hara-truffle.jar"

install-all: install-rust install-truffle

uninstall:
	rm -f "$(DESTDIR)$(BINDIR)/hara" \
	      "$(DESTDIR)$(BINDIR)/hara-truffle" \
	      "$(DESTDIR)$(HARA_DATADIR)/hara-truffle.jar"
	@rmdir "$(DESTDIR)$(HARA_DATADIR)" 2>/dev/null || true
	@rmdir "$(DESTDIR)$(BINDIR)" 2>/dev/null || true

check-install:
	sh scripts/runtime/test-make-install.sh

help:
	@printf '%s\n' \
	  'make install            Build and install the Rust CLI as hara' \
	  'make install-truffle    Build and install the JVM/Truffle launcher' \
	  'make install-all        Install both runtimes' \
	  'make uninstall          Remove files installed by these targets' \
	  'make check-install      Exercise staged install and uninstall flows' \
	  '' \
	  'Variables: PREFIX, DESTDIR, BINDIR, DATADIR, HARA_DATADIR'
