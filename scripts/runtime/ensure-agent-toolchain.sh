#!/usr/bin/env bash
# Ensure the repository's Java 21 + Rust toolchains are active in the current shell.
# This is designed to be sourced by future agents or scripts running in the repo.

set -Eeuo pipefail

_hara_agent_toolchain_candidates=()
_hara_agent_toolchain_candidates+=("/usr/lib/jvm/temurin-21-jdk-amd64")
_hara_agent_toolchain_candidates+=("/usr/lib/jvm/java-21-openjdk-amd64")
_hara_agent_toolchain_candidates+=("/usr/lib/jvm/default-java")

ensure_hara_agent_toolchain() {
  local candidate
  local java_home="${JAVA_HOME:-}"

  if [[ -n "$java_home" && -x "$java_home/bin/java" ]]; then
    export JAVA_HOME
    export PATH="$JAVA_HOME/bin:$PATH"
  fi

  for candidate in "${_hara_agent_toolchain_candidates[@]}"; do
    if [[ -x "$candidate/bin/java" ]]; then
      export JAVA_HOME="$candidate"
      export PATH="$JAVA_HOME/bin:$PATH"
      break
    fi
  done

  if [[ -n "${HOME:-}" && -d "$HOME/.cargo/bin" ]]; then
    export PATH="$HOME/.cargo/bin:$PATH"
  fi

  hash -r 2>/dev/null || true

  if command -v rustup >/dev/null 2>&1; then
    rustup default stable >/dev/null 2>&1 || true
  fi

  if [[ -n "${JAVA_HOME:-}" && -x "$JAVA_HOME/bin/java" ]]; then
    export PATH="$JAVA_HOME/bin:$PATH"
    hash -r 2>/dev/null || true
  fi
}

ensure_hara_agent_toolchain

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  if command -v java >/dev/null 2>&1; then java -version 2>&1 | head -n 1; fi
  if command -v cargo >/dev/null 2>&1; then cargo --version; fi
fi
