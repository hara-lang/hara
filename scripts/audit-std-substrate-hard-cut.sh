#!/usr/bin/env bash
set -euo pipefail

legacy_namespace='std.lib.''substrate'
legacy_path='std/lib/''substrate'
failed=0

if git grep -n -F "$legacy_namespace" -- .; then
  echo "A retired substrate namespace remains in tracked content." >&2
  failed=1
fi

if git grep -n -F "$legacy_path" -- .; then
  echo "A retired substrate source path remains in tracked content." >&2
  failed=1
fi

for root in core/lib/src/std/substrate core/rust/hal-src/std/substrate; do
  for file in core frame json protocol pubsub request router space transport_memory util util_handlers; do
    if [[ ! -f "$root/$file.hal" ]]; then
      echo "Canonical substrate module is missing: $root/$file.hal" >&2
      failed=1
    fi
  done
done

if [[ ! -f core/lib/src/std/substrate.hal ]] || [[ ! -f core/rust/hal-src/std/substrate.hal ]]; then
  echo "Canonical substrate roots are incomplete." >&2
  failed=1
fi

if ! cmp -s core/lib/src/std/substrate.hal core/rust/hal-src/std/substrate.hal; then
  echo "Portable and Rust HAL substrate roots differ." >&2
  failed=1
fi

if ! diff -qr core/lib/src/std/substrate core/rust/hal-src/std/substrate; then
  echo "Portable and Rust HAL substrate namespace trees differ." >&2
  failed=1
fi

protocols=$(rg -o '^\(defprotocol [A-Za-z0-9_-]+' core/lib/src/std/substrate/protocol.hal | cut -d ' ' -f 2)
if [[ -z "$protocols" ]] || grep -Ev '^ISubstrate[A-Za-z0-9_-]+$' <<<"$protocols"; then
  echo "Every canonical substrate protocol must begin with ISubstrate." >&2
  failed=1
fi

exit "$failed"
