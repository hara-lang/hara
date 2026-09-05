#!/bin/sh
# Exercise Make-based source installs without requiring Cargo, Maven, or Java.
set -eu

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
HARA_ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/../.." && pwd)
WORK=$(mktemp -d "${TMPDIR:-/tmp}/hara-make-install.XXXXXX")
trap 'rm -rf "$WORK"' EXIT INT TERM

PREFIX=/usr/local
STAGE="$WORK/stage"
FAKE_RUST="$WORK/hara"
FAKE_JAR="$WORK/hara-truffle.jar"
FAKE_JAVA="$WORK/java"
LOG="$WORK/java.args"
EXPECTED="$WORK/java.expected"

cat > "$FAKE_RUST" <<'RUST'
#!/bin/sh
printf 'fake-hara:%s\n' "$*"
RUST
chmod 755 "$FAKE_RUST"
printf 'fake-jar\n' > "$FAKE_JAR"

make -C "$HARA_ROOT" --no-print-directory install-rust-files \
  DESTDIR="$STAGE" PREFIX="$PREFIX" RUST_BINARY="$FAKE_RUST"

test -x "$STAGE$PREFIX/bin/hara"
test "$("$STAGE$PREFIX/bin/hara" version)" = 'fake-hara:version'

make -C "$HARA_ROOT" --no-print-directory install-truffle-files \
  DESTDIR="$STAGE" PREFIX="$PREFIX" TRUFFLE_JAR="$FAKE_JAR"

test -x "$STAGE$PREFIX/bin/hara-truffle"
test -r "$STAGE$PREFIX/share/hara/hara-truffle.jar"

cat > "$FAKE_JAVA" <<'JAVA'
#!/bin/sh
printf '%s\n' "$@" > "$HARA_TEST_LOG"
JAVA
chmod 755 "$FAKE_JAVA"

HARA_TEST_LOG="$LOG" \
HARA_JAVA="$FAKE_JAVA" \
HARA_RUNTIME_JAR="$STAGE$PREFIX/share/hara/hara-truffle.jar" \
  "$STAGE$PREFIX/bin/hara-truffle" eval '(+ 19 23)'

printf '%s\n' \
  '-jar' \
  "$STAGE$PREFIX/share/hara/hara-truffle.jar" \
  'eval' \
  '(+ 19 23)' > "$EXPECTED"
cmp "$EXPECTED" "$LOG"

make -C "$HARA_ROOT" --no-print-directory uninstall \
  DESTDIR="$STAGE" PREFIX="$PREFIX"

test ! -e "$STAGE$PREFIX/bin/hara"
test ! -e "$STAGE$PREFIX/bin/hara-truffle"
test ! -e "$STAGE$PREFIX/share/hara/hara-truffle.jar"

printf 'make install checks passed\n'
