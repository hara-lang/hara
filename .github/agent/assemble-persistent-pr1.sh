#!/usr/bin/env bash
set -euo pipefail
while IFS= read -r path; do
  mkdir -p "pr1/$(dirname "$path")"
  cp "full/$path" "pr1/$path"
done < staging/.github/agent/persistent-pr1-files.txt
perl -0pi -e 's/\Q(count [point] (field point :x))\E/(count [point] (:x point))/' \
  pr1/core/lib/test/std/struct_test.hal
perl -0pi -e 's/\Qeval("(do (defstruct Point [x y]) (field (->Point 19 23) :y))")\E/eval("(do (defstruct Point [x y]) (:y (->Point 19 23)))")/' \
  pr1/core/rust/src/vm/execution_tests.rs
perl -0pi -e 's/\Qeval("(do (defstruct Point [x y]) (def make ->Point) (field (make 1 2) :x))")\E/eval("(do (defstruct Point [x y]) (def make ->Point) (:x (make 1 2)))")/' \
  pr1/core/rust/src/vm/execution_tests.rs
test "$(git -C pr1 status --porcelain | wc -l)" -eq 79
! git -C pr1 status --porcelain | grep -E '^.. \.github/'
