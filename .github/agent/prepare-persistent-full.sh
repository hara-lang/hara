#!/usr/bin/env bash
set -euo pipefail
cat \
  staging/.github/agent/full2-00 \
  staging/.github/agent/full2-01 \
  staging/.github/agent/full2-02a \
  staging/.github/agent/full2-02b \
  staging/.github/agent/full2-03 \
  staging/.github/agent/full2-04 \
  staging/.github/agent/full2-05a0 \
  staging/.github/agent/full2-05a1 \
  staging/.github/agent/full2-05a2 \
  staging/.github/agent/full2-05b \
  | base64 --decode | gzip --decompress > /tmp/full2.patch
echo '5ae8bfe82c6bb4ea27c441aa01ffddf3c0c062e20068c3b057223f6d51ad1a49  /tmp/full2.patch' | sha256sum --check
base64 --decode staging/.github/agent/java-fix-1.patch.gz.b64 \
  | gzip --decompress > /tmp/java-fix-1.patch
echo '72776a5c5349af694e9a7f770d492176756716036eec47e666fd877cdc67eff7  /tmp/java-fix-1.patch' | sha256sum --check
git -C full apply --check /tmp/full2.patch
git -C full apply /tmp/full2.patch
git -C full apply --check /tmp/java-fix-1.patch
git -C full apply /tmp/java-fix-1.patch
cargo fmt --manifest-path full/core/rust/Cargo.toml --all
