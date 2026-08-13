#!/usr/bin/env bash
set -euo pipefail
while IFS= read -r -d '' path; do
  mkdir -p "pr2/$(dirname "$path")"
  cp "full/$path" "pr2/$path"
done < <(
  git -C full diff --name-only -z
  git -C full ls-files --others --exclude-standard -z
)
test "$(git -C pr2 status --porcelain | wc -l)" -eq 34
! git -C pr2 status --porcelain | grep -E '^.. \.github/'
