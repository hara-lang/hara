#!/usr/bin/env bash
set -euo pipefail

failed=0
lint_adapter='core/lib/src/tool/lint/schema.hal'
lint_analyzer='core/lib/src/tool/lint/analyze.hal'

required_paths=(
  'core/lib/src/std/typed.hal'
  'core/lib/src/std/typed/catalog.hal'
  'core/lib/src/std/typed/catalog/document.hal'
  'core/lib/src/std/typed/explain.hal'
  'core/lib/src/std/typed/registry.hal'
  'core/lib/src/std/typed/schema.hal'
  'core/lib/src/std/typed/infer.hal'
  "$lint_adapter"
  "$lint_analyzer"
)
for path in "${required_paths[@]}"; do
  if [[ ! -f "$path" ]]; then
    echo "Required std.typed dependency-boundary file is missing: $path" >&2
    failed=1
  fi
done

mapfile -t typed_source_files < <(
  git ls-files | grep -E '^core/lib/src/std/typed(\.hal|/)' || true
)
if [[ "${#typed_source_files[@]}" -eq 0 ]]; then
  echo 'No canonical std.typed source files were found.' >&2
  failed=1
else
  for dependency in 'std.block' 'tool.lint' 'code.test'; do
    if git grep -n -F "$dependency" -- "${typed_source_files[@]}"; then
      echo "Portable std.typed source depends on forbidden upper layer: $dependency" >&2
      failed=1
    fi
  done
fi

mapfile -t foundation_files < <(
  git ls-files | grep -E '^core/lib/src/std/foundation(\.hal|/)' || true
)
if [[ "${#foundation_files[@]}" -gt 0 ]] && \
   git grep -n -F 'std.typed' -- "${foundation_files[@]}"; then
  echo 'Foundation must not depend on std.typed.' >&2
  failed=1
fi

mapfile -t lint_files < <(
  git ls-files | grep -E '^core/lib/src/tool/lint(\.hal|/)' || true
)
mapfile -t lint_typed_consumers < <(
  if [[ "${#lint_files[@]}" -gt 0 ]]; then
    git grep -l -E 'std\.typed\.(schema|infer)' -- "${lint_files[@]}" || true
  fi
)
found_adapter=0
for path in "${lint_typed_consumers[@]}"; do
  if [[ "$path" == "$lint_adapter" ]]; then
    found_adapter=1
  else
    echo "tool.lint bypasses its typed adapter: $path" >&2
    failed=1
  fi
done
if [[ "$found_adapter" -ne 1 ]]; then
  echo 'tool.lint.schema must be the sole direct tool.lint consumer of std.typed.' >&2
  failed=1
fi

if grep -Fq 'std.typed.' "$lint_analyzer"; then
  echo 'tool.lint.analyze must depend on tool.lint.schema, not std.typed directly.' >&2
  failed=1
fi
for dependency in 'std.typed.schema' 'std.typed.infer'; do
  if ! grep -Fq "$dependency" "$lint_adapter"; then
    echo "tool.lint.schema is missing typed dependency: $dependency" >&2
    failed=1
  fi
done

mapfile -t typed_contract_files < <(
  git ls-files | grep -E '^core/(lib/(src|test)/std/typed(\.hal|/)|spec/std/typed)' || true
)
if [[ "${#typed_contract_files[@]}" -gt 0 ]] && \
   git grep -n -E 'res-(success|error|synchronize|with-context|status|data|error-value|context|timeout)' \
     -- "${typed_contract_files[@]}"; then
  echo 'Retired res-* vocabulary remains in the std.typed contract.' >&2
  echo 'Use result, result?, result-status, and the native Result surface.' >&2
  failed=1
fi

for namespace in std.typed std.typed.catalog std.typed.catalog.document std.typed.explain std.typed.registry std.typed.schema std.typed.infer; do
  if ! grep -Fxq "$namespace" core/rust/standard-library.namespaces; then
    echo "std.typed namespace is missing from the standard-library inventory: $namespace" >&2
    failed=1
  fi
done
for namespace in std.typed.registry std.typed.schema std.typed.catalog std.typed.catalog.document std.typed; do
  if ! grep -Fxq "$namespace" core/rust/bootstrap.namespaces; then
    echo "Portable schema bootstrap namespace is missing: $namespace" >&2
    failed=1
  fi
done
registry_line=$(grep -n -F 'std.typed.registry' core/rust/bootstrap.namespaces | cut -d: -f1)
schema_line=$(grep -n -F 'std.typed.schema' core/rust/bootstrap.namespaces | cut -d: -f1)
catalog_line=$(grep -n -F 'std.typed.catalog' core/rust/bootstrap.namespaces | head -n 1 | cut -d: -f1)
document_line=$(grep -n -F 'std.typed.catalog.document' core/rust/bootstrap.namespaces | cut -d: -f1)
typed_line=$(grep -n -F 'std.typed' core/rust/bootstrap.namespaces | tail -n 1 | cut -d: -f1)
if [[ -n "$registry_line" && -n "$schema_line" && "$registry_line" -ge "$schema_line" ]]; then
  echo 'std.typed.registry must bootstrap before std.typed.schema.' >&2
  failed=1
fi
if [[ -n "$catalog_line" && -n "$document_line" && "$catalog_line" -ge "$document_line" ]]; then
  echo 'std.typed.catalog must bootstrap before std.typed.catalog.document.' >&2
  failed=1
fi
if [[ -n "$document_line" && -n "$typed_line" && "$document_line" -ge "$typed_line" ]]; then
  echo 'std.typed.catalog.document must bootstrap before the std.typed facade.' >&2
  failed=1
fi
if grep -Fxq 'tool.lint.schema' core/rust/bootstrap.namespaces; then
  echo 'The lint adapter must not enter the embedded runtime bootstrap.' >&2
  failed=1
fi

if [[ "$failed" -eq 0 ]]; then
  echo 'std.typed dependency boundary is clean.'
fi
exit "$failed"
