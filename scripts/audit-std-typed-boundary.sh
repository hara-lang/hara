#!/usr/bin/env bash
set -euo pipefail

# Temporary migration baseline for issue #667.
#
# std.typed must ultimately contain no std.block dependency. At the start of
# the migration, the canonical and mirrored infer namespaces are the only
# known exceptions. This audit prevents the dependency from spreading. The
# implementation tranche that moves block conversion into tool.lint must
# remove both allowlist entries and leave the match set empty.

roots=(
  core/lib/src/std/typed
  core/rust/hal-src/std/typed
)

allowed=(
  core/lib/src/std/typed/infer.hal
  core/rust/hal-src/std/typed/infer.hal
)

failed=0
mapfile -t matches < <(git grep -l -F 'std.block' -- "${roots[@]}" || true)

allowed_path() {
  local candidate="$1"
  local path
  for path in "${allowed[@]}"; do
    if [[ "$candidate" == "$path" ]]; then
      return 0
    fi
  done
  return 1
}

for path in "${matches[@]}"; do
  if ! allowed_path "$path"; then
    echo "Unexpected std.block dependency under std.typed: $path" >&2
    failed=1
  fi
done

for path in "${allowed[@]}"; do
  if [[ ! -f "$path" ]]; then
    echo "Temporary std.typed boundary allowlist path is missing: $path" >&2
    failed=1
  elif ! grep -q -F 'std.block' "$path"; then
    echo "Remove resolved std.typed boundary exception from this audit: $path" >&2
    failed=1
  fi
done

if [[ -f core/lib/src/std/typed/infer.hal ]] \
   && [[ -f core/rust/hal-src/std/typed/infer.hal ]] \
   && ! cmp -s core/lib/src/std/typed/infer.hal core/rust/hal-src/std/typed/infer.hal; then
  echo "Canonical and Rust-mirrored std.typed.infer sources differ." >&2
  failed=1
fi

if [[ "$failed" -eq 0 ]]; then
  echo "std.typed boundary baseline holds: std.block is confined to the two temporary infer mirrors."
fi

exit "$failed"
