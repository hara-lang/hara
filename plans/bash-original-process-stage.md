# Bash original source and direct-process stage — 2026-09-15

Authority: fe54cc866473a888bfbd4ce2d3de6d8011f09f72. The complete original Bash
model is now structurally generated and evaluated in fresh native 0.1.33
processes. It is staged, not installed or fully migrated.

The adapter preserves literal feature/template composition, :sh grammar tag,
:bash Book language, :xtalk parent, and original script/install. It maps the
existing collection/form? owner to native form?, merge-nested to the prelude,
Java literal quote replacement to str/replace, and the keyword-only f/strn
callback to str/to-string. The unary Clojure conj in bash-dash-param is an
identity operation and is lowered accordingly. The owning namespace is
internal; top-level bash-vector becomes an ordinary testable definition.
Quoted target defn- remains target syntax. The three original quasiquoted
branch helpers were retained verbatim and pass their emission cases.

## Executed assertions

- Complete model load and initial helper/emission smoke: passed.
- Historical helper/emission stage: 20 facts, 30 assertions, all pass.
- Direct real-Bash stage: 22 facts, 62 of 64 historical assertions, all pass.
- Deliberately wrong expectations: zero passing facts, 22 failed, zero errors.
- Restored direct-process stage: 22 facts pass again in a fresh process.

The process fixture replaces !.sh with existing impl/emit-as plus the original
adapted oneshot/sh-exec (:raw true, :layout :flat). It runs real bash -c,
including bc, conditionals, function arguments, loops, here documents, brace
expansion and process substitution. The child cwd is Foundation's root so the
original project.clj directory assertion retains its meaning. No fixture file
was invented. This proves emitted-source/process behavior, not the original
registered !.sh frontend or lifecycle.

Two notification-server assertions remain deferred and are explicitly stored
in the stage record. Their original source, setup, metadata and all 64 original
assertions remain in the immutable test unit. Additional correspondence tests
for bash-vector and model data definitions are still required before source
installation, along with notification and original runtime-path validation.

## Silent title collision found and corrected in the adapter

The first emission run reported only 18 results for 20 selected facts. Native
code.test.registry indexes facts by namespace and description; it does not use
refer metadata to distinguish identical descriptions. Original pairs share
"quotes a string value" and "constructs pipeleft syntax". Those first 18
results were not accepted as complete stage coverage.

The staged adapter gives each fact an owner-qualified title and retains its
original title in :migration/original-title. The corrected run executes all
20 emission facts, and the process stage executes all 22 selected facts.
No expected output was changed. This collision class warrants checking other
migrated test assemblies; green summaries alone cannot rule it out.

## Reproduction

Foundation resources/code/migrate/candidates/lang_model_bash_original_process_stage.edn
contains both pinned units, source adapter and before-images, test assembly and
adaptation functions, two deferred operations, fixture boundaries and checksums.
The saved record was read back and regenerated twice. Source, staged tests and
complete native program are byte-exact, and source adaptation is idempotent.

No production Bash source/test file was written, no Foundation tahto file was
changed, no package was installed and no commit was made. The temporary stdin
test link was removed after all native processes completed. Next finish the
notification fixture and original Bash runtime registration, author the
remaining corresponding contracts, then scaffold, install and catalog the pair.
