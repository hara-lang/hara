#!/usr/bin/env bash
set -euo pipefail
cd pr2
python3 scripts/runtime/sync-rust-hal-src --check
cargo fmt --manifest-path core/rust/Cargo.toml --all -- --check
cargo check --manifest-path core/rust/Cargo.toml --features bytecode-vm
mvn -B -f core/java/pom.xml -DskipTests compile
git diff --check
