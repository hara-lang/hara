#!/usr/bin/env bash
set -euo pipefail

REPO="${1:?repository checkout required}"
SPECS="${2:?specification checkout required}"
BUILDER="$(cd "$(dirname "$0")/.." && pwd)"
REPO="$(cd "$REPO" && pwd)"
SPECS="$(cd "$SPECS" && pwd)"
CANDIDATES="${RUNNER_TEMP:-/tmp}/std-fs-candidates"
FINAL_BRANCH="agent/650-sync-std-fs"
MATERIALIZER="$BUILDER/scripts/agent-650-make-std-fs-synchronous.py"

cd "$REPO"
BASE_SHA="$(git rev-parse HEAD)"
ln -sfn "$SPECS" hara-specs-registry

cargo build --manifest-path core/rust/Cargo.toml --bin hara --bin hara-test
HARA=core/rust/target/debug/hara
HARA_TEST=core/rust/target/debug/hara-test
"$HARA" --version

python3 -m py_compile "$MATERIALIZER"
rm -rf "$CANDIDATES"
python3 "$MATERIALIZER" candidates . "$CANDIDATES"
find "$CANDIDATES" -type f -print | sort

rebuild_embedded_library() {
  python3 scripts/runtime/sync-rust-hal-src
  python3 scripts/runtime/sync-rust-hal-src --check
  cargo build --manifest-path core/rust/Cargo.toml --bin hara --bin hara-test
}

apply_candidate() {
  python3 "$MATERIALIZER" apply . "$CANDIDATES" "$@"
}

# std.fs.walk: complete candidate, focused assertion, write, file run,
# rebuilt fresh-process focused assertion.
WALK_CANDIDATE="$CANDIDATES/core/lib/src/std/fs/walk.hal"
"$HARA" --project core --offline stdin < "$WALK_CANDIDATE"
cat "$WALK_CANDIDATE" > "$RUNNER_TEMP/walk-candidate-check.hal"
cat >> "$RUNNER_TEMP/walk-candidate-check.hal" <<'HAL'

(let [root (deref (File/temp-directory "/" {:prefix "std-fs-walk-candidate-"}))
      a (str root "/a")
      b (str root "/b")
      _ (deref (File/write b (bytes 2)))
      _ (deref (File/write a (bytes 1)))
      values (vec (map :path (walk root)))
      result [(promise? (File/entries root))
              (= values [a b])
              (promise? values)]
      _ (deref (File/delete a))
      _ (deref (File/delete b))
      _ (deref (File/delete root))]
  (if (= result [true true false])
    :walk-candidate-ok
    (throw (ex-info "Synchronous walk candidate assertion failed"
                    {:result result :values values}))))
HAL
"$HARA" --project core --offline stdin < "$RUNNER_TEMP/walk-candidate-check.hal"
apply_candidate core/lib/src/std/fs/walk.hal
rebuild_embedded_library
"$HARA" --offline run core/lib/src/std/fs/walk.hal
cat > "$RUNNER_TEMP/walk-written-check.hal" <<'HAL'
(do
  (require 'std.fs.walk)
  (let [root (deref (File/temp-directory "/" {:prefix "std-fs-walk-written-"}))
        a (str root "/a")
        b (str root "/b")
        _ (deref (File/write b (bytes 2)))
        _ (deref (File/write a (bytes 1)))
        values (vec (map :path (std.fs.walk/walk root)))
        result [(promise? (File/entries root))
                (= values [a b])
                (promise? values)]
        _ (deref (File/delete a))
        _ (deref (File/delete b))
        _ (deref (File/delete root))]
    (if (= result [true true false])
      :walk-written-ok
      (throw (ex-info "Written synchronous walk assertion failed"
                      {:result result :values values})))))
HAL
"$HARA" --project core --offline stdin < "$RUNNER_TEMP/walk-written-check.hal"

# std.fs: repeat the required evaluate/write/rebuild/fresh-process workflow.
FS_CANDIDATE="$CANDIDATES/core/lib/src/std/fs.hal"
"$HARA" --project core --offline stdin < "$FS_CANDIDATE"
cat "$FS_CANDIDATE" > "$RUNNER_TEMP/fs-candidate-check.hal"
cat >> "$RUNNER_TEMP/fs-candidate-check.hal" <<'HAL'

(let [root (temp-directory "/" {:prefix "std-fs-candidate-"})
      file (path/join root "value.txt")
      written (write-bytes file (str/encode-utf8 "value"))
      bytes-value (read-bytes file)
      result [(= written file)
              (= (str/decode-utf8 bytes-value) "value")
              (file? file)
              (directory? root)
              (promise? written)
              (promise? bytes-value)]
      deleted (delete root {:recursive? true})]
  (if (and (= result [true true true true false false])
           (= deleted [file root]))
    :fs-candidate-ok
    (throw (ex-info "Synchronous std.fs candidate assertion failed"
                    {:result result :deleted deleted}))))
HAL
"$HARA" --project core --offline stdin < "$RUNNER_TEMP/fs-candidate-check.hal"
apply_candidate core/lib/src/std/fs.hal
rebuild_embedded_library
"$HARA" --offline run core/lib/src/std/fs.hal
cat > "$RUNNER_TEMP/fs-written-check.hal" <<'HAL'
(do
  (require 'std.fs)
  (let [root (std.fs/temp-directory "/" {:prefix "std-fs-written-"})
        file (std.fs.path/join root "value.txt")
        written (std.fs/write-bytes
                 file
                 (std.foundation.string/encode-utf8 "value"))
        bytes-value (std.fs/read-bytes file)
        result [(= written file)
                (= (std.foundation.string/decode-utf8 bytes-value) "value")
                (std.fs/file? file)
                (std.fs/directory? root)
                (promise? written)
                (promise? bytes-value)]
        deleted (std.fs/delete root {:recursive? true})]
    (if (and (= result [true true true true false false])
             (= deleted [file root]))
      :fs-written-ok
      (throw (ex-info "Written synchronous std.fs assertion failed"
                      {:result result :deleted deleted})))))
HAL
"$HARA" --project core --offline stdin < "$RUNNER_TEMP/fs-written-check.hal"

# The Foundation source change is documentation-only, but it is still a .hal
# edit and therefore receives the same complete-candidate/fresh-process cycle.
FOUNDATION_CANDIDATE="$CANDIDATES/core/lib/src/std/foundation.hal"
"$HARA" --project core --offline stdin < "$FOUNDATION_CANDIDATE"
cat "$FOUNDATION_CANDIDATE" > "$RUNNER_TEMP/foundation-candidate-check.hal"
cat >> "$RUNNER_TEMP/foundation-candidate-check.hal" <<'HAL'

(if (= 2 (inc 1))
  :foundation-candidate-ok
  (throw (ex-info "Foundation candidate assertion failed" {})))
HAL
"$HARA" --project core --offline stdin < "$RUNNER_TEMP/foundation-candidate-check.hal"
apply_candidate core/lib/src/std/foundation.hal
rebuild_embedded_library
"$HARA" --offline run core/lib/src/std/foundation.hal
cat > "$RUNNER_TEMP/foundation-written-check.hal" <<'HAL'
(do
  (require 'std.foundation)
  (if (= 2 (inc 1))
    :foundation-written-ok
    (throw (ex-info "Foundation written assertion failed" {}))))
HAL
"$HARA" --project core --offline stdin < "$RUNNER_TEMP/foundation-written-check.hal"

# The focused .hal fixture is also evaluated before and after writing.
FACADE_CANDIDATE="$CANDIDATES/core/lib/test/std/fs/facade_test.hal"
"$HARA" --project core --offline stdin < "$FACADE_CANDIDATE"
apply_candidate core/lib/test/std/fs/facade_test.hal
"$HARA" --offline run core/lib/test/std/fs/facade_test.hal
"$HARA_TEST" --root core core/lib/test/std/fs/facade_test.hal

apply_candidate \
  core/java/src/test/java/hara/truffle/StdFsTest.java \
  core/spec/std/filesystem.md \
  core/spec/std/foundation-architecture.md
rebuild_embedded_library

git diff --check
test -z "$(git diff --name-only -- core/lib/src/std/fs/path.hal core/rust/hal-src/std/fs/path.hal)"
! rg -n 'broader std\.fs remains planned|portable promise-based filesystem operations|Returns a promise of entry maps' \
  core/lib/src/std/foundation.hal \
  core/lib/src/std/fs.hal \
  core/lib/src/std/fs/walk.hal \
  core/spec/std/filesystem.md \
  core/spec/std/foundation-architecture.md

python3 - <<'PY'
import subprocess
expected = {
    "core/java/src/test/java/hara/truffle/StdFsTest.java",
    "core/lib/src/std/foundation.hal",
    "core/lib/src/std/fs.hal",
    "core/lib/src/std/fs/walk.hal",
    "core/lib/test/std/fs/facade_test.hal",
    "core/rust/hal-src/std/foundation.hal",
    "core/rust/hal-src/std/fs.hal",
    "core/rust/hal-src/std/fs/walk.hal",
    "core/spec/std/filesystem.md",
    "core/spec/std/foundation-architecture.md",
}
actual = set(subprocess.check_output(
    ["git", "diff", "--name-only"], text=True).splitlines())
if actual != expected:
    raise SystemExit(
        "unexpected synchronous std.fs diff\n"
        f"missing={sorted(expected - actual)}\n"
        f"extra={sorted(actual - expected)}"
    )
PY

git status --short

# Required Hara verification and regressions.
"$HARA_TEST" --root core \
  core/lib/test/std/fs/facade_test.hal \
  core/lib/test/std/fs/path_test.hal
"$HARA_TEST" --root core core/lib/test/std/foundation_test.hal
cargo test --manifest-path core/rust/Cargo.toml file::tests --lib -- --nocapture
cargo test --manifest-path core/rust/raw/Cargo.toml
(
  cd core/rust/web
  npm ci
  npm run test:hta
)
mvn -B -Ptruffle --file core/java/pom.xml \
  -Dtest=HaraLogicalPathTest,HaraFileProviderTest,StdFsTest test

# Publish one clean implementation commit from the exact main revision tested.
rm hara-specs-registry
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git checkout -B "$FINAL_BRANCH" "$BASE_SHA"
git add \
  core/java/src/test/java/hara/truffle/StdFsTest.java \
  core/lib/src/std/foundation.hal \
  core/lib/src/std/fs.hal \
  core/lib/src/std/fs/walk.hal \
  core/lib/test/std/fs/facade_test.hal \
  core/rust/hal-src/std/foundation.hal \
  core/rust/hal-src/std/fs.hal \
  core/rust/hal-src/std/fs/walk.hal \
  core/spec/std/filesystem.md \
  core/spec/std/foundation-architecture.md
git diff --cached --check
git commit -m "Make std.fs synchronous"
git push --force origin HEAD:refs/heads/$FINAL_BRANCH
