# GLSL original migration inventory — 2026-09-15

Authority: Foundation fe54cc866473a888bfbd4ce2d3de6d8011f09f72,
src/tahto/model/spec_glsl.clj and test/tahto/model/spec_glsl_test.clj.
Both complete files were inspected. The original GLSL model and its path-matched
tests are now installed; full core-facade integration remains unfinished.

The source contains eight live definitions: +types+, +typeops+, +features+,
+template+, +grammar+, +meta+, +book+, and +init+. Preserve their literal
groupings and composition. The trailing comment is retained source, not a
request to implement gl-def. The original grammar tag is :gl whereas the Book
language is :glsl; do not silently normalize that difference.

Required existing owners are lang.base.book, lang.base.emit,
lang.base.grammar, lang.core.script, and lang.base.util. Collection operations
and template/$ need explicit native mappings. Source does not require the
full lang.core facade. The tests do, solely to call emit-as; lang.core.impl
already owns that function. Any focused test import adaptation must be recorded,
and does not certify the eventual full facade.

Historical tests contain two exact-output assertions:

1. A shader defrun with attribute/uniform declarations and a void main body.
2. A sampler2D uniform declaration and vec4 constructor assignment.

Preserve their exact whitespace, numeric literals, metadata, and shader symbol
spelling (including u_Persp versus u_persp). The first fact refers to CANARY with
:adopt true; CANARY is not a missing source function to invent. The second
historical fact has no refer metadata. Additional native contract tests should
cover type operations and the module-import template, not replace these facts.

## Executed planner probe

The existing Foundation nREPL was verified running on port 58936 outside the
sandbox. Invoked code.migrate/read-pinned-unit and code.migrate/plan-unit against
the pinned source, with no source writes or model installation.

- Source checksum: 9f03693566e99080d636110746bef686ac136e2af02a058ac74884ffda77f4b0
- Output checksum: 7432dbc859ce9cd7bf4c63bab1aff1e5a7769d61c6f11be14d67982ea1066bec
- Applied rules: [:foundation/std-lib-collection]
- Diagnostics: []

Despite empty diagnostics, the output retains tahto namespace/imports,
template/$, and collection/map-juxt while changing the collection import to
std.foundation. This is an incomplete intermediate plan, not a native candidate.
The current rewrite-model-source owner set only includes the JS/Lua/Python
XTalk callback source/test namespaces; GLSL is absent. Its reusable
rewrite-model-template-form can lower the template, but is not automatically
selected for this source. Existing rewrite-owned-source supplies structural
top-level replacement and indexed before-images; use it rather than introducing
a new text replacement engine.

Next: create a scoped replayable adapter using these existing operations,
verify native collection arities/owners, evaluate the complete generated model
and both historical emission assertions in a fresh native process, then follow
scaffolding, negative-control, guarded-write and repeat-generation checks.
The initial inventory above did not certify native behavior. The following
installation evidence supersedes that initial stage.

## Installed source/test pair

Installed src/lang/model/v1/spec_glsl.hal and its path-matched test, and added
the source to project.edn's explicit inventory. The source preserves all eight
original definitions, the :gl grammar tag, the :glsl Book language, and script
installation. It does not use grammar-api. Source transformation composes the
existing rewrite-owned-source, rewrite-grammar-form and
rewrite-model-template-form functions; no new replacement engine was added.

Both historical output assertions pass unchanged. Their import points directly
to the existing lang.core.impl owner of emit-as until the full facade is ready.
Each emission uses its own Library and resets it and its settings in finally.
Eight authored facts cover the model definitions: full 47-type inventory,
constructor descriptors, grammar extension, literal/punctuation configuration,
grammar tag/resolution, include-template output, Book fields, and installation.

Fresh native 0.1.33 evidence:

- Initial complete candidate: two historical assertions passed.
- Complete candidate with native contracts: 10 passed, zero failures/errors.
- Negative control replacing all expectations: zero passed, 10 failed,
  zero errors. All permanent expectations were retained in the written file.
- Written file: 10 passed twice in separate processes.
- Installed CLI scaffold preview and write: exactly one owner inventoried,
  zero changes and no new obligations. Installed incomplete/unchecked: empty.
- Broader run: emit 21, C 38, GLSL 10; total 69 passed, zero failures/errors.

The first scaffold diagnostic incorrectly assumed defs were excluded; it
identified all eight obligations, which were then authored. The first CLI probe
found zero units before manifest registration and was not accepted as coverage.
A subsequent probe incorrectly expected an empty edits vector; the actual plan
contains an unchanged edit record. Corrected checks verify owner count, changed
count and new obligations rather than treating an unchanged record as a change.

Foundation resources/code/migrate/candidates/lang_model_glsl_original_installed.edn
contains immutable source/test units, structural before-images, explicit test
adaptations, additions and executable replay. Read-back replay produced both
installed files byte-for-byte twice. The final blank line is an explicit,
idempotent artifact-layout step. The files did not exist before this migration;
no existing native owner was overwritten. No Foundation tahto source/test file
was changed, and no commit was created.

## Catalog registration

The GLSL target is now registered in Foundation resources/code/migrate/lang.edn
as :migration/lang-model-glsl. The existing rewrite-model-source dispatch owns
the scoped source/test adapters; it composes the existing structural operations.
The eight authored tests are catalog native additions, not replacements for the
two historical assertions. migrate-pinned-pair reproduces both installed files
byte-for-byte with no diagnostics, and repeat generation is identical.

Permanent test/code/migrate/glsl_test.clj passes through code.test/run:
2 passed, zero failed/throw/timeout. Both facts fail against deliberately wrong
generators and pass after restoration. Written handler reload preserves the
previous JS/Lua/Python callback migration results. A subsequent fresh native run
passes GLSL 10 and emit 21 (31 total). Recovery and registration evidence is in
lang_model_glsl_catalog_registered.edn alongside the installation record.

The target names its six registered source/test prerequisites. One important
catalog gap remains: lang.core.script is installed but its original namespace
does not yet have a target in this catalog (script-control and script-macro do).
This is recorded explicitly, not treated as an automatically reproducible
dependency closure. Next register the original script owner, then finish the
other original eager-model dependencies before restoring the complete core
facade. This model-level pass does not certify the full migration goal.
