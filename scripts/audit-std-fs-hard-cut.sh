#!/usr/bin/env bash
set -euo pipefail

legacy_namespace='std.lib.''fs'
legacy_path='std/lib/''fs'
portable_legacy_root="core/lib/src/${legacy_path}"
test_legacy_root="core/lib/test/${legacy_path}"
rust_legacy_root="core/rust/hal-src/${legacy_path}"
portable_root='core/lib/src/std/fs'
test_root='core/lib/test/std/fs'
rust_root='core/rust/hal-src/std/fs'
legacy_jvm_test='core/java/src/test/java/hara/truffle/Std''FsTest.java'
canonical_jvm_test='core/java/src/test/java/hara/truffle/StdLibFsTest.java'
failed=0

if git grep -n -F "$legacy_namespace" -- .; then
  echo "A retired filesystem namespace remains in tracked content." >&2
  failed=1
fi

if git grep -n -F "$legacy_path" -- .; then
  echo "A retired filesystem source path remains in tracked content." >&2
  failed=1
fi

if git ls-files | grep -F "$legacy_path"; then
  echo "A retired filesystem path remains in the repository tree." >&2
  failed=1
fi

if [[ -e "${portable_legacy_root}.hal" ]] || [[ -e "$portable_legacy_root" ]]; then
  echo "Removed portable filesystem source paths still exist." >&2
  failed=1
fi

if [[ -e "$test_legacy_root" ]]; then
  echo "Removed portable filesystem test paths still exist." >&2
  failed=1
fi

if [[ -e "${rust_legacy_root}.hal" ]] || [[ -e "$rust_legacy_root" ]]; then
  echo "Removed Rust HAL filesystem mirror paths still exist." >&2
  failed=1
fi

if [[ -e "$legacy_jvm_test" ]]; then
  echo "The retired JVM filesystem test class remains." >&2
  failed=1
fi

paths=(
  "${portable_root}.hal"
  "$portable_root/path.hal"
  "$portable_root/walk.hal"
  "$test_root/path_test.hal"
  "$test_root/facade_test.hal"
  "$canonical_jvm_test"
)
for path in "${paths[@]}"; do
  if [[ ! -f "$path" ]]; then
    echo "Canonical filesystem source or test is missing: $path" >&2
    failed=1
  fi
done

for relative in fs.hal fs/path.hal fs/walk.hal; do
  portable="core/lib/src/std/$relative"
  rust="core/rust/hal-src/std/$relative"
  if [[ ! -f "$rust" ]]; then
    echo "Generated Rust HAL filesystem source is missing: $rust" >&2
    failed=1
  elif ! cmp -s "$portable" "$rust"; then
    echo "Portable and generated Rust HAL filesystem sources differ: $relative" >&2
    failed=1
  fi
done

for namespace in std.fs std.fs.path std.fs.walk; do
  if ! grep -Fxq "$namespace" core/rust/standard-library.namespaces; then
    echo "Registered filesystem namespace is missing: $namespace" >&2
    failed=1
  fi
done

exit "$failed"
