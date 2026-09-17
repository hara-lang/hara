# Original type-verify boundary audit — 2026-09-15

Authority: Foundation revision `fe54cc866473a888bfbd4ce2d3de6d8011f09f72`,
`src/tahto/runtime/basic/type_verify.clj` and its path-matched test.

The original owns six functions: verify-temp-file, substitute-file,
verify-error-result, verify-exec-oneshot, verify-exec-file, and
verify-exec-twostep. Its three historical facts contain five assertions.
Native type_verify.hal instead has six existing structured-checker helpers;
none of the original verify-exec entry points is installed. Its path-matched
test file is absent. Preserve the existing helpers while adding the original
contracts and their tests; a file copy is not a completed migration.

## Fresh native process evidence

Native project runner, using an stdin-backed temporary test path:

- `File/temp-file "/private/tmp"` failed with file/not-found under the
  project-mounted filesystem. This is not evidence that host /private/tmp is
  absent.
- `File/temp-file "."` created a logical `/tahto_verify_...sh` path. Native
  File/read saw the written source, but `/bin/bash -n` returned exit 127 and
  “No such file or directory” for that same path.
- `File/temp-file (OS/cwd)` in the Hara project produced a host-absolute path
  usable by both File and Bash. One exact behavioral fact passed: valid
  `echo verified\n` returned exit 0 and preserved source bytes; invalid
  `if then\n` returned exit 2, empty stdout and syntax-error stderr. Finally
  removed the allocated file and its test-created compiled sidecar, with
  File/exists? returning false for both after cleanup.

The native provider implementation in rust/src/file.rs returns logical paths;
the project runner mounts the project root. Relative logical paths must not be
passed unchanged to host processes. This probe did not change production code
or claim that the original verify-exec-file has been installed.

## Next implementation constraints

Use the existing File/temp-file atomic allocator with the original prefix and
extension normalization. Resolve explicit roots to a host-visible path within
the mounted filesystem. Original root=nil uses the host temporary directory;
do not silently substitute cwd or assume a universal /tmp. Establish that
mapping before claiming the default arity is faithful.

Preserve placeholder replacement versus appended filename, unmodified source
on success, stderr-then-stdout error precedence, and cleanup of both the source
and its `c` sidecar on all exits. tool.sh/result-record retains nonzero exit,
stdout and stderr, unlike the oneshot raw vector that loses separate streams.
Shell option handling still requires an explicit audited mapping; do not drop
options just to fit tool.sh/run's narrower contract.

Retain the five historical assertions, add helper correspondence and real
process/cleanup assertions, prove negative controls, then register structural
migration rules and byte-exact regeneration. No source/test migration was
installed during this audit.

## Validated oneshot stage

Foundation record `resources/code/migrate/candidates/lang_runtime_basic_verify_oneshot_stage.edn`
retains pinned source/tests, the existing native source before-image, the
code.migrate plan, std.block structural adapters and before-images, the candidate
program, and executable replay assertions. Replay matched the complete program
twice before saving.

Three original functions are evaluated in this candidate: substitute-file,
verify-error-result and verify-exec-oneshot. Four native facts pass, including
both original oneshot expectations, placeholder edge cases, error precedence
and exception data, and actual Bash pipe/inline checking. Deliberately replacing
every expected result produced four failed facts and zero errors; restoring
the correct candidate produced four passes and zero errors.

Native with-redefs is absent. The two historical mock fixtures are explicitly
adapted to real shell processes with controlled exit/output; expected values
remain `body` and `bad syntax`. This is two of five historical assertions, not
full verifier parity. The candidate preserves the original oneshot docstring
and uses Promise/all plus a finally cleanup boundary for native processes.

No production source was replaced. File/default-root behavior, the remaining
historical assertions, shell-option parity, tests for preserved native helpers,
scaffolding, installation, and catalog registration remain outstanding.
