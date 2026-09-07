# Faithful Foundation language migration through Clojure code.migrate

Status: approved; execution in progress. The goal includes the complete plan,
including emitter-generated JavaScript, Lua, and Python process bootstraps.
No commit or publication is requested.

## Required Foundation architecture restoration

The user explicitly requires removal of `lang.base.grammar-api`, not a rename
or a compatibility facade preserving its materialization architecture.
The pinned Foundation model flow is the authority:

1. Build each model's literal `+features+` through `grammar/build`,
   `grammar/build:override`, and `grammar/build:extend`, preserving shared
   XTalk group composition.
2. Merge the model's concrete `+template+` with emitter defaults and construct
   the concrete `+grammar+` with `grammar/grammar` and `grammar/to-reserved`.
3. Construct `+book+` with that grammar, parent, and metadata, then install it
   through the pinned script/library lifecycle (`script/install +book+`).

Do not retain grammar source declarations, profile/template coordinates,
facet materialization, or `+selection+` as the required book/DSL path.
Port and verify the model owners and their shared helpers against this direct
flow before adapting base consumers. In particular, the current temporary
preprocess-value mapping from `+grammar+` to `(:grammar +selection+)` must be
removed when the concrete model grammars are restored; it is not the intended
final architecture.

The inspected native JS model currently constructs a registry, resolves a
selection, embeds `:selection` and `:coordinate` into the book, and sets
`+init+` to the book without the pinned installation call. These are explicit
migration differences to correct, not host-representation exceptions.

Removal must reconcile the actual callers, including `lang.base.registry`,
compiler/emitter tests, V1 XTalk and target model sources, the V1 grammar
snapshot, and migration policy. Other target models already depending on the
removed layer must not be left with unresolved imports. Preserve before-images
and meaningful tests, migrate required behavior to its Foundation owner, and
delete the obsolete implementation/tests only after callers are converted.
Completion requires no executable `lang.base.grammar-api` references and
verified direct book installation, DSL staging, emission, and runtime bootstrap
behavior. Do not claim this restoration complete from a passing isolated model
or from mechanically deleting the namespace.

## Approved behavioral adaptations (2026-09-07)

The user approved loosening JVM-specific incidental behavior to unblock the
core language pipeline. Collection UUID seeds use documented deterministic
canonical hashing rather than JVM map-print order. Book types must retain all
supplied fields, lifecycle callbacks and lookup/update behavior, but need not
reproduce Clojure record identity or printing. Native exception classes,
incidental display formatting and equivalent numeric representations may differ
where generated-program behavior is unchanged. Emitted-code semantics,
dependency order, macro expansion, lifecycle cleanup, actual JS/Lua/Python
execution and emitter-generated bootstraps remain strict. Historical assertions
remain traceable; each intentional difference requires explicit adaptation and
tests, never silent suppression.

## Approved generator ownership change (2026-09-06)

The user selected Clojure `code.migrate` as the authoritative source/test
generator, not a temporary bootstrap for a second native generator. The
original native-engine requirements below are superseded only in generator
ownership: retain structural `std.block`/`code.query` matching and the full
language/runtime fidelity scope. Existing native repairs and user changes
remain protected; they are evidence to reconcile, not authoritative templates.

The user explicitly authorized edits to migration tooling, corresponding tests,
and migration resources inside `reference/foundation-base`. Its `src/tahto` and
`test/tahto` authority stays unchanged; language input remains pinned to
`fe54cc866473a888bfbd4ce2d3de6d8011f09f72`. This is a scoped Clojure exception,
not a workspace-wide relaxation of the native automation rule.

Execution evidence for the Clojure-owned path:

- Added a complete pinned BookModule source candidate, independent of the
  Book/Snapshot open-field runtime decision because BookModule declares no
  custom dependency protocol. `rewrite-book-module-form` and ordinary
  `plan-unit` now lower its declaration, private helper forms, assertions and
  Foundation collection aliases. Open-map construction retains extension
  fields and source metadata while legacy native structs remain recognizable.
  Four focused native checks pass against the complete generated namespace;
  all four fail against the current native implementation. The differences
  cover dropped fields/metadata and defaults, omitted polyfill dependencies
  in the one-argument query, retained self hard-link dependencies, and spaces
  instead of quote characters in module entry labels. Recursion-guard state
  is restored after the dependency probe. Repeated generation and indexed
  reconstruction of the original source are exact.
  Recipe: `resources/code/migrate/candidates/lang_base_book_module_source.edn`.
  No native BookModule file was installed or counted. Full historical tests
  still require actual cloned-library, JS and XTalk fixture dependencies.
  Existing native helper names and the selected-target-language materialization
  test must be reconciled explicitly; the latter differs from the pinned
  module-language context and must not silently disappear.
  Written/reloaded full migration tooling passes 343 checks, zero failures,
  throws or timeouts (58.512 seconds).

- Full pinned Book planning with the seven tested mutation adaptations was
  reporting zero diagnostics despite failing fresh native loading on
  `std.lib.impl`. The migration engine now reports an explicit
  `:migration/unlowered-declaration` for the owning `impl/defimpl Book`, and
  guarded generation rejects it before native execution. Alias and renamed
  imports are resolved, namespace transitions reset imports, and quoted or
  syntax-template target data remain untouched while executable unquotes are
  checked. Negative controls detect the previous detector and disabled
  declaration analysis. Written/reloaded engine tests pass 128 checks; the
  full tooling suite passes 340 checks with zero failures, throws or timeouts
  (56.566 seconds).
  Do not treat this as a runnable Book port. Its `impl/defimpl Book` boundary
  must preserve both arbitrary fields and Book-specific `IDeps` behavior.
  Native named structs support guest protocol implementations; ordinary maps
  do not have per-Book guest dispatch, and existing struct constructors drop
  extension fields. A fresh isolated capability probe further shows that
  custom `ILookup` returns `:custom` through `(get value :extra)`, while the
  equivalent keyword call `(:extra value)` returns nil. Thus custom lookup
  alone does not establish faithful keyword/collection behavior for a wrapper.
  Approval is requested for opt-in native open-field struct support for Book
  and Snapshot. No native runtime implementation or Book representation was
  changed while this decision is pending. The installed count remains 20.

- Installed the generated `lang.base.book-meta` source/test pair from the
  immutable Foundation authority. The structural rule preserves all pinned
  fields (including `teardows-module`), native `transforms` and
  `teardown-module`, arbitrary extension fields, source metadata and callable
  lifecycle behavior. Open maps retain the existing legacy struct predicate;
  reconstruction is idempotent. Display now sorts configured keys as pinned.
  Native `filter-values` and `book-meta-data` remain present. Both historical
  assertions and all six prior native checks remain; three additional checks
  cover the preservation boundary and legacy constructor. Written tests pass
  11 checks; the combined BookMeta, JS metadata and PostgreSQL metadata run
  passes 47 checks. Native correspondence reports six definitions, no
  incomplete tests and no unchecked tests. Scoped scaffold preview/write was
  run and all placeholders were replaced with semantic facts.
  The old native implementation fails the preservation/display controls
  (one failure, one error), and an incorrect legacy expectation fails its
  added fact. The permanent regeneration regression rejects deliberately
  changed output bytes. The written pair regenerates exactly from its review
  recipe, which retains exact native before-images:
  `resources/code/migrate/candidates/lang_base_book_meta.edn`.
  Written/reloaded full migration tooling passes 331 checks, zero failures,
  throws or timeouts (55.540 seconds).
  This is the twentieth locally installed source/test pair. Its reviewed
  dependency rule and two-assertion historical contract are now registered:
  the central catalog has 20 installed candidates and five planning targets.
  The permanent regeneration regression uses `migrate-pinned-pair` directly,
  without appending an ad hoc target. Guarded preparation and verification
  also pass in the fresh contained stage
  `target/migration-book-meta-b1fc8f7e-a6dc-4476-bf86-b03d9271aa8c`:
  11 checks, no failures/errors/timeouts/skips; a second pinned generation
  preserves both installed hashes. Direct book/library installation and
  removal of `grammar-api` remain unfinished.

- Added a source-owned structural candidate for the seven Book mutation
  functions. Foundation mutations return `[change-data updated-book]`, while
  current native consumers expect only the updated book. Native probes of the
  translated functions match pinned change-data and resulting state in ten
  replacement, nested merge, missing-path, repeated deletion and empty-batch
  cases. The test reads the Book source at the pinned revision and verifies
  the Clojure atom helper source used by its differential oracle is byte-exact
  to that revision. Disabling the adapter fails the regression; applying it
  twice is stable and quoted calls remain unchanged. No new native atom API
  is introduced. This rule is not an installed Book port: native callers,
  snapshot/library mutation contracts, complete source/test generation and
  the full Book correspondence still need coordinated migration.
  Written/reloaded full tooling passes 332 checks, zero failures, throws or
  timeouts (56.434 seconds).

- Added and verified the shared model-template migration boundary. The rule
  preserves literal target symbols (including `Promise`) and scoped template
  metadata. Quote-aware engine traversal now treats syntax-template bodies as
  data and migrates only matching unquoted expressions; its regressions detect
  the previous target equality rewrite and still report real host interop.
  Chained reader discards are structurally redistributed without deleting
  disabled source, changing its reader value, or touching string literals.
  Negative controls reject the previous template/quote behavior and the native
  reader rejects the original chained-discard fixture. Written/reloaded full
  migration tooling passes 327 checks, zero failures, throws, or timeouts
  (written/reloaded suite: 54.596 seconds).

  Complete pinned shared JS/Lua model source drafts now evaluate: 18 existing
  JS native checks pass with one remaining socket-shape error, and eight Lua
  checks pass with three errors in tests expecting the prior portable bit-loop
  shape. No model source/test pair is installed or counted yet. Draft hashes,
  target coordinates, generation steps, and remaining gates are recorded in
  `resources/code/migrate/candidates/lang_model_js_lua_source_drafts.edn`.
  Ordinary pinned-input `plan-unit` now owns template and discard adaptation,
  retains original input and adaptation before-images, and regenerates both
  recorded candidate hashes exactly. The draft recipe includes the required
  rule flag; these targets remain unregistered and uninstalled. Migrating the
  historical emission tests and restoring direct model/book installation
  remain required. `grammar-api` has not yet been removed.
  Current native `core.impl` lacks the pinned `emit-as` owner, while
  `core.script` lacks `install`; restore their Foundation contracts, including
  library installation, runtime language registration, and grammar macro
  publication, rather than routing historical tests through selection-based
  compiler substitutes.

- The user prioritized required JS/Lua model migration before further base
  test convergence. After the installed grammar repair, pinned value
  preprocessing runs all seven facts: four pass, three fail, and none error.
  Two failing facts are target-model differences: JS native-type expansion
  omits custom constructor-name detection, and Lua prototype creation expands
  to an immediate function call instead of the pinned block. Preserve the
  original base expectations and port the owning model dependencies first.
  The third failure is separate: a returned callback does not compare equal
  to the original callback even though non-callable entry fields match. It
  must not be silently suppressed or attributed to the JS/Lua models.

- Installed a scoped grammar override dependency repair through
  `code.migrate.lang/rewrite-grammar-override-form`: native entry overrides now
  recursively merge nested maps, preserving pinned argument metadata and the
  existing native macro-lowering refresh. The exact before-images and reviewed
  candidate are retained in `lang_base_grammar_override.edn`. Its initial
  installation status is a pre-install review snapshot; this ledger records
  the subsequent installation. Generated source bytes and the complete written
  test candidate match exactly.

  Fresh written-file validation passes 40 grammar checks. All 33 new semantic
  correspondence facts were individually fault-injected in one negative run:
  33 failed and the seven preserved native checks passed. The original source
  also fails the nested-argument-metadata regression. Five downstream files
  (preprocess-value, preprocess-assign, preprocess-resolve,
  preprocess-staging, emit-preprocess) pass 48 checks with no failures or
  errors. These are existing native consumer checks, not historical parity
  certification. The installed namespace-pair count remains 19: complete pinned
  grammar migration and the remaining preprocessing ports are still unfinished.

- Added complete pinned preprocess-value source/test adaptation and dispatcher
  wiring. All six existing native checks pass. Three historical let-scoped
  facts contain eight loose nested assertions; each actual/expected comparison
  now executes within the same lexical scope, with none left unchecked.
  Source/test generation retains original inputs, produces no diagnostics,
  and is repeatable. The permanent regression rejects a disabled adapter.
  Written/reloaded migration tooling passes 319 checks, zero failures/throws/
  timeouts; Foundation diff-check passes. The reviewed pair, dependency routes
  and before-images are in
  `resources/code/migrate/candidates/lang_base_preprocess_value.edn`.

  Historical execution remains four facts passed, two failed and one errored.
  One comparison exposes native equality of maps containing callable values;
  its assertion representation needs explicit review. Target behavior is not
  weakened: JS native-type expected forms still retain custom constructor-name
  detection, which the native js-type-native macro omits. Lua prototype lifting
  lacks its argument list because native grammar/nested-merge-entry uses shallow
  merge whereas pinned build:override uses collection/merge-nested. A minimal
  fresh native probe confirms both arglists and an untouched nested option are
  dropped; exact evidence is saved in
  `resources/code/migrate/fixtures/preprocess_value_grammar_override.edn`.
  Lua's prototype macro also differs from the pinned block-valued expansion.
  The spec-writing workflow places the merge repair in shared grammar and the
  macro repairs in their owning target specs. No native value-preprocessor file
  was overwritten; installed count stays 19 pending those dependency fixes.

- Continued independent preprocessing work while the caller-context decision
  remains unanswered. Native `rewrite-tail-return` omits the pinned `cond` and
  `try` branches: both fresh baseline probes incorrectly wrap the entire form
  in an assignment. Added the tested structural `rewrite-assignment-tail-form`
  adapter, preserving conditional/catch tail assignment and untouched finally
  cleanup, plus the existing native `spec/as-list` normalization. Reused tail
  iterators must be materialized: native rest destructuring otherwise consumes
  lexical body forms before the final expression is read. The adapter expands
  grouped let/let* cases and materializes lexical and split body/handler tails.
  Nine existing native checks plus three new branch checks pass (12 total).
  The permanent regression reads the pinned function, verifies its checksum,
  exact generated candidate and repeatability, and rejects a disabled adapter.
  Written/reloaded migration tooling passes 318 checks with zero failures,
  throws or timeouts; Foundation diff-check passes. Reviewed function candidate,
  before-images and probes are saved in
  `resources/code/migrate/candidates/lang_base_preprocess_assign_tail.edn`.
  This rule is not yet a complete preprocess-assign source/test migration:
  ProtectedHead preservation, the full source recipe and historical JS/Lua
  grammar/fixture tests still need integration. No assignment HAL file was
  overwritten, and installed pair count remains 19.

- Probed pinned `impl-entry/create-common`, `create-fragment`, and
  `create-macro` against the installed open BookEntry. Existing prelude
  `qualified-keys` handles the required static/rt/api filtering without a new
  compatibility helper. Explicit common metadata and fragment construction
  pass. Three contracts remain red: default namespace is the defining
  `lang.core.impl-entry` rather than the caller; a wrapped single-arity fn is
  rejected by native eval; and a caller-local macro helper is unbound. The
  five-check native probe reports two passed, one failed and two errors.
  Native function execution selects the defining namespace; caller scoping is
  restricted to specific Foundation helpers. Entry metadata `:namespace` is
  not equivalent to Foundation's dynamic evaluation namespace, so using it as
  an automatic substitute would change the source contract. Saved exact
  selected authority forms, native before-image, candidate and failing probes
  in `resources/code/migrate/candidates/lang_core_impl_entry_context.edn`.
  No impl-entry native file was changed and the installed count remains 19.
  Preserving implicit caller context requires a runtime-context design decision;
  the approved incidental-JVM relaxation does not authorize a different macro
  resolution contract. Full multi/variadic template arities also remain pending.

- Installed and promoted the pinned `tahto.common.book-entry` source/test pair
  as native `lang.base.book-entry`. Production catalog: 24 targets, comprising
  19 installed candidates and five planning targets. Common inventory and
  dependency graph match the new target; uncataloged counts are 177/201 and
  98/122 respectively. `rewrite-book-entry-form` uses structural before-images,
  validates the pinned 20-field inventory, retains the legacy struct, and
  constructs tagged open maps with all extension fields and source metadata.
  Historical predicates execute explicitly before the native matcher; neither
  historical fact nor any of their three assertions was removed. All three
  native-only helpers and six existing native assertions are preserved.
  The six native checks are represented as uniquely identified facts because
  `code.manage` does not index their prior Test/check representation. Five
  additional facts cover extension construction/association, reconstruction,
  metadata, executable callbacks, and the legacy struct constructor. All four
  new preservation checks fail against the old implementation (two failures,
  two errors); the generated pair passes all 13 facts.

  Scaffold preview exposed seven obligations; scaffold write was performed,
  then all placeholders were replaced by the validated generated facts.
  Post-write preview/write both report zero changes. Final native analysis
  sees all seven declarations with no missing, TODO or unchecked tests.
  Fresh five-file Book/Library/snapshot/resolver integration passes 111 checks.
  Production-catalog regeneration matches both installed files byte-for-byte;
  the permanent regression checks those bytes directly. Before-images and the
  reviewed recipe are in `resources/code/migrate/candidates/lang_base_book_entry.edn`;
  its planning/installation fields describe the pre-install review snapshot,
  while this ledger and the production catalog record installation.

  Final inspection caught an existing single-arity rule bug that discarded
  symbol/argument metadata. The scoped renderer now preserves it; its permanent
  regression fails before the fix and passes afterward. BookEntry's supported
  `:public true` marker is restored. A byte-regression caught the resulting
  formatting change before final verification; the written file now uses the
  exact regenerated output. Written/reloaded migration tooling passes all 317
  checks with zero failures, throws or timeouts. Foundation diff-check and the
  changed Hara pair's diff-check pass; immutable authority trees are unchanged.
  Next resolver dependency: faithful macro/entry construction and JS/Lua Book
  fixture setup, not a substitute fixture map.

- Traced resolver historical fixture construction to `impl-entry/create-macro`
  and BookEntry extension-field retention. Fresh native baseline checks prove
  `book-entry` drops `:op`, `:static/return`, and arbitrary keys; associating
  an extra field with the returned struct throws `unknown struct field: op`.
  A candidate using the approved open-map representation preserves every
  provided field, defaults, source metadata and executable templates, while
  `book-entry?` still recognizes legacy structs. It passes six existing native
  checks plus four extension/reconstruction checks in a fresh process. Exact
  source/test before-images and the tested exploration are saved in Foundation
  `resources/code/migrate/candidates/lang_base_book_entry_exploration.edn`.
  Read-back matches the tested data and source SHA-256; installed source is
  unchanged. This is exploration, not a completed migration: encode from the
  pinned structural source, retain both historical facts (three assertions),
  scaffold permanent tests and validate Book/Library consumers before install.

- Persisted `rewrite-preprocess-resolve-form` and its source-only `plan-unit`
  dispatch. Structural adaptations preserve skip-dependency branches, expand
  grouped case constants, replace fnil dependency accumulation, and retain
  native named/docstring/attribute template arguments. Two permanent tooling
  facts cover exact transformed forms, metadata, quotes, repeat generation,
  original pinned source/checksum, and three native skip-dependency assertions.
  Both facts reject an identity adapter and pass after restoration. The full
  written/reloaded migration suite passes 315 checks with zero failures,
  throws or timeouts; Foundation diff-check passes. The generated candidate
  also passes all 12 existing resolver checks plus the three new checks in a
  fresh native process. No resolver `.hal` files were overwritten, and the
  production catalog is unchanged: 18 installed pairs. Resolver historical
  coverage remains pending: five facts contain 13 assertions and require real
  JS/Lua Book fixtures, macro construction, and Library import/entry setup.
  Native `lang.core.impl-entry` currently has emission operations rather than
  the pinned `create-macro` constructor, so this dependency must be reconciled;
  simplified fixture maps are not a substitute for the historical setup.

- Promoted emit-common into the production catalog: 18 installed candidate
  pairs and five planning targets (23 total). The installed-pair regression
  now reads the production catalog directly, without substituting the reviewed
  candidate target. Namespace routing for emit-preprocess carries the mandatory
  pinned source evidence; catalog validation rejected its omission before the
  correction. Catalog-count and pinned-workflow expectations are updated.
  Written/reloaded tooling passes all 313 checks with zero failures, throws or
  timeouts. Inventory and dependency graph reflect the promoted common owner.

- Started preprocessing fidelity checks in dependency order. At the immutable
  authority, preprocess-resolve honors *macro-skip-deps* in process-code-entry,
  process-fragment-entry and process-namespaced-symbol. Fresh native probes
  fail all three contracts: both dependency accumulators are still mutated and
  fragment bodies are expanded instead of returning the resolved symbol.
  These are semantic regressions, not approved incidental JVM differences.
  Historical resolver tests also depend on emit-prep JS/Lua fixtures and core
  library/entry construction; preserve those setup contracts while porting.
- Installed the reconciled emit-common source/test pair after confirming both
  current files matched the reviewed native before-images. Its first native
  integration run exposed a retained native grammar convention: token spellings
  may be scalar strings such as {:token {:nil "null"}}, whereas the Foundation
  implementation destructures option maps. The scoped `rewrite-emit-token-form`
  adapter normalizes only these strings to :as option maps and uses native Var
  recognition/dereferencing for token callbacks. Existing map options, quoted
  forms, other owners and metadata remain unchanged; repeat adaptation is
  identical. The negative control rejects an identity adapter. Native contracts
  cover nil/boolean/empty spellings, map options, ordinary functions and Var
  callbacks. All 130 common facts now pass, and the seven-file native emitter
  integration passes 206 facts with zero failures/errors/timeouts (common, data,
  assignment, block, function, top-level and main emitter). Both installed
  files exactly match regeneration from the reviewed candidate. The permanent
  reconstruction regression now checks installed bytes as well. Written/reloaded
  tooling passes 313 checks with zero failures, throws or timeouts, and
  Foundation diff-check passes. Final on-disk native CLI scaffold preview and
  write both exit zero with zero changes; the aggregate assertion passes.
  Production catalog promotion is still pending;
  do not count this as full emitter/book/preprocessing pipeline completion.

- Reconciled the complete common emitter with all 71 existing native facts.
  The combined candidate passes 129 native facts (58 historical plus 71 native),
  retaining all 86 historical assertions and all three native-only helpers.
  Existing native random tests asserted fixed values 1 and 0.5; their reviewed
  replacement checks actual integer/double bounds and callback argument
  preservation. Native predicate construction and stored-Var invocation are
  explicitly adapted. All other native facts retain their assertions. The
  native correspondence audit passes for all 70 definitions, with no missing,
  incomplete or unchecked tests. No native common file has been overwritten.
  The REPL connection failed and the managed JVM restarted; reconstruction from
  written rules and original/native sources again passed all 129 facts.
  `resources/code/migrate/candidates/lang_base_emit_common.edn` now retains the
  complete reviewed target, dependency rule and SHA-256-checked native
  before-images so future work does not depend on ephemeral REPL state.

- User explicitly prioritized all preprocessing libraries alongside the key
  base emitters/books. Remaining native owners are preprocess-input,
  preprocess-value, preprocess-assign, preprocess-resolve, preprocess-staging,
  and emit-preprocess; preprocess-base is already migrated. There is no separate
  preprocess-macro namespace in this tree. Validate their staging and emission
  consumers, not merely successful namespace loads.

- Implemented the approved collection UUID boundary in the Clojure-owned
  structural adapter `rewrite-emit-uuid-form`, now dispatched by
  `rewrite-emit-common-form`. Map entries and set elements sort recursively
  rendered text; lists/vectors preserve order. Numeric and string seed hashes
  retain their previously validated behavior. All 67 existing seed fixtures
  pass, plus five nested/order cases and a random UUID version/variant check
  (73 native checks). The old map-print hashing fails the new regression.
  Original definition preamble/metadata and repeat-adaptation identity pass.
  Native Base and Bits owners are now recognized by the host-interoperability
  gate, consistent with registry native-protocol conformance; unknown JVM
  owners remain diagnostics, with before/after regression evidence.
  Regenerating the complete pinned emit-common source/test through the written
  rules produces zero diagnostics and passes all 58 historical facts with all
  86 assertions retained. Written/reloaded full tooling validation passes 311
  checks with zero failures, throws or timeouts; Foundation diff-check passes.
  The seed fixture records the approved recursive canonicalization boundary
  while retaining its immutable authority hash and UUID values.
  The common pair is not installed yet: preserve and
  reconcile existing native additions/tests before promoting its catalog entry.

- Closed the grammar-macro installed-pair regression and inventory follow-up
  (2026-09-07). The permanent `pinned-grammar-macro-native-parity` fact verifies
  fourteen historical assertions, eleven/zero source/test adaptations, retained
  `form` and `all-but-last` helpers, both installed byte strings, repeated-plan
  equality, both recovery hashes and original Git before-images, and all 41
  native facts. An injected extra newline makes the byte-parity regression
  fail; the restored candidate passes. After writing and reloading the facts,
  all eleven migration tooling namespaces pass 309 checks with zero failures,
  throws or timeouts. Catalog-count and pinned workflow expectations include
  the 22nd target. All 201 inventory records and 122 dependency-graph nodes
  agree with the live catalog; 179 inventory namespaces and 100 graph owners
  remain uncataloged. Foundation `git diff --check` passes. No authority source
  or tests were edited and no commits or publication were performed.

- Grammar-macro installation readback (2026-09-07): the production catalog now
  has 22 targets (17 candidate, five planning). Fresh migration from the pinned
  original source/test preserves fourteen historical assertions and records
  eleven source adaptations, zero test adaptations. Both written native files
  match generated bytes exactly; repeating migration yields identical plans.
  Both recovery before-images match their recorded SHA-256 and the files at
  the recorded native Git revision. Fresh native execution of the written
  path-matched macro tests passes 41 facts, and the V1 grammar consumer passes
  three more: 44 passed, zero failures/errors/timeouts. The attempted command
  including a source file as a test was rejected by the test-path gate; the
  corrected two-test-file invocation is the evidence above. The direct native
  code.manage CLI scaffold preview and write both exit zero with zero changes
  (one aggregate native assertion passed). Inventory snapshots,
  catalog-count regression expectations and permanent installed-pair integration
  still need updating; do not infer whole-pipeline completion from this count.

- Wired `:foundation/lang-grammar-macro` into `plan-unit` using the owning
  structural adapter and original pinned units. Permanent regression verifies
  fourteen historical assertions, eleven source adaptations and zero test
  adaptations, original input/checksum identity, indexed before-image recovery,
  repeated-plan equality and twelve passing native historical facts. The
  regression fails before dispatch is wired and passes afterward. Production
  catalog promotion and native installation are still pending.

- Preserved both existing macro helpers unchanged and all fifteen existing
  native assertions as uniquely identified facts with explicit alias ownership.
  This exposed two defects: dependency pruning removed `spec`, which the
  retained `form` helper needs; and native `seq?` did not recognize the
  persistent return form in `tf-lambda-arrow`, producing a nested return.
  `rewrite-unused-requires` now includes dependencies referenced by same-kind
  native additions, while still pruning unrelated/opposite-kind aliases. The
  macro adapter maps the owned lambda predicate to `form?` without rewriting
  quoted data or unrelated definitions. Both permanent regressions fail before
  their fixes. No assertion or native helper was removed to obtain parity.

- Written/reloaded tooling passes all 308 checks with zero failures, throws
  or timeouts; Foundation diff-check passes. The preserved macro candidate,
  rebuilt through the written dispatcher, passes all 27 native facts. In-memory
  native scaffolding inventories twenty source definitions and confirms the
  only missing corresponding facts are the five groups `+op-macro-arrow+`,
  `+op-macro-let+`, `+op-macro-xor+`, `+op-macro-case+`, and
  `+op-macro-forange+`. Author these contracts and retain the extra boundary and
  threading probes before installing. No macro source/test files were written;
  the installed count remains sixteen.

- Completed the grammar-owned threading adaptation in the written
  `rewrite-grammar-macro-form`. Foundation's actual core macro definitions
  confirm direct nesting, symbol-step wrapping and list-step metadata
  preservation. The owning callback now constructs those shapes for the
  declared `->`/`->>` operators, without changing Hara's global macros; other
  forms still delegate to native `macroexpand-1`. Missing initial operands are
  rejected by native indexed access, preserving the previous native rejection
  boundary rather than silently returning nil. This does not claim the native
  bounds exception is the JVM ArityException type or message.

- Added a permanent threading regression with twelve Foundation goldens:
  zero-step input, symbol steps, multistep ordering, empty-list and vector
  steps, and exact metadata. It checks definition metadata, idempotence,
  missing-operand rejection, unchanged nonmacro forms and retained fallback.
  Bypassing the threading adaptation fails; restoration passes all 75 focused
  checks. Written/reloaded full tooling passes 305 checks with zero failures,
  throws or timeouts; Foundation diff-check passes.

- Fresh native verification rebuilt from the written rule now passes all
  twelve historical grammar-macro facts/fourteen assertions. Four additional
  native probes pass, covering the twelve threading goldens, missing operands,
  nonmacro identity and one-step expansion of a registered non-thread macro.
  A probe-only nil-metadata projection was corrected to use the native empty
  map fallback; no Foundation/runtime API was changed. The workspace-referenced
  variadic-functions document was absent, so the actual native `apply`, list
  construction and threading helper definitions were inspected instead.
  Grammar-macro still needs production dispatch/catalog wiring, preservation
  of native additions/tests, recovery and installation audits. Sixteen pairs
  remain installed.

- Added and wrote `code.migrate.lang/rewrite-grammar-macro-form`. It reuses
  the finite-rest adapter, materializes only the owned repeatedly traversed
  cond/case/ternary binding expressions, and lowers the pinned nested else-rest
  pattern to a named tail plus `first`, retaining the original truthiness
  condition. Exact binding-pair matches make materialization idempotent;
  unrelated definitions and quoted forms are unchanged. Full-source second
  adaptation is identical. Historical native execution now reports eleven
  passes, one failure and zero errors; the sole historical failure remains
  the Foundation/native macro-expansion shape, with its expectation unchanged.

- The permanent adapter regression compares thirteen boundary inputs with
  pinned Foundation behavior, checks exact required structural rewrites,
  metadata, quotation protection and repeat identity. Replacing the adapter
  with identity fails; the restored adapter passes all 74 focused checks.
  Written/reloaded full tooling passes 304 checks with no failures, throws
  or timeouts, and Foundation diff-check passes. A fresh candidate reconstructed
  from the written rule passes all three prior native defects: false else,
  first-pair `cond :else`, and scalar lambda return. The rule is not yet wired
  into production dispatch/catalog and grammar-macro is not installed.
  Installed count remains sixteen.

- Began the next dependency, `grammar-macro`, from the immutable source/test
  pair: twelve functions, six literal groups, twelve historical facts and
  fourteen assertions. Existing rules generate both files without diagnostics,
  but initial native execution reports six passes, four failures and two
  errors. Diagnostic-free generation is therefore not evidence of parity.
  A scratch structural candidate reuses the existing finite-rest materializer
  and materializes repeatedly traversed `tf-cond`, `tf-case` and `tf-tcond`
  collections. That raises historical execution to ten passes, one failure and
  one error. Remaining seams are nested rest destructuring in `tf-if` and
  native `macroexpand-1` threading shape: native emits a temporary-binding
  form where Foundation expects direct nested calls. The scratch candidate
  is not yet an idempotent permanent adapter, catalog entry or installed pair.

- Verified three additional defects in the existing hand-written macro port.
  For `(if true :A false)`, Foundation omits the false else branch while native
  retains it. For `(cond :else :A)`, Foundation retains the first pair as an
  initial if while native emits an `(if nil nil)` branch. A scalar lambda
  result, `(fn:> [] 42)`, becomes `(fn [] (return 42))` in Foundation but raises
  an iterator-protocol error in native. A fresh native probe reports two
  failures and one error; direct Foundation execution, after checking loaded
  source bytes against the pin, confirms all three expected results. Preserve
  these cases in the permanent migration tests. No grammar-macro source/test
  files were written, and the installed count remains sixteen.

- Installed `tahto.base.grammar-spec` -> `lang.base.grammar-spec` from the
  immutable pin through the production catalog: sixteen installed source/test
  pairs and twenty-one catalog targets. The five pinned functions and all 32
  literal groups remain source-owned; the five previous native helper
  definitions are preserved unchanged. All eleven native assertions are
  retained as uniquely identified facts with explicit alias ownership. Exact
  native before-images, hashes and native revision are stored in
  `resources/code/migrate/recovery/lang_base_grammar_spec.edn`; the permanent
  integration test compares both before-images against that native Git pin.

- Hand-authored 38 additional facts covering every grammar group, symbol
  binding restoration, eleven argument-parser boundary inputs, ignored and
  generally evaluated mixins, and full-field canonical snapshot parity with
  idempotent callback normalization. All 38 reject deliberately incorrect
  expectations (16 preserved/historical passes, 38 failures, zero errors);
  restoration passes all 54 facts. Direct `code.manage.unit` scaffold,
  incomplete and unchecked checks pass for all 43 source definitions.

- The written source loads via `hara-native run`; its path-matched test and
  existing V1 snapshot test pass together in fresh runtimes: 57 facts, zero
  failures/errors. Catalog readback regenerates both installed files exactly;
  repeated planning is identical. The installed-byte integration regression
  rejects deliberately appended byte drift and passes after restoration.

- Fixed a native-fact generator omission exposed by the full-field test:
  `:target/test-requires` was honored only by the legacy Test/run emitter.
  `emit-test-facts` now preserves explicitly declared native dependencies after
  migration, with deduplication, input/checksum retention and idempotent output.
  Its regression fails before the fix and the written focused suite passes 22
  checks. The complete written/reloaded eleven-namespace tooling suite passes
  303 checks with zero failures, throws or timeouts.

- `hara --project . --offline manage scaffold lang.base.grammar-spec` hit the
  installed launcher's stack overflow. The direct native `code.manage.cli`
  preview/write invocation was interrupted at a 60-second evaluation deadline;
  retrying after confirmed interruption with a 180-second allowance completed
  successfully. Both scaffold operations report exit zero and zero changes;
  subsequent native tests and byte-exact checks still pass. This is an installed
  launcher/validation-duration limitation, not a failing grammar assertion.

- Updated the base inventory, dependency graph and README target counts.
  Foundation diff-check passes and `src/tahto`/`test/tahto` remain unchanged.
  Hara diff-check reports sixteen source trailing-whitespace lines retained
  from the immutable grammar authority; these were verified against the pinned
  source and are not a clean diff-check claim. No V1 artifact was overwritten,
  and no commit or publication was performed. Remaining language/runtime
  migrations and the previously recorded semantic decisions stay open.

- Wired `:foundation/lang-grammar-spec` into `code.migrate/plan-unit` through
  the existing `std.block`-owned adaptation path. The dispatcher now consumes
  original pinned units, not scratch pre-adapted strings: it records two
  source adaptations and one historical test adaptation, preserves original
  input/checksum identity, and generates all seven historical assertions with
  zero diagnostics. The generated pair passes five native facts.

- Made the owning historical mixin callback rewrite permanent. Only the
  quoted `tahto.base.grammar-spec-test/mixin-add-symbol` callback in the
  `format-defn-mixins` fact is retargeted; unrelated quoted forms, unowned
  facts, metadata and repeated-transform identity are preserved. Its new
  regression fails against the old adapter (72 passes/one failure), then the
  written/reloaded adapter passes all 73 focused checks.

- Added `pinned-grammar-spec-dispatch`: an explicit preflight target fixture
  checks original input/checksum identity, exact reconstruction from indexed
  before-images, adaptation counts, repeat-plan equality and native historical
  test execution. It retains the catalog's dependency mappings. Bypassing the
  adapter fails the fact; restoring it passes. The written/reloaded full
  eleven-namespace migration suite passes 301 checks with zero failures,
  throws or timeouts; Foundation diff-check passes. This wires the dispatcher,
  not the production catalog or native installation. Grammar-spec source/test
  and V1 artifact remain unchanged, and the installed count remains fifteen.

- Audited the grammar-spec candidate's versioned consumer before installation.
  A fresh native probe compares all 32 source-owned groups, including every
  entry field and callback Var identity normalized to its qualified symbol,
  against the corresponding checked-in V1 `*-data+` groups. Both current native
  source and the complete pinned migration candidate pass. Injecting an extra
  field into the compared entries fails and reports all 32 mismatching groups;
  the restored candidate passes. The candidate also passes the existing three
  V1 snapshot facts (including fingerprint validity and extension/overwrite)
  and the five historical grammar facts/seven assertions, in separate fresh
  native processes. The existing snapshot data does not need replacement for
  these formatter changes.

- Classified two audit limitations explicitly. Loading the entire native
  policy materializer exceeded the 60-second nREPL evaluation limit and was
  interrupted; no successful byte-exact materialization claim follows from
  that run. Its source template still contains the obsolete Foundation require,
  `ex-info`, and unresolved callable-Var hydration, unlike the checked-in V1
  artifact. Do not overwrite the artifact with this stale template. Also,
  concatenating separate test namespaces into one scratch file loses their
  alias context and produced three unbound-symbol errors; that combined run
  is invalid evidence, superseded by the passing isolated runs above. No Hara
  source or test files were changed by this audit. Fifteen pairs remain
  installed; grammar-spec still needs catalog wiring, native additions/test
  preservation, recovery evidence, correspondence, and regeneration checks.

- Expanded the grammar-spec structural adapter with the pinned argument-parser
  contract, inlining its initialization/validation semantics into the existing
  `format-fargs` definition without adding a native compatibility API. The
  current native parser incorrectly accepts empty/scalar bodies, retains a nil
  that the authority drops in one input shape, and unwraps a nested parameter
  vector. Eleven golden cases cover these distinctions plus documentation,
  attributes, single/multiple arities and exact error-body data. Existing
  native behavior fails the probe; the candidate matches every case.

- Added `grammar-fargs-parity`, guarding the grammar/function authorities
  against pinned bytes and comparing the adapted parser with Foundation.
  Definition metadata, preamble and idempotence are checked as well. A candidate
  with deliberately incorrect nil-slot handling fails the regression. The
  written adapter passes 72 focused checks; full tooling suite passes 299 with
  zero failures, throws or timeouts. The native candidate combining both
  formatter adaptations passes fifteen checks (eleven existing plus parser and
  three mixin probes).

- Whole-namespace grammar-spec preflight now generates the complete pinned
  source and historical test pair with no diagnostics; all seven historical
  assertions across five facts pass in a fresh native process. This scratch
  preflight explicitly adapts the quoted owning test callback namespace. It is
  not yet catalog-wired or installed: preserve the five existing native helper
  functions/tests, complete correspondence and grammar-group/consumer audits,
  record recovery, and account for derived versioned grammar artifacts before
  counting the pair. Installed count remains fifteen.

- Continued independently with `grammar-spec` while file/process bridge
  approval remains pending. Its current native mixin mini-evaluator throws on
  entries Foundation ignores and leaves nested form/map expressions
  unevaluated. A fresh three-case probe reports two failures and one error.
  Replacing only `format-defn-mixins` with the pinned function adapted through
  existing native `form?`, `resolve`, `deref` and `eval` passes those cases and
  all eleven existing native facts (fourteen total). This restores general
  evaluation rather than extending the limited evaluator one expression at a
  time; no new Hara surface is needed for this behavior.

- Added `code.migrate.lang/rewrite-grammar-spec-form` and its focused test.
  The owned structural rule adapts the callable Var and form predicate while
  preserving eval, ignored-entry behavior, quoted forms, list/vector shape and
  idempotence. An identity rule fails the regression (70 passes/one failure);
  the written rule passes all 71 focused checks and reconstructs the passing
  fourteen-fact native candidate from the pinned function. Full tooling suite
  passes 298 checks with zero failures, throws or timeouts. Native source,
  grammar groups and versioned generated grammar artifacts remain unchanged;
  this rule is not yet catalog-wired or counted as an installed pair.

- The remaining argument-parser boundary is Foundation
  `std.lib.function/fn:call-body`, which delegates to `fn:init-args` and rejects
  invalid callable body shapes. No corresponding native function was found.
  Complete that contract/adaptation before installing the full grammar-spec
  source/test pair. Installed count remains fifteen.

- File-verifier native boundary probe identified a host/provider mismatch.
  The native project test runner installs File with the project directory as
  provider root (`hara-native.rs`), while Process consumes host paths. Creating
  a temporary file beneath logical `test` returns `/test/...lua`. A real
  `/bin/test -f` returns exit 1 for that logical path and exit 0 for the known
  harness project-root-prefixed host path. A fresh positive probe also checks
  exact written/read source and absence after finally-based deletion; it
  passes. `/private/tmp` is not present inside this project File provider.
  Rewriting the already-created temporary file requires `:mode :replace`;
  default create mode correctly raises `:file/already-exists`.

- This is not evidence that concatenating `OS/cwd` is a valid general bridge:
  provider roots, process working directories and non-host mounts can differ.
  Inspected File's registered surface and the native Process/tool.sh boundary;
  no file-to-host-path mapping is exposed there. The probe's translation is
  valid only for its explicitly known project-root mount. No native or Rust
  source was changed, no filesystem capability was broadened, and owned probe
  files were deleted. File-backed verification needs an explicit approved
  capability-aware file/process boundary before porting the JVM temporary-file
  behavior. Requested direction before adding that surface; do not substitute
  stdin or a cwd-based path guess for Foundation's file-checker contract.

- Runtime verifier preflight: pinned `type-verify` has six function
  definitions (three private helpers) and five historical assertions. The
  existing native file has a different six-function stdin-only surface and no
  path-matched test file. Aggregate `basic_test.hal` checks result normalization
  and success/error projection, not Foundation's inline/file transports or
  cleanup. No native verifier source or tests were changed in this preflight.

- Added `pinned-verifier-execution-contract`. It evaluates the actual pinned
  function bodies with instrumented process and filesystem boundaries, without
  running a checker or creating temporary files. Assertions cover argv versus
  pipe input and effect order, forced spawn options and shell environment,
  unchanged successful source, unused trim callbacks, exact stderr/stdout
  diagnostic precedence, exception payloads, embedded and appended filename
  substitution, missing-extension rejection before effects, twostep
  delegation, and cleanup of both source and compiled companion files after
  success or failure. Both deletions are attempted even when writing and
  deletion throw. Deliberately removing source cleanup is rejected (44 passes,
  one throw); restored written focused suite passes 45 checks. Full tooling
  suite passes 297 checks with zero failures, throws or timeouts; Foundation
  diff check passes. This proves the authority contract, not native parity.

- Existing native boundaries include `File/temp-file` with prefix/suffix,
  exclusive creation and bounded retries, Promise-based File effects, and
  `tool.sh` argv execution/result access. Default temporary-directory selection
  and complete shell-option mapping still need validation before translating
  verifier execution. The next port must also preserve the existing native
  result helpers rather than overwrite them. Installed count remains fifteen;
  immutable language source/tests remain untouched.

- Installed `tahto.core.compile-links` -> `lang.core.compile-links` through
  the Clojure-owned structural pipeline. Four source blocks (namespace plus
  three functions) and one historical test block have reversible adaptation
  evidence. All nine pinned assertions remain. The two existing native
  filter/replacement helpers and all five existing native facts are retained
  as explicit additions with before-image provenance. Callable Vars are
  recognized through `type` and dereferenced before invocation; no native
  public API or runtime implementation was added.

- Added four native contract facts for filter boundaries, exact-key/callback
  precedence, exact path options, and dynamic default restoration after both
  return and failure. The first three deliberately incorrect expectations
  produce eight passes/three failures; the defaults negative produces eleven
  passes/one failure. Restoring correct expectations and running the written
  pair passes all twelve facts. The lifecycle consumer passes sixteen more
  facts in the same fresh invocation (28 total). CLI scaffold preview/write
  reports zero changes, but direct unit correspondence also indexes
  `*link-defaults*`; its stricter obligation is now satisfied. Final direct
  audit passes all three checks for six definitions, with no missing or
  unchecked facts.

- Final source review corrected generated string-companion requires and
  qualified calls to the preloaded `str/*` alias, and preserved the native
  internal role through explicit catalog configuration. Structural regression
  checks cover exact obsolete-import removal, list/vector shape safety,
  ownership, metadata, quoted forms and idempotence. The old alias adapter
  fails its expanded test; the written adapter passes all seventy focused
  checks. Byte-exact integration caught and corrected the corresponding test
  namespace metadata update. Both installed files now regenerate exactly,
  authority source/test blocks reconstruct exactly, and deliberately appending
  a newline to generated output makes the byte-exact checks fail.

- Final eleven-namespace migration suite: 296 passed, zero failed, throws or
  timeouts. Both repository diff checks pass; `src/tahto` and `test/tahto`
  remain unchanged. Recovery EDN readback equals the captured before-images.
  Inventory/graph readback equals the pinned scan with current catalog
  summaries: fifteen installed candidate pairs, five planning targets, 181
  uncataloged broad-scan namespaces; 102 of 122 declared graph owners remain
  uncataloged. This completes the compile-links slice, not the full language
  pipeline or the JS/Lua/Python runtime processes. No commits or publication.

- Compile-links preflight: the pinned owner has three functions and nine
  historical assertions; the existing native owner has five functions and five
  consolidated facts, all passing in a fresh process. A new callable-Var filter
  probe fails on that installed source with `Link filter is not valid`.
  Foundation's `std.lib.env/match-filter` accepts function Vars as well as
  functions. A native candidate recognizing `:std.native.Var` still fails
  because direct invocation is suppressed; explicitly dereferencing the Var
  before calling it passes all five existing facts plus the new probe. This
  uses existing `type` and `deref`, with no runtime surface expansion. The
  native candidate is evaluated only, not installed or counted as migrated.

- Added `pinned-compile-links-contract` to the Clojure migration tests. It
  guards both loaded source files against their exact pinned bytes and checks
  callable-Var filters, exact-key precedence and callback counts, suppressed
  callback failure, false-valued entries, relative/library paths, namespace
  prefix trimming, suffix fallback/precedence, label overrides and literal
  substitutions. Deliberately rejecting Vars yields 42 passes and one throw;
  automatic restoration and the written focused suite yield 43 passes. The
  full eleven-namespace tooling suite passes 294 checks with zero failures,
  throws or timeouts, and Foundation diff checks pass. Next: encode the
  verified Var adaptation with compile-links structural rules, preserve native
  helper/test before-images, and regenerate/validate the complete pair. No
  source under `src/tahto` or `test/tahto` was edited.

- BookMeta preflight found a representation gap, not just a rewrite spelling:
  native map->BookMeta drops arbitrary extension keys, including :lang and
  :extra. It also has different default field presence (:transforms and
  :teardown-module versus the pinned :teardows-module), and book-meta-string
  omits the authority's key sorting. The native runtime's named-value map
  constructor and native_struct_forms_issue_223 test explicitly specify closed
  struct behavior. No native source or runtime implementation was changed.

- Added a pinned BookMeta record contract regression. It guards the loaded
  source against exact pinned bytes and checks extension keys, default-key
  presence, record identity across assoc/dissoc, inverse removal of an added
  key, loss of record identity when removing a declared field, metadata
  preservation, and sorted display. A deliberately closed constructor is
  rejected with one throw/41 passes; automatic restoration and the written
  focused suite pass all 42 checks. Full tooling suite passes 293 checks with
  zero failures, throws or timeouts; diff checks pass and pinned source/tests
  remain unchanged. An exploratory protocol-backed wrapper is not sufficient
  evidence of a replacement: Base/field rejects immutable struct access.
  Requested direction before expanding into native open-record support.
  BookMeta is not cataloged or counted as migrated; fourteen pairs remain
  installed, with independent migration work still available.

- Built the pinned declared dependency graph for all base/core directory owners
  and the JS/Lua/Python model/process roots. The initial 65 roots reach 102
  namespaces in the directory inventory. Resolving the missing tahto.core and
  tahto.typed facade files exposes an additional 20-owner expansion through
  eager model registrations, giving 122 declared owners across 32 dependency
  layers. No declared internal edge remains unresolved or cyclic; all 47
  external namespace boundaries are explicit. The facade expansion is recorded
  separately and is not automatic authorization to add optional target-language
  delivery. Dynamic loading, macro expansion and quoted XTalk dependency audits
  remain open. The graph is saved as inventory/pipeline-dependencies.edn; exact
  EDN readback and every internal edge's ordering were verified. At this
  snapshot, 103 graph owners have no catalog target. No language source or
  runtime behavior changed, and fourteen source/test pairs remain installed.

- Installed lang.core.rewrite.unpack: fourteen installed pairs and nineteen
  catalog targets. The source uses a target-scoped structural predicate rule;
  only the owning historical predicate fact needs the shape-aware vector
  constructor adaptation. All four historical assertions are retained. Four
  hand-authored native facts preserve the previous native checks and add nil
  and sequence inputs, exact arity, callback order, error propagation before
  wrapper invocation, reusable results and empty input. Every added/retained
  fact rejects an incorrect expectation (four passes/four failures); restored
  written tests pass eight facts. Scaffold preview/write reports no changes;
  native correspondence audit passes three checks for four definitions.

- Unpack recovery preserves both prior native files in
  recovery/lang_core_rewrite_unpack.edn. Permanent integration verifies exact
  recovery of the one adapted test form, repeated-plan equality and byte-exact
  installed source/test regeneration. Written/reloaded tooling passes 292
  checks across eleven namespaces with zero failures, throws or timeouts.
  Diff checks pass, pinned language source/tests are unchanged, and refreshed
  inventory readback matches the authority data and current catalog status.

- Core rewrite milestone: all ten pinned tahto.core.rewrite namespaces now
  have installed native source/test pairs. Regeneration of every pair produces
  exact installed bytes. Running all ten native test files together passes 96
  facts; the Dart unpack consumer also passes 45 facts. This closes that family
  of ports, not the broader lang.base/core/runtime pipeline. Emitter-generated
  JS/Lua/Python process bootstraps and the outstanding semantic decisions remain
  open. No commits or publication were made.

- Added optional :rule/target-ids scoping to dependency-routes. This lets the
  existing structural token engine apply reviewed local alias replacements
  without introducing a namespace-specific source adapter or rewriting
  unrelated targets. Absent scope retains existing global routes; empty scope
  disables the route. The old engine fails the scope regression (112 passes,
  one failure); evaluated and written/reloaded code passes all 113 focused
  checks. Full migration-tooling suite: 290 passed, no failures/throws/timeouts.
  Diff-check passes. No installed Hara source changed in this step.

- Unpack candidate uses the scoped collection/form? replacement. Its four
  pinned facts initially produce three passes/one failure because equal quoted
  list/vector constants lose collection shape in a shared native compilation
  context. A direct native predicate probe confirms the collapse. The prior
  native fixture used a vector constructor. An unwritten, owning-fact-only
  adaptation of the exact quoted vector to an ordinary vector constructor
  preserves the expected false value and makes all four historical facts pass.
  Matching must explicitly check vector shape: Clojure equality alone also
  matches the list, as a rejected intermediate candidate demonstrated. This
  adaptation still needs permanent tests, recovery, catalog wiring and native
  installation. Unpack is not counted; thirteen pairs remain installed.

- Installed lang.core.rewrite.inline-do: thirteen installed pairs and eighteen
  catalog targets. Its pinned twelve assertions in ten facts preserve every
  behavior exercised by the previous three consolidated native facts. Added
  four hand-authored facts for the metadata alias, sequence/data boundaries,
  empty and single-value bodies, return arity, recursive map/set rewriting,
  quote preservation and normalization idempotence. Scaffold preview/write
  identified one missing alias fact; its placeholder is replaced. The native
  audit passes all three checks for four definitions, with no missing,
  incomplete or unchecked tests.

- Inline-do adaptation uses the native form predicate and preserves the
  forward declaration inside a non-executing comment. The existing native file
  passes its three tests with declare, while the combined candidate harness
  rejects executable declare; the candidate and written namespace both resolve
  the mutual function references without it. This is an explicit compilation
  adaptation, not removal of a behavioral assertion. Three source forms and
  zero test forms need owned adaptation. Before-images are retained in
  recovery/lang_core_rewrite_inline_do.edn. Permanent integration proves exact
  reconstruction, repeat-plan identity and byte-exact installed regeneration.

- Written inline-do passes fourteen facts. Each added fact rejects a wrong
  expectation (ten passes/four failures), and restored expectations pass. Its
  Python rewrite consumer passes six facts; the broader Dart rewrite check
  passes 45. The full written/reloaded eleven-namespace migration-tooling suite
  passes 289 checks with zero failures, throws or timeouts. Diff checks pass and
  pinned language source/tests remain unchanged. Refreshed namespace inventory
  catalog status for this port; the broader runtime and emitter work remains
  open and no commit/publication was made.

- Saved a pinned namespace-level inventory under Foundation's
  resources/code/migrate/inventory. The broad scan covers 201 namespaces:
  base 25, common 4, core 34, model 85, runtime/basic 40, typed 13. Optional
  models and annex runtimes are inspection inputs, not additional approved
  ports. All entries parsed with reader evaluation/custom readers disabled;
  source/test hashes, namespace dependency declarations, syntactic definition
  counts, paired facts/assertion markers and literal catalog status are retained.
  Written EDN readback exactly matches the scan. Seven source files lack
  path-matched authority tests; 70 nested assertion markers across 17 paired
  test files require review (including the conditional case already adapted).
  Catalog coverage is twelve candidate targets, five planning, and 184 scanned
  namespaces not cataloged. These are inventory counts, not passing-test or
  completed-port claims. Required dependency closure and per-symbol parity
  audits remain open; no source or runtime behavior was changed in this scan.

- Added a pinned oneshot execution contract test using the exact sh-exec and
  raw-eval-oneshot definitions from the immutable authority. Instrumented
  process I/O verifies shell environment forwarding, enforced args/wait/root,
  pipe write-close-wait-output ordering, raw [exit lines] results, stderr-line
  fallback, empty output, custom trimming, and exec-fn override/fallback
  dispatch. This is source-contract evidence, not a native process parity
  claim. A deliberately wrong expectation is rejected; restoration and the
  written/reloaded focused suite pass 39 checks. The full migration-tooling
  suite passes 287 checks with zero failures, throws, or timeouts.

- Native oneshot audit confirms further fidelity gaps: process-options only
  forwards cwd/env/stdin, raw-eval ignores exec-fn and returns a structured map
  for :raw instead of Foundation's [exit lines], and execution always appends
  source to argv rather than honoring :pipe. The facade test at
  test/lang/runtime/basic_test.hal for raw-eval-oneshot asserts literal true;
  it does not execute the runtime and cannot establish process parity. The
  native launcher is unchanged. Requested the semantic decision to restore
  Foundation's raw contract and adapt native consumers; no response yet.
  Installed migration count remains twelve pairs. Diff checks pass and the
  pinned language source/test trees remain untouched.

- Installed and promoted lang.core.rewrite.truthy: twelve installed source/test
  pairs and seventeen catalog targets. The owner-scoped adapter replaces
  collection/form?, Boolean instance checks, owned set membership, and the
  dotted-call nth reads with existing native form?/boolean?/has? and safe
  first/drop operations. No new runtime surface was introduced. Two source
  forms require adaptation; no historical test forms require owned adaptation.
  All six pinned historical assertions remain, with all previous native
  assertions retained in six explicitly authored boundary facts. These also
  cover short and sequence-backed calls, recursion defaults, false/nil values,
  custom callbacks, metadata, and boolish rewrite idempotence.

- Truthy validation: native scaffold preview/write reports no changes; the
  complete candidate and written pair pass twelve facts. Deliberately wrong
  expectations in all six added/retained facts produce six passes/six failures,
  then restored expectations pass. Written truthy plus its Dart rewrite
  consumer pass 57 facts. The native correspondence audit passes three checks
  for six definitions, with no missing, incomplete, or unchecked tests.
  Recovery stores both exact prior native files in
  recovery/lang_core_rewrite_truthy.edn. Permanent integration proves byte-exact
  installed source/test regeneration, repeated-plan equality, and exact
  reconstruction from indexed adaptation evidence. The full written/reloaded
  eleven-namespace migration-tooling suite passes 286 checks with zero failures,
  throws, or timeouts. Diff checks pass; pinned src/tahto and test/tahto remain
  untouched. Runtime process and emitter-generated bootstrap work remains open.

- Fixed reader-function structural replacement in code.migrate.engine after
  the truthy candidate exposed map reconstruction sorting non-comparable list
  keys. Replacement now uses the existing parsed-form-block boundary instead
  of reconstructing raw collection data through std.block's map constructor.
  A permanent regression checks executable map-key/value semantics, quoted
  reader-data preservation, and repeat-rewrite idempotence. Against the prior
  written engine: 111 checks pass and one throws. With the evaluated and then
  written/reloaded fix: all 112 focused checks pass. The full eleven-namespace
  migration-tooling suite passes 284 checks with zero failures, throws, or
  timeouts; git diff --check passes. Truthy remains an uninstalled candidate;
  the installed count remains eleven source/test pairs.

- Installed and promoted lang.core.rewrite.hoist: eleven installed pairs,
  sixteen catalog targets. Replaced the scaffold placeholder with the three
  preserved historical assertions, both existing native operand regressions,
  and quote/skip, map prefix order/metadata, bulk-transform, and sequence-backed
  function coverage. All six native facts pass. Deliberately wrong expectations
  in each of the five added/retained facts produce one pass/five failures;
  restored written files pass. Exact prior native source/test bytes remain in
  recovery/lang_core_rewrite_hoist.edn.

- Native scaffold/incomplete/unchecked audit passes all three checks with one
  owning factory definition and no missing or placeholder tests. Permanent
  integration proves byte-exact installed source/test generation, guarded
  acceptance, identical repeated plans, and exact recovery of the one source
  and four test adaptations. Written hoist and Dart rewrite suites pass 51
  facts. All eleven written/reloaded tooling namespaces pass 283 checks with
  zero failures/throws/timeouts; diff-check is clean and pinned authority
  source/tests remain untouched. Runtime process and emitter-driven bootstrap
  installation remain outstanding; no broader completion claim is made.

- Persisted the fn-parts sequence-input repair in the owned migration adapter:
  materialize its finite form with vec before positional destructuring, retaining
  seq for the reusable tail. Added a catalog-owned regression covering named,
  anonymous, empty-body, cons, and nil inputs. The old generation fails the new
  fact (fourteen passes/one failure); repaired generation and the written pair
  pass fifteen. Scaffold preview/write reports no new obligations, and the
  native scaffold/incomplete/unchecked audit passes all three checks.

- Regenerated both native function files through code.migrate. The updated
  permanent integration verifies their exact installed bytes and recovery;
  statement and Dart rewrite consumers pass, with 75 native facts across the
  three written test files. The hoist candidate now correctly lifts a named
  sequence-backed function and passes six facts including retained native
  operand regressions and new boundary coverage. Hoist still awaits installation
  and catalog promotion; its scaffold placeholder remains pending replacement.
  Full written/reloaded tooling passes 282 checks with zero failures/throws/
  timeouts, clean diff-check, and unchanged pinned authority source/tests.
  Installed namespace count remains ten; this repairs an existing migrated pair.

- Hoist boundary probes pass quote/skip isolation, map-key-before-value prefix
  order with metadata, and bulk flattening suppressed by a body transform.
  Together with the historical fact these pass four native facts. Captured
  exact native hoist source/test before-images in recovery/lang_core_rewrite_hoist.edn.
  Scaffold preview/write reported one changed test entry, not the predicted
  zero: the existing Test/check correspondence did not satisfy the fact
  inventory. Its generated placeholder remains pending replacement; hoist
  source has not been overwritten or promoted.

- Found a prerequisite failure before hoist installation: fn-parts applied
  to (seq '[fn named [x] x]) returns [nil nil ([x] x)], losing the function
  name and arguments even though the predicate accepts sequence forms. A fresh
  native assertion fails on the installed source. A candidate migration change
  materializes the fn-parts input with vec before positional destructuring;
  it passes all fourteen existing native function facts plus the new sequence
  regression (fifteen total). This candidate is not written yet; restored the
  loaded adapter from disk after the probe. Next action is to persist the rule
  and regression, regenerate the existing function pair, and only then finish
  hoist installation. Installed namespace count remains ten.

- Developed and wrote the lang.core.rewrite.hoist adapter and plan-unit dispatch.
  The candidate retains all three historical assertions in its one fact. It
  reuses the statement tail adapter, adds the owned var-tail normalization,
  maps collection/form? and fn-tags membership to existing native predicates,
  and routes the test's grammar and prewalk-replace dependencies to existing
  native owners. Only the two pinned fixture Vars lose private metadata;
  quoted data and unrelated membership forms remain unchanged.

- The hoist adapter regression detects missing adaptation and passes after
  restoration. Written/reloaded candidate passes all historical checks in a
  fresh native process with empty diagnostics. One source and four test
  adaptations recover exact authority bytes and are idempotent; repeated
  generation is identical. Full tooling passes 282 checks across eleven
  namespaces, zero failures/throws/timeouts, with clean diff-check and untouched
  pinned source/tests. Native recovery capture, additional factory/branch tests,
  scaffolding, installation, and permanent catalog integration remain pending.
  Installed count remains ten. Re-requested the outstanding emit-options tuple
  restoration decision; no answer or broader API authorization is assumed.

- Installed and promoted lang.core.rewrite.statement: ten installed pairs,
  fifteen catalog targets. Saved exact existing native source/test bytes,
  checksums, HEAD, and authority revision in recovery/lang_core_rewrite_statement.edn
  before native scaffold preview/write. Replaced the discovered alias placeholder
  and added false/nil else routing, var final-value routing, and do preparation
  order checks with exact metadata and callback traces. The sixteen historical
  assertions remain intact; all fifteen native facts pass. Wrong expectations
  in all four new facts produce eleven passes/four failures before restoration.

- Native correspondence audit reports all twelve definitions, no missing
  scaffold entries, and no incomplete or unchecked tests. Permanent integration
  proves guarded migration acceptance, byte-exact installed source/test output,
  identical repeated generation, and exact recovery of nine source adaptations
  with no owned test adaptation. Written/reloaded migration tooling passes 281
  checks across eleven namespaces, zero failures/throws/timeouts. Written
  statement, hoist, and Dart rewrite files pass 62 native facts. Diff-check is
  clean; immutable authority source/tests are untouched. This is rewrite parity,
  not completion of emitter-generated JS/Lua/Python runtime bootstraps.

- Developed and wrote the lang.core.rewrite.statement adapter and generator
  dispatch. Its immutable authority pair contains eleven facts and sixteen
  assertions. The unadapted native candidate passed five facts, failed two,
  and errored four: rest-bound iterators were consumed by repeated callback
  reads, and nested sequence destructuring was not equivalent to Foundation.
  The scoped adapter converts the pinned rest-binding patterns through seq,
  reads the nested branch condition/body with first/rest, and lowers optional
  else through the existing first/seq operations. It preserves the binding RHS,
  exact expected values, false-else behavior, and quoted forms.

- The statement adapter test fails with the adapter disabled and passes after
  restoration. Written/reloaded candidate passes all sixteen historical
  assertions in eleven native facts with no diagnostics. Repeated generation
  is identical; nine source adaptations recover exact pinned bytes and are
  idempotent, while the historical test needs no owned adaptation. The full
  eleven-namespace tooling suite passes 280 checks with zero failures, throws,
  or timeouts. Native installation, before-images, alias/branch coverage,
  scaffolding, and catalog promotion remain pending; this is a REPL candidate,
  not the tenth installed pair. Authority source/tests remain unchanged.

- Installed and promoted lang.core.rewrite.fn: nine installed source/test pairs
  and fourteen catalog targets. Exact previous native bytes, checksums, HEAD,
  and authority revision are retained in recovery/lang_core_rewrite_fn.edn.
  Native scaffold preview/write reports no missing functions. Added four
  semantic checks for sequence forms, reusable/nil tails, rewrite/prepare order
  and metadata with idempotent normalization, and symbol callback precedence.
  The generated pair retains all 28 historical assertions and passes fourteen
  native facts. Against the previous native source it fails three facts,
  exposing both narrowed list-only predicates and empty-tail differences.
  Deliberately wrong expectations in all four additions fail four facts;
  restored written source/tests pass fourteen.

- Native correspondence audit reports all ten functions, no missing scaffold
  obligations, no incomplete or unchecked tests. Permanent pinned integration
  verifies guarded acceptance, exact installed source/test bytes, repeatable
  generation, and recovery of all five source/two test structural adaptations.
  Written/reloaded full migration tooling passes 279 checks with zero failures,
  throws, or timeouts. Native statement/hoist/Dart rewrite consumer tests pass
  58 facts; Lua/Python XTalk helper checks additionally pass 28 facts. These are
  local rewrite/helper checks, not proof of emitted runtime bootstraps or live
  process parity. Diff-check is clean and pinned authority source/tests remain
  unchanged. The five runtime/core/util planning targets remain uninstalled.

- Developed and wrote the scoped lang.core.rewrite.fn adapter and plan-unit
  dispatch. The pinned pair has ten facts and 28 historical assertions. Its
  first structural candidate failed six native facts: rest destructuring in
  fn-parts produced a consuming iterator, losing body forms when read repeatedly.
  The existing native pair still passed its narrower ten-fact suite. Converting
  only this owned tail through the existing prelude seq API restores the pinned
  nil-empty and reusable-body contract without weakening any expected values.
  Other explicit adaptations use form? for collection/form? and the owned block
  predicate fixture, preserve passthrough-rewrite as an ordinary test helper,
  and give repeated ignored bindings distinct names in lift-named-lambda.

- The written/reloaded fn adapter passes 64 focused checks; all eleven tooling
  namespaces pass 278 checks with zero failures/throws/timeouts. Regeneration
  of the candidate repeats identically, five source and two test adaptations
  recover exact authority bytes and are idempotent, and fresh native execution
  passes all ten historical facts. This target remains a REPL candidate, not
  an installed or catalogued pair: native before-images, scaffold/additional
  boundary contracts, permanent integration, and catalog promotion remain.
  Installed count remains eight. No pinned source/test files were modified.

- Installed and promoted lang.core.rewrite.conditional: eight installed pairs,
  thirteen catalog targets. The copied dependency replacement mechanism and
  owned std.block adaptation replace collection/form? with the native prelude
  predicate and lift only the pinned closed expected vector outside letfn.
  The original letfn computation, expected data, and fact metadata are retained;
  unknown expectations/owners are not lifted and remain subject to the nested
  assertion guard. Both historical assertions are now counted and checked.

- Preserved exact prior native source/test bytes, checksums, native HEAD, and
  authority revision in recovery/lang_core_rewrite_conditional.edn before
  running native scaffold preview/write. Replaced its alias placeholder with
  semantic metadata/scalar assertions and added all conditional branch routing
  and metadata checks. The complete generated pair passes four native checks;
  deliberately wrong historical expectation fails one, and deliberately wrong
  expectations in both additions fail two. The restored written file passes
  four. Explicit native scaffold/incomplete/unchecked audit passes three checks:
  all three definitions correspond, with no missing or placeholder tests.

- Written/reloaded conditional integration proves byte-exact source/test
  regeneration, identical repeated plans, one recoverable structural adaptation
  in each authority file, and guarded migration acceptance. Full eleven-namespace
  tooling suite passes 277 checks with zero failures/throws/timeouts. Diff-check
  is clean; pinned src/tahto and test/tahto are unchanged. JS/Lua/Python process
  targets remain planning-only; no claim of completed runtime pipeline.

- Corrected the conditional probe interpretation below: changing the nested
  expected value to a deliberately wrong value still produced two native
  passes. Native code.test.compile/fact only checks top-level loose assertion
  pairs; the nested letfn body executed without asserting equality. Two passing
  registered facts were not evidence of two executed historical assertions.
  Conditional promotion therefore requires a structural adaptation that moves
  this closed expected literal to a top-level checked pair while retaining the
  letfn-bound actual expression and exact expected data.

- Added quote-aware nested-fact-markers and a fail-closed diagnostic to
  code.migrate.test/emit-test-facts for executable nested loose markers. Quoted
  data and top-level checks are excluded. The regression fails on the previous
  implementation (20 passed/one failed) and passes with the evaluated guard
  (21 passed). Written/reloaded full migration tooling passes 275 checks across
  eleven namespaces, zero failed/throw/timeout; diff-check is clean. No native
  source/test pair or pinned authority was changed by this guard repair.

- Revalidated the written migration integration namespace: 33 passed, zero
  failed/throw/timeout, including walker native execution, exact installed
  source/test bytes, recovery, and guarded acceptance. Corrected the stale
  walker catalog status from planning to candidate; a fresh catalog read now
  reports seven candidates and five planning targets.

- Probed pinned conditional rewrite source/test through code.migrate with a
  candidate target and the existing dependency-symbol replacement mechanism
  scoped to collection/form? -> form?. Both historical native checks pass in
  a fresh process with empty source/test diagnostics. The pair is not installed
  or added to the catalog yet. Found an assertion inventory defect: native
  execution reports two checks, but the planner reports one because the second
  historical assertion is nested inside letfn. Preserve that assertion and fix
  inventory accounting before promotion; callable alias correspondence and
  branch/metadata additions also remain required.

- Hardened verifier process ownership after the observed orphan. `run-process`
  now delegates capture to `capture-process`, which terminates a still-live
  owned child, waits for termination, cancels both output reader futures and
  closes streams on normal completion, input failure or interruption. A real
  sleep-process regression fails against the old capture behavior and passes
  with cleanup; its emergency test teardown runs only after observing the
  production result. Input-conversion failure preserves the original exception
  data while terminating the child and settling both readers. Exact cat/false
  results preserve command, cwd, Unicode stdout, stderr, exit and passed fields.

- Added an end-to-end interrupted native-verifier regression: it observes the
  owned child, interrupts capture after stdin conversion begins, waits for the
  worker's finally boundary, and verifies that the child is dead and the unique
  stdin link has been removed. Written/reloaded focused verification passes six
  checks; all eleven tooling namespaces pass 273 checks with zero failures,
  throws or timeouts. No verification children or stdin links remain after the
  suite, diff-check is clean, and the pinned language authority is unchanged.
  This repairs the direct-child interruption leak; it is not a claim that
  every future runtime descendant/lifecycle scenario is already covered.
  Installed migration count remains seven; conditional rewrite is next.

- Installed and promoted `lang.core.rewrite.walk`: seven installed pairs,
  twelve catalog targets. Saved exact native source/test before-images, their
  checksums, native HEAD and authority revision in
  `resources/code/migrate/recovery/lang_core_rewrite_walk.edn`, referenced by
  the target. Native scaffolding identified the callable with-form-meta alias;
  its generated TODO was replaced by hand-written metadata assertions. Added
  short/nil map-entry, binding-vector boundary and list/scalar dispatch tests.
  All eight historical assertions remain, with four native additions. The old
  native source fails the short-entry test with nth index out of bounds; the
  generated source passes all twelve. Deliberately wrong expectations for each
  of the four additions produce eight passes/four failures, then restoration
  passes. Written walker and existing conditional consumer pass 14 native checks.

- Fixed the engine metadata traversal before installation. code.query's
  metadata wrapper skipped alternating metadata maps, leaving some :refer
  owners in tahto even though native facts executed. Dependency-symbol rewriting
  now traverses std.block tokens directly, retaining comments, metadata wrappers
  and quoted host-operator protections. The old engine fails the consecutive
  metadata regression; written engine passes 111 checks. All walker :refer
  owners are native. Other installed pair outputs are unchanged; the known
  emit-template require indentation difference remains the only mismatch.

- Native explicit-unit scaffold audit reports all nine walker definitions,
  no new scaffold entries, and empty incomplete/unchecked findings. Both written
  walker files match generation byte-for-byte; repeated pinned generation is
  identical and recovery checksums validate. Removed the reconciliation guard
  and updated the permanent native integration check to require successful
  promotion and exact installed bytes. All eleven tooling namespaces pass 269
  checks, zero failures/throws/timeouts. Authority src/tahto and test/tahto remain
  unchanged; no commits or publication were performed.

- Audit lifecycle issue remains to fix: a broad native CLI audit exceeded its
  nREPL observation timeout, and run-process did not terminate its child when
  evaluation was interrupted. Confirmed the child remained live, explicitly
  terminated that task-owned orphan, and replaced only the wedged nREPL session
  (not the server or loaded Vars). Reran the audit directly on explicit units
  with a terminal process handle; it passed three checks. No native audit
  processes remained at final inspection. Harden verifier interruption cleanup
  before further long-running REPL-owned process checks; do not mistake an
  interrupted observer for child completion.

- Ran the actual generated preprocessing and emission-helper pairs through the
  repaired native verifier: preprocessing passes 13 registered checks and
  helper passes 40, with zero failures/errors/timeouts. These counts include
  native additions; their historical planned assertion counts remain 4 and 25.

- Added guarded catalog target `lang.core.rewrite.walk` (twelve targets total,
  still six installed pairs), sourced from the immutable pin. Its nine
  definitions include eight functions plus the with-form-meta alias. Implemented
  and wired `rewrite-walk-form`: lower the owned map transducer to two-argument
  into/map, use the existing prelude form? predicate, adapt only the historical
  JVM map-type expectation to the native hash-map type, and disambiguate vector
  and set fact titles without changing their assertions. The binding-vector
  fixture initially failed natively because native seq? excludes persistent
  lists; its exact owned callback now uses form?, whose existing implementation
  accepts cons/list/seq values. Quoted seq? forms remain untouched.

- The complete generated walker pair passes all eight historical assertions
  natively. Permanent tests prove exact reconstruction from indexed before-images
  and idempotence for both source and tests (two adapted source blocks, four
  test blocks), deterministic full plans and promotion rejection on the sole
  `:native-source-test-reconciliation` boundary. Missing-adapter controls are
  rejected; old catalog inventory expectations fail before their twelve-target
  updates. Written/reloaded full tooling passes 268 checks, zero failures,
  throws or timeouts. One evaluator command timed out; the same REPL completed
  the subsequent focused run without a restart. No native walker file or pinned
  authority was edited. Before installation, reconcile/preserve the existing
  native pair, scaffold correspondence (including the callable with-form-meta
  alias), add meaningful remaining contracts and clear the promotion guard.
  Conditional rewriting remains the next dependent pair, not a completed port.

- Repaired native migration verification using an explicit `:native-test`
  runner in `code.migrate.verify`. The runner feeds the complete candidate
  through a uniquely named test-path symlink to stdin and removes that exact
  link in finally; it does not write native source or modify the unrelated
  project inventory. The existing CLI pathway remains available. Live tests
  prove success, failed assertions, unreadable input, and cleanup after a
  missing executable. Integration fixtures now resolve the current workspace
  layout and native binary (with HARA_NATIVE_BIN override), rather than the
  obsolete core/hara path. This POSIX stdin-link pathway was tested locally;
  it does not establish Windows runner support.

- Native pair verification now lets the native test runner execute the actual
  generated fact source and own failure status, rather than appending the old
  empty-Test/run trailer. Its fixture has an explicit source/test catalog owner
  and fact format. A complete generated increment pair passes; changing only
  its generated implementation to decrement fails the original assertion in
  another fresh native process. This is integration evidence for generated
  pair execution, not for the unfinished language/process pipeline.

- Investigating the previous fixture exposed an engine ownership bug:
  `target-for-unit` matched nil source paths to absent catalog test paths,
  selecting std.block.protocol for an unrelated pathless fixture. Matching now
  requires a nonblank string path. The old engine fails the new regression;
  written/reloaded engine passes 110 checks covering missing, empty, unknown,
  source and test paths. All eleven tooling namespaces now pass 266 checks,
  zero failures, throws or timeouts, including the two previously broken native
  integration checks. Foundation src/tahto and test/tahto remain unchanged,
  diff-check is clean, and no verification symlinks remain after execution.

- Added the pinned oneshot setup contract regression in `code.migrate-test`.
  It evaluates the exact source function from the authority revision with
  local program/options/exec callbacks, checking nested sibling preservation,
  vector replacement, nil overrides at shell and environment levels, scalar
  replacement, four/five-argument context forwarding and explicit executable
  precedence. The shallow-merge alternative produces a different result.
  Deliberately incorrect expectations are rejected (the Foundation runner
  reports one throw); restoration and written/reloaded focused execution pass
  all 32 checks. No language authority or native source was edited.

- Native runtime option audit found two distinct merge defects: oneshot setup
  uses shallow merge, and `type-common/runtime-deep-merge` loses a default map
  when the override is nil. A fresh native regression against that helper
  fails with `{:shell nil}` instead of the retained shell/environment map.
  The existing prelude `merge-nested` passes the five exact authority cases in
  a fresh native process and is the correct existing owner for this boundary.
  Neither launcher has yet been changed. Nested `:shell :env` forwarding still
  needs repair; no actual process or emitted bootstrap parity is claimed.

- Expanded validation from the prior seven namespaces to all eleven migration
  test namespaces: 262 passed, two throws, zero failed/timeouts. After explicitly
  reloading facts, per-namespace execution localizes the throws to native
  integration checks in `code.migrate.probe-test` and `code.migrate.verify-test`.
  Both still construct the obsolete path
  `workspace/workspace/technology/hara/core/hara` and fail before process launch.
  The actual installed `hara` CLI runs a standalone stdin definition correctly,
  but using the current Hara project fails its source inventory on existing
  `src-lang/postgres/sample/scratch_v3.hal` (no ns/ns+ declaration). That unrelated
  source and the legacy integration tests remain unchanged. The native debug
  test runner used for focused probes succeeds; its test-path constraint rejects
  direct `/dev/stdin`, so a CLI path substitution alone is not a verified repair.
  The full tooling suite must not be reported green until these checks execute.

- Corrected the process inventory to include private functions, macros and
  defonce forms. Lua owns 26 definitions, not the previously reported 25:
  `lua-local-rocks-env` was excluded by the inventory's def/defn-only filter.
  Complete JS/Lua/Python source inventories therefore total 63 definitions;
  all 26 historical assertions across those pairs remain. Implemented and
  wired `rewrite-process-lua-form`, preserving that helper as ordinary `defn`
  in the existing internal owner (no public marker). The internal namespace
  config initially did not appear because its existing namespace-override rule
  was not selected; focused validation caught this. Selected
  `:clojure/source-namespace-overrides` and verified the written plan's role.

- LuaRocks environment adaptation uses OS/getenv and awaited File/exists?,
  retaining omitted branches, exact user-local paths, inherited empty values
  and original semicolon defaults. Five branch cases pass in the Clojure
  adapter contract and a fresh native boundary-injected probe using real
  Promise values. The actual generated helper evaluates natively; a separate
  fresh native check proves File/exists? yields awaited true/false values.
  Omitting the await deliberately fails the branch test; restoring it passes
  both native checks. No HOME mutation or Lua process launch was performed.
  The exact basic-client duplicate fact title is disambiguated; all 14 Lua
  historical assertions remain. Full Lua plans now report only the three
  declared integration blockers, no host interop or duplicate title findings.
  Both adapted source/test recover exactly from indexed before-images and are
  idempotent. Written/reloaded full tooling passes 247 checks, zero failures,
  throws or timeouts. No native process pair is installed; bootstrap emission,
  registration/lifecycle and program-environment integration remain unfinished.

- Implemented and wired `rewrite-process-js-form` for the JS environment
  boundary: exact `System/getenv`, `user.dir` property and platform separator
  uses lower to existing OS intrinsics; string calls use the default `str`
  prelude and the redundant require is removed. Other system properties and
  quoted client programs remain unchanged. Native OS source confirms platform
  keywords and missing-env nil behavior; a fresh probe confirms the local macOS
  contract. Seven exact node-path cases cover cwd fallback, PWD precedence,
  empty PWD, blank/exact-duplicate existing paths, retained multi-entry strings
  and Windows separators (the Windows branch is injected-contract evidence,
  not execution on Windows). Preserve pinned whole-input distinct behavior;
  do not split/deduplicate existing NODE_PATH entries as an unrequested change.
  A fresh native generated-function probe passes with scoped NODE_PATH.
  Adapted the exact node-path historical fixture's quoted-regex separator to
  native literal splitting; it uses only colon/semicolon. Unrelated regex
  expressions are not rewritten. The complete historical fact, including all
  five original boolean checks, passes in fresh native runs before and after
  tooling writes. Full JS source/test plans now have no host-interop findings,
  retain all three historical assertions and reject installation on declared
  bootstrap/lifecycle/environment integration boundaries. Negative controls
  detect old adapter/dispatch behavior; initial source/test before-images
  reconstruct exactly and adaptation is idempotent. Written/reloaded tooling
  passes 245 checks, zero failures/throws/timeouts. No native process files or
  Foundation language authority were edited; emitted bootstrap integration is
  not completed by this node-path boundary validation.

- Tightened lossless layout comparison without forcing unreadable fallback for
  harmless map entry ordering. The comparator recursively retains collection
  shape, metadata and scalar printed spelling, while treating map/set ordering
  as unordered. Tests accept a reordered map but reject vector-to-list changes,
  lost nested metadata and negative-zero changes; the previous comparator
  fails the new map-layout assertion. Fixed top-level namespace alignment to
  identify `ns`/`ns+` by declaration kind and occurrence, not the namespace name
  that migration intentionally changes. Old alignment fails both identity and
  multiline preservation regressions. Written/reloaded tooling passes 242
  checks, zero failures/throws/timeouts. These fixes restore exact installed
  helper source regeneration without editing native files: five of six installed
  source files and all six test files now match generated bytes. The remaining
  template source difference is indentation of two namespace require entries
  (generator four spaces, installed twelve), not source behavior. Do not claim
  all installed artifacts are byte-exact until that last difference is resolved.

- Full process planning exposed and fixed two engine fidelity bugs. Dependency
  rewriting had applied the copied `==` -> `=` host operation rule inside
  quoted Lua/Python client programs; it now leaves quoted symbol-replacement
  rules and quoted anonymous-reader functions untouched, while namespace
  migration remains explicit and executable host calls still lower. Source
  alignment/layout had dropped fn-valued definition arglists metadata whenever
  a factory body changed. Source records and rewritten forms now use semantic
  reads, metadata participates in unchanged-form comparison, and layout is
  accepted only when its reader value and metadata survive; otherwise it falls
  back to lossless printing. Tests include explicit nested line metadata,
  string escapes and metadata-only edits. Four old-engine negative controls
  fail; written engine checks pass. The prior formatting fixture now reads its
  explicit source string instead of unintentionally injecting the Clojure
  compiler's line/column metadata through a quoted test constant.

- Added complete JS/Lua/Python process pairs as three guarded planning targets
  (eleven total catalog targets, still only six installed pairs). Inventories
  retain 19/25/18 definitions and 3/14/9 historical assertions respectively.
  Quoted `+client-basic+` values, bootstrap factory arglists, definition order
  and emitter-call counts match the pin; repeated full planning is identical.
  All six process units reject promotion on bootstrap emission, runtime
  registration/lifecycle and program-environment boundaries. JS/Lua additionally
  expose JVM environment/filesystem operations; Lua tests expose a duplicate
  "wraps with the eval wrapper" title still needing adaptation. No process
  source/test artifact was installed and no process was launched.

- Audited installed-pair regeneration after these engine changes. Five pairs'
  output is unchanged by the engine fixes; helper source has a formatting-only
  change whose parsed form equals the previous output. All six installed tests
  match generation; existing template/helper source formatting differences
  remain and were not overwritten. Guarded helper generation initially exposed
  a pre-existing false host diagnostic for `Num/parse-double`; the current
  native registry explicitly declares Num and that method. Added Num to the
  existing native-class allowlist (no new native API); the old allowlist fails
  its regression. All six installed targets now pass guarded generation with
  identical second generations. Written/reloaded full tooling passes 239
  checks, zero failures/throws/timeouts. Foundation language authority and native
  files remain unchanged; whole-pipeline parity is still incomplete.

- Composed the core adapter with the established provenance adaptation only
  for `emit-direct` and the exact historical direct-emission failure fact.
  This preserves asserted form/data while lowering provenance keys and JVM
  catch syntax; resource-registration forms and quoted programs outside that
  owned fixture are unchanged. Selected the existing `:clojure/ex-info-native`
  test rule after the full generated fixture exposed its remaining constructor.
  Old-adapter and old-catalog negative controls detect the missing adaptations.
  The test recovery inventory now correctly records two changed blocks (title
  plus failure fixture), following an observed failure of its old one-block
  expectation. Full pinned source/test adaptation reconstructs both originals
  exactly and is idempotent (one source block, two test blocks). All 45 core
  historical assertions remain. Written/reloaded migration tooling passes 233
  checks, zero failed/throw/timeout. No native core files were installed and
  no end-to-end native error behavior is claimed; all three declared core
  runtime-contract blockers remain, including the unanswered options API choice.

- Traced the core library boundary further. Pinned `get-book` delegates through
  snapshot `get-book` to `book-from`, which returns nil for an absent non-nil
  language. Native `library-get-book` attempts selection and throws "Book is
  not registered". A fresh native local-Library probe confirms that behavior
  without touching default/global state (one passing assertion; the initial
  harness lacked the fact macro, corrected to native `Test/check`). Added a
  permanent test executing the exact pinned `emit-as` function with local
  instrumented library/emitter callbacks: uppercase keyword fallback, missing
  key preservation, metadata precedence, ordered emission, two-newline joining
  and lookup even for an empty forms vector are exact assertions. Incorrect
  fallback expectation fails; restored and written focused tooling passes 27
  checks. This is pinned boundary authority, not native emission parity.
  A mechanical `get-book` -> `library-get-book` rename is therefore invalid.
  Pinned core reset also delegates to component/resource teardown, while native
  library reset clears the existing state-owning object; these remain an
  explicit lifecycle reconciliation, not an approved interchangeable mapping.
  Asked whether to restore the original tuple-returning core `emit-options`
  while adapting current native compiler/lifecycle consumers to construct their
  map context separately. No answer or implicit approval is assumed.

- Implemented and wired `rewrite-core-impl-form`: only the pinned `%.str`
  fact with the colliding output-string title gets a distinct macro-specific
  title. The executable body, both expected strings and metadata are unchanged;
  quoted forms, the `emit-str` fact and unrelated owners remain unchanged.
  Complete pinned planning retains all 45 assertions and no longer reports
  duplicate descriptions. Identity-adapter and old-planner controls fail.
  Structural recovery restores the original test blob byte-for-byte; the
  adaptation changes one block and is idempotent. Runtime contract blockers
  remain in both plans; this does not claim native core execution parity.
  Added `:foundation/lang-book-module` through the existing dependency engine:
  `tahto.common.book-module` routes to verified native `lang.base.book-module`,
  which owns `resolve-module-view`. The original alias and call remain intact.
  Old-catalog routing control fails and repeated source planning is identical.
  Written/reloaded migration tooling passes 230 checks, zero failures, throws
  or timeouts. No native files or Foundation language authority were edited.
  Library tracing identifies existing native state owners rather than a need
  for a new resource API: `lang.core.library` already owns default/dynamic and
  namespace-selected libraries, while `std.lib.context.resource` owns the
  resource registry and setup/teardown hooks. Their contracts still need to be
  reconciled with pinned core resource registration, annex selection and reset;
  do not create a second independent default library by mechanical renaming.

- Completed the previously running native core baseline (session 44160):
  `test/lang/core/impl_test.hal` passes all six registered tests. This covers
  the existing native API, not the missing Foundation emission surface.
  Added a permanent pinned bootstrap contract fact for JS, Lua and Python:
  structurally reads only the exact authority blobs, evaluates factory
  initializers with local instrumented emitter callbacks, and checks exact
  dependency/client/call order, option forwarding, quoted client preservation,
  arglist metadata, default/custom/nil host behavior, port arguments, Lua's
  cjson prefix and two-newline assembly. Dependencies/client emission occurs
  once at factory initialization; each invocation emits only its client call.
  All state is local; no runtime registration or network process is launched.
  The deliberate incorrect expectation fails; restored and written tests pass.
  This is factory wiring authority, not native emitter or process parity.

- Added `:migration/lang-core-impl` as the eighth catalog target, explicitly
  planning-only. Full source planning retains all 31 pinned definitions in
  order, including `emit-as`, `emit-entry-deps-collect` and `emit-entry-deps`;
  full test planning retains all 45 historical assertions. Repeat planning is
  identical. The selected rule blocks both units on the options-context,
  library-resource-lifecycle and caller-namespace contracts. Historical tests
  additionally report a duplicate description, "converts to an output string",
  which must be adapted before native registration. The old catalog fails the
  new inventory regression; guarded migration rejects both units. Updated the
  two exact catalog/workflow inventory expectations after observing their
  failures. Written/reloaded tooling passes 227 checks, with zero failures,
  throws or timeouts. No native core/process files were overwritten; core
  implementation and actual emitter-generated bootstraps remain unfinished.

- Traced the required bootstrap emission owners at the pin: `emit-as`,
  `emit-entry-deps-collect` and `emit-entry-deps` are defined in
  `tahto.core.impl`, not `impl-deps`; all three are absent from native
  `lang.core.impl`. A prerequisite contract conflict is now explicit:
  Foundation `emit-options` returns `[stage grammar book namespace mopts]`
  after library/snapshot/module preparation, whereas native `emit-options`
  returns a compiler-context map. Existing native `impl-lifecycle` and its
  tests consume the map. Reconciliation must preserve those consumers while
  restoring the original preparation/library contracts, not alias the missing
  functions onto the differently shaped API.
  Pinned JS/Python basic factories assemble emitted `return-eval` dependencies,
  emitted `+client-basic+`, then an emitted `(client-basic host port {})` call;
  Lua adds the authoritative cjson require prefix, passes
  `+lua-basic-script-emit+` to dependency emission, and invokes the client with
  only host/port. Default host is 127.0.0.1 and sections use two newlines.
  The native `test/lang/core/impl_test.hal` baseline is still running in exec
  session 44160 (PID 2136, verified alive at 87 seconds and 96.2% CPU), without
  output yet; this is not a passing result or a confirmed hang. Continue
  polling that handle before starting another core implementation test.

- Reconciled the two exact utility exception fixtures without partial-map
  checkers. Native exception causes compare by object identity, so the actual
  comparison converts only `:ex/cause` to its complete `ex-data`; expected
  maps explicitly retain every outer and nested native field, original user
  data, messages, wrapped marker and cause class. Cause accessor failure still
  fails the test. `rewrite-util-exception-fact` is limited to the two pinned
  owner references and exact original expectation shape; unrelated facts and
  quoted programs remain untouched. The identity-rule negative control fails,
  and the earlier generated native fixtures fail on both exact comparisons.
  Written/reloaded tooling passes 225 checks. All five generated utility
  provenance/error facts pass in fresh native runs, including the nested
  provenance-stack assertion. The original test source reconstructs exactly
  from structural before-images; adaptation is idempotent and repeated full
  source/test generation is identical, retaining 22 assertions. This is still
  a planning-only utility pair: runtime context/pointer blockers remain and no
  native source/test installation or broader pipeline completion is claimed.

- Composed utility adaptation with the existing provenance adapter, preserving
  provenance field expectations while lowering Throwable hints/catches and
  `.getMessage`. Fresh generated historical tests then exposed unselected
  `ex-info` conversion; activated the existing `:clojure/ex-info-native` test
  rule used by the provenance target. Both old-adapter and old-catalog negative
  controls fail; written full migration tooling passes 224 checks. All 22
  utility historical assertions remain in the full plan. An isolated native
  run of the five generated provenance/error facts now passes three and fails
  two: provenance normalization, threading and merged nested provenance pass;
  error/throw full-data comparisons differ because native exception data
  includes reserved `:ex/*` fields and uses `:ex.class/internal` rather than
  the JVM class-name string. The actual messages are both exactly "wrap: inner"
  and asserted user data survives. Do not weaken those exact maps into truthy
  or partial checks; explicitly reconcile the exception representation in
  migration while retaining original assertions/before-images. No native
  utility files were written, and the two runtime semantic blockers remain.

- Traced utility runtime owners to existing native `std.lib.context.space`
  and `.registry`; the space path-matched suite passes 25 tests in a fresh
  process. Generated explicit-namespace `lang-rt-list`/`lang-rt` probes from
  pinned definitions pass exact filtering, lookup and two-namespace isolation
  checks, using dynamically isolated space state restored on binding exit.
  Activated the already implemented `:foundation/map-juxt-native` rule for the
  utility target rather than adding a replacement library or new native API.
  Its old-catalog negative control fails; the written full tooling suite passes
  222 checks. This validates explicit namespace arities only: no implicit
  namespace behavior was selected or runtime lookup contract changed. The
  copied `:foundation/context-semantic-recipe` remains an unsupported general
  handler and points at legacy recipe paths, not proof of faithful current
  ownership. Native Pointer is explicitly documented as an immutable descriptor
  whose resolution belongs to the evaluator context; retaining `:context/fn`
  in descriptor data alone cannot establish the original callback behavior.
  Both utility semantic blockers remain declared and no native pair is installed.

- Wired `rewrite-util-form` into public planning and added
  `:migration/lang-base-util` as the seventh catalog target, explicitly
  `:planning`, not installed. The complete pinned pair preserves all seventeen
  definition names in original order and all twenty-two historical assertions;
  repeated generation is identical. Its declared blockers are
  `:runtime-context-selection` and `:pointer-context-callback`; the historical
  test plan additionally exposes `.getMessage` host interop awaiting the
  existing provenance/error adaptation. Guarded migration rejects the pair.
  The old planner fails the new inventory/dispatch regression. Two fixed
  catalog/workflow inventory expectations were updated from six targets to
  seven after their original failures were observed; assertions for copied
  rule counts and the actual guarded refusal remain intact. Written/reloaded
  full tooling passes 221 checks, zero failures/throws/timeouts. No native
  utility source/test files were overwritten or runtime contracts changed.

- Audited pinned `tahto.base.util` and its full historical tests. The pinned
  source owns seventeen functions; native `lang.base.util` owns thirteen and
  has no definitions for `lang-rt-list`, `lang-rt`, `lang-rt-default` or
  `lang-pointer` anywhere under native src. Its thirteen path-matched tests
  pass but do not prove this missing runtime surface. Qualified symbol and
  keyword conversion also loses namespaces because native code uses `name`
  instead of Foundation `strn`; three exact fresh-process regressions fail on
  the installed implementation. Added and REPL-tested `rewrite-util-form`,
  reusing the existing common string/error adaptation and preserving runtime
  definitions unchanged. It is not yet selected by a catalog target or wired
  into public plan dispatch. The old/identity rule fails new tests; written
  focused tests pass 54 and full migration tooling passes 220. Two definitions
  generated directly from pinned authority pass fresh native forward/inverse
  spelling probes (including qualified symbols/keywords, nil, boolean, integer
  and strings). This is boundary validation only: no native utility source/test
  pair is installed, and the runtime-context/pointer owners must be ported,
  not dropped from a future utility catalog target.

- Closed a migration guard gap: `plan-unit` now appends deterministic
  `:migration/unresolved-rule-boundary` diagnostics for unresolved boundaries
  declared by selected rules. Unselected rules do not block unrelated targets;
  resolved rule declarations clear their blockers, existing host diagnostics
  remain intact, and duplicate diagnostics are deduplicated. `analyze-unit`
  now uses this same full planning path, retaining diagnostics/checksums/rule
  evidence without exposing generated source, instead of bypassing adaptations
  through the raw engine. Pre-change planner and analyzer negative controls
  detect both gaps. Written/reloaded full tooling suites pass 218 checks with
  zero failures/throws/timeouts. Actual pinned preprocessing source and test
  now both report the caller-namespace boundary consistently in plan and
  analysis; the public guarded migration rejects both units. No pending
  semantic decision is silently promoted on the strength of an empty static
  host-symbol scan.

- Isolated historical in-fact definitions: evaluating the three literal `def`
  forms through `eval-in-ns` creates/replaces actual namespace Vars, unlike
  replacing them with lexical bindings. An unwritten std.block candidate
  preserves their position inside each fact and makes all three historical
  facts / twelve assertions execute: nine pass, three fail at caller-namespace
  resolution. Both original `any` alternatives remain intact. Do not install
  this experiment as a general fact rule: initializer scope and per-fact state
  restoration still need a reviewed implementation.
  Added a written, narrowly matched nil-safe adaptation for
  `(f/var-sym (resolve ...))`: resolve once, call native var-sym only for a
  resolved Var, otherwise preserve nil for Foundation's explicit error branch.
  The old rule fails the new regression; written/reloaded focused tests pass
  52 and full migration tooling passes 216. A complete generated-source native
  probe returns exactly `["Var not found" missing-fixture-var]` for the missing
  Var exception. This does not fix caller context. Asked whether to propagate
  the existing `*macro-opts* :namespace` through emitter/test callers (explicit
  context for direct calls), or investigate preserving implicit caller behavior
  in the runtime. No answer or runtime-change authorization is assumed.

- Generated the complete pinned preprocessing input source/test pair in the
  language catalog with a REPL-only planning target. Reused copied rules for
  volatile-to-atom and vreset-to-reset conversion. Extended the owning adapter
  to route `tahto.base.util`, use existing prelude functions, remove preloaded
  dependency requires, and lower the exact host error calls. Added narrowly
  scoped executable equality adaptations for qualified Foundation reader
  markers (`deref`, `unquote`, splice marker); quoted programs remain unchanged.
  Old-rule negative controls detect both additions. Written/reloaded full
  migration-tooling suite passes 215 checks, zero failures/throws/timeouts.
  A fresh native process evaluates the complete generated source and passes
  four exact assertions: pointer-to-symbol conversion, ordinary template,
  Cons template and historical enabled-splice order. The complete generated
  historical test file does not yet register: native evaluation reports
  `unbound symbol: def` at the first fact containing `(def hello 1)`. This is
  a test-setup migration boundary, not a passing or skipped historical case.
  Preserve these setup definitions and their Var-resolution semantics when
  adapting fact setup; do not replace them with lexical lets or qualify only
  the expected/input fixtures. Caller-context integration remains unresolved.
  Plans currently report no diagnostics despite these runtime gaps, so empty
  diagnostics are not promotion evidence. Source/test plans remain unwritten
  native candidates; existing native source/test bytes were not overwritten.

- Added `code.migrate.lang/rewrite-preprocess-input-form` and connected it to
  `plan-unit` through the declared `:foundation/lang-preprocess-input` rule.
  It maps executable `collection/form?` and `ptr/pointer?` to the existing
  native predicates, retaining the pinned pointer-to-symbol branch instead of
  copying the native omission. Quoted programs, strings and unrelated owners
  are unchanged; structural before-images reconstruct the original source
  exactly and a second rewrite is identical. Identity-rule and pre-change
  planner negative controls fail the new tests. Written/reloaded migration
  suites pass 212 checks, zero failures/throws/timeouts. This is generator
  wiring, not an installed preprocessing pair; the rule explicitly records
  unresolved caller-namespace evaluation and no target is promoted yet.
  Tracing shows native emit metadata already carries a namespace, while
  `prep-form` calls `to-input` without propagating it. Pinned `create-code-raw`
  also performs `to-input` followed by `eval-template-forms`; the native
  `impl-entry` currently has a different materialize/emit-only surface. Restore
  these owning integration contracts rather than qualifying test fixtures.
  Foundation nREPL 58892 was verified running using the authorized localhost
  check after the sandbox-only status probe incorrectly appeared stopped;
  no server restart was performed.

- Preprocessing dependency audit selected pinned `tahto.base.preprocess-input`
  (three definitions, three historical facts, twelve assertions). The existing
  native path-matched file passes its three registered tests in a fresh process,
  but that is not historical parity. A fresh fourteen-case diagnostic covering
  the twelve historical cases plus pointer and Cons contracts reports nine
  passing, four failing and one error. The two historical `any` static-invoke
  alternatives were probed at the nil alternative accepted by the authority;
  permanent migration must preserve both alternatives rather than narrow them.
  Three historical failures concern caller-namespace resolution: unqualified
  `hello` is resolvable in the caller, but `to-input-form` reports "Var not found"
  and persisted template/value evaluation leaves the unqualified names intact.
  A second fresh probe confirms fully qualified names work. Do not conceal this
  boundary by qualifying only historical test inputs; preserve the caller's
  resolution contract in the implementation or report it explicitly unresolved.
  The other two failures are confirmed source omissions: `to-input` leaves a
  native pointer unchanged instead of calling `util/sym-full`, and `list?`
  rejects Cons forms that native `form?` correctly recognizes. Enabled splice
  order, disabled-splice failure, ordinary template evaluation, language forms,
  deferred eval and unresolved-template preservation pass their probes.
  Next generator work must restore the pointer branch, preserve the broader
  form predicate, and resolve the caller-context boundary without expanding
  native APIs. No preprocessing source/test files were overwritten by the audit.
  An isolated two-namespace native probe further confirms this is not merely
  a prelude-wrapper issue: inside an ordinary function, `Base/current-namespace`
  is the function's owning namespace, both `resolve` and `Base/resolve` miss
  the caller's unqualified Var, and both `eval` and `Runtime/eval` fail for it.
  Existing explicit namespace evaluation is available, but selecting where
  caller context is captured/passed requires tracing the preprocessing callers;
  replacing prelude calls with intrinsics alone does not repair the contract.
  The pinned `emit-preprocess` is a publication-only facade and has no
  path-matched historical test at the pin; inventory that absence explicitly,
  then publish its owning preprocessing functions after their contracts pass.

- Predicate fact migration now wraps successfully returned values in a
  `:code.migrate/returned` vector and checks that envelope before applying the
  historical predicate. This prevents native code.test from treating a caught
  host-error string as a successful string-valued result. It preserves the
  original expression, evaluates it once, propagates errors, and retains
  comments/quoted fixtures/lexical fact boundaries. Clojure negative controls
  reject the previous rule; the native harness verifies one real string passes
  and one host failure fails. Written/reloaded focused tests pass 19/19, and
  the full migration-tooling suite passes 209/209. This correction is in the
  migration layer, not an unauthorized native test-runner semantic change.

- Built and natively exercised an unwritten UUID candidate covering 67 seed
  cases: signed longs, big integers, UTF-16 string hashes, keywords, symbols,
  vectors, maps and the 50 finite-double fixtures. Numeric hashing reconstructs
  Java long/double hash bits using existing native arithmetic. Native `mod`
  returns signed remainders, so explicit positive-mod normalization is required
  for unsigned-word hashing; this fixes -1 and negative big-integer cases.
  After that correction 66/67 exact UUID values match Foundation. The remaining
  map case differs because native iteration order and map printing differ from
  Foundation; asked the user whether to require ordered maps or canonicalize
  this seed domain. Do not install the current candidate's ordinary-map branch.
  Exact authority hashes/UUIDs are recorded in
  `resources/code/migrate/fixtures/uuid-seed-hashing.edn`, with the ambiguous
  map case explicitly marked. The live candidate is `uuid-positive-mod-form`
  in the Foundation REPL; no UUID migration rule or native common file has been
  written for this candidate yet.

- Installed the numeric helper source/test reconciliation with exact recovery
  in `plans/lang-migration-numeric-helper-before.edn`. Only the owning
  `default-emit-fn` source form changed; the intervening multiline namespace
  formatting is preserved. Numeric tests are owned by the migration catalog,
  not hand-maintained output. The old helper fails the new tests. Scaffold
  preview/write plans have no additions; missing/incomplete/unchecked are empty.
  Complete candidates and a fresh load of the written source/test pass 56
  assertions; the written path-matched suite passes 40 registered tests.
  Installed bytes match the recovery after-hashes; test regeneration is exact,
  and repeated source generation is stable, with the documented namespace
  layout difference still pending reconciliation. Tooling remains 208/208.
  Generated common's random fact now passes both actual integer and floating
  readback checks. The common aggregate reports 85/86, but its unseeded UUID
  check incorrectly accepts the thrown error message as a string via native
  predicate matching. Both UUID flows are still semantically unresolved; that
  apparent pass is not UUID fidelity evidence.

- Implemented the finite-double default-emitter migration using exact integer
  arithmetic: derive the binary rational, search shortest round-tripping
  decimal candidates with the JVM's minimum precision/nearest-even choice,
  and apply fixed/scientific notation boundaries. It uses existing native
  arithmetic and parsing, adds no native API, and leaves custom callbacks
  untouched. All 50 golden strings and round trips pass in a fresh native
  candidate process; signed zero is supplied through runtime parsing. A JVM
  differential check of 1,000 deterministic finite bit patterns also has zero
  mismatches. The permanent regression covers all 50 strings/raw bits, unchanged
  nonfloat forms, metadata prefix and idempotence; removing the adaptation
  produces a failing test. Written/reloaded focused checks pass 45/45 and the
  full tooling suite passes 208/208. This is written migration tooling only:
  regenerate/reconcile the complete helper source/test pair with recovery and
  native permanent numeric tests before installing it, then rerun common.

- Captured 50 exact finite-double authority cases in
  `reference/foundation-base/resources/code/migrate/fixtures/finite-double-spelling.edn`.
  They cover positive/negative zero, the first 20 subnormals, extrema, and both
  adjacent representable values around notation boundaries. Every case stores
  decimal input, exact expected spelling and raw IEEE-754 bits; read-back
  verification passes all 50 against the actual Foundation JVM 26.0.2.
  Inspected that JVM's local `DoubleToDecimal.java`: it uses Schubfach decimal
  selection as well as fixed/scientific formatting, so changing exponent case
  alone is insufficient (smallest subnormal is `4.9E-324`). A faithful native
  implementation must select the closest shortest round-tripping decimal with
  the JVM's minimum-digit and tie rules, then apply its notation thresholds.

- Investigated the numeric default-emitter owner with a complete generated
  helper candidate. Removing native float constructor syntax and retaining a
  decimal point gives Float readback for zero, halves, integral floats and large
  values. However, it does not reproduce Foundation's actual spelling:
  native `10000000.0` versus JVM `1.0E7`, and native
  `100000000000000000000000.0` versus the current Foundation JVM's `1.0E23`.
  The candidate passed value-roundtrip probes but is not faithful formatting,
  so it was NOT written or installed; the live helper adapter was restored to
  the disk version. Preserve signed-zero and scientific-notation fixtures when
  implementing the real numeric spelling boundary. Do not replace this with
  a random-only output patch or weaken the historical numeric predicate.

- Shared structural fact normalization now wraps bare integer/float/double/
  string predicate expectations in native `satisfies`. It rewrites only
  top-level assertion expectation blocks, preserving quoted fixtures, comments,
  already-constructed checkers and nested lexical facts byte-for-byte. The
  negative control fails against the old normalizer; written/reloaded focused
  checks pass 18/18 and the full tooling regression passes 207/207. This fixes
  predicate semantics rather than changing the asserted expression or replacing
  the numeric/string contract with a truthy check.
  One whole-run summary reported 84/86 passing, but a repeat inspecting the
  actual random fact shows integer passing and floating readback failing.
  Direct verification proves read-string of native printed 0.5 has type
  `:std.native.List`, is not double?, and is unequal to 0.5. Therefore the
  84/86 summary is not accepted as float fidelity evidence: UUID and numeric
  emission remain unresolved. Preserve detailed fact results in subsequent
  runs rather than relying on the aggregate count alone.

- Mapped the common historical `float?` checker to the native IEEE-754
  `double?` predicate, leaving the original read-string expression unchanged.
  Written/reloaded focused migration checks pass 44/44; the old mapping is
  detected by a negative control. All 86 historical assertions now execute:
  82 pass and four fail (two UUID and both random checks), with no random
  preparation error. Inspection of native `code.test.checker/matches?` explains
  why even the valid integer random value currently fails: bare expected
  functions are compared as values, unlike Foundation predicate shorthand.
  They must be migrated to explicit `satisfies` checkers. Separately,
  `emit-helper/default-emit-fn` still uses native `pr-str`, whose float output
  is constructor syntax; read-string therefore returns a form, not a float.
  Fix the checker translation and the owning default-print boundary, not the
  random generator's callback semantics or historical assertions.

- Reconciled free-emission input contracts: the pinned options map supplies
  `:sep`, while the existing native direct separator string remains supported.
  Metadata and the historical multiline implementation remain in the generated
  definition. Empty maps and nil preserve default spacing. Fresh native output
  for map/string/empty/nil inputs is exactly `["1,2" "1;2" "1 2" "1 2"]`.
  The previous adapter fails the new regression; written/reloaded checks pass
  41/41, and the full migration-tooling regression passes 203/203. Full-common
  execution is now 82 passed, two failed out of 84 executed assertions. The
  two UUID checks fail; random preparation still prevents two of the 86
  declared assertions from executing. No common native files are installed.

- Fixed common lookup dispatch through Var references while retaining direct
  function callbacks and dynamic root lookup. Native Vars require dereference
  before invocation; use `Base/type` to identify `:std.native.Var` because the
  dispatcher already binds a local named `type`. The regression checks direct
  callbacks, changed isolated Var roots, exact structural output, quotes and
  idempotence, and detects the original rule. Written/reloaded focused tests
  pass 40/40. Fresh full-common execution now passes 81 of 84 assertions, with
  three failing checks and random preparation still blocking two of the 86
  declared assertions. Internal-string and recursive common-emitter checks
  now pass. Remaining executing failures are free-emission argument shape and
  the two UUID checks. The common native pair remains uninstalled.

- Common symbol replacement now uses the helper migration's simultaneous
  character lookup idiom, preserving unmapped characters, nil/false mapped
  values, quote boundaries and idempotence. Written regression checks pass
  39/39; full tooling passes 201/201 before the subsequent list refinement.
  This raised whole-common parity to 73/84 executed assertions.
  Fresh native probes then isolated the nil-position defect: destructuring
  `(seq [true 'x 'y])` yields a nil first binding in this runtime, while a
  persistent list preserves true. Refined finite rest materialization to
  `(not-empty (apply list (vec value)))`, preserving nil for an empty rest
  without exposing sequence-backed destructuring. The regenerated full pair
  now passes 77 assertions with seven failures out of 84 executed; all ternary
  and constructor checks pass. Written/reloaded focused checks remain 39/39.
  Remaining cases: free-emission input shape, internal/common dispatch,
  seeded/unseeded UUID, and random-test preparation (`float?`). The pair is
  still planning-only; no common source/test artifact has been installed.

- Diagnosed argument loss with fresh native probes: `&` rest bindings are
  iterators, whereas ordinary persistent lists and concatenation remain
  repeatable. Added `rewrite-common-rest-bindings` to materialize finite
  common-emitter rest bindings as reusable sequences, in both source and test
  fixtures. It handles local bindings, named/anonymous functions and arities,
  preserves `:as`, quote boundaries and repeat-rewrite identity. This is a
  finite emitter-input adaptation, not a change to general native iterator
  semantics. The negative control detects its absence; written/reloaded
  focused checks pass 38/38 and migration-tooling checks pass 200/200.
  Regenerating both members of the full pair yields 68 passing and 16 failing
  assertions out of 84 executed (86 declared, random preparation still fails).
  Macro, static and general invocation historical checks now pass. Remaining
  ternary/constructor cases produce nil in particular positions despite basic
  standalone shadowing, rest-binding and apply-list probes passing; do not
  classify these as a proven general native binding bug yet. Common remains
  planning-only and uninstalled.

- Core thread forms in common emission are expanded before one-argument join
  normalization, so `(->> xs (clojure.string/join separator))` does not gain
  an erroneous extra argument. Invocation length summation uses an explicit
  zero identity through `(reduce + 0 (map count str-array))`. Tests preserve
  quoted data, empty/single/multiple lengths and repeat-generation identity;
  the old rule is detected by the negative control. Written/reloaded focused
  checks pass 37/37 and the seven-namespace tooling regression passes 199/199.
  Fresh whole-common validation improves to 62 passed, 22 failed, 84 executed
  assertions (86 declared); only random-test preparation remains blocked.
  Return emission now passes. Static/general invocation now executes but loses
  arguments (`table.new()`/`call()`), exposing a remaining sequence/argument
  handling defect rather than permitting weaker expected output. This pair
  remains planning-only, not an installed completed migration.

- Added structural common-emitter assertion lowering without removing guards:
  evaluate the condition once, return nil on success, evaluate the message only
  on failure, and retain the source condition in the original assertion message.
  Java exception identity is recorded as `:migration/source-class`; it is not
  assigned to Hara's reserved `:ex/class`. A fresh whole generated pair now
  passes 58 of 83 executed assertions (25 failures, 17 throwing facts); all 58
  facts register, with 86 assertions declared and two preparation errors still
  outstanding. A direct failing `emit-pre` call verifies the intended assertion
  message and provenance instead of accepting an unrelated exception. The
  candidate regression detects the previous rule; the complete common native
  pair remains uninstalled pending the remaining parity failures.

- Persisted the common invocation keyword-index adaptation: the exact
  `(collection/index-at keyword? args)` form routes to the retained native
  `first-keyword-index` helper. Quoted data and other predicates are untouched;
  repeated rewriting is stable. The new regression detects the prior rule
  (34 passed, one throw); after writing and reloading, all 35 focused checks
  pass. The seven-namespace migration tooling regression now passes 197 checks
  with zero failures, throws, or timeouts. `src/tahto` and `test/tahto` remain
  unchanged. This persists an already-probed REPL candidate, not a new claim
  of whole-common parity.
- Latest whole generated common diagnostic: all 58 historical facts register,
  but only 83 of 86 declared assertions execute; 45 pass and 38 fail, with two
  facts failing during preparation. The outer diagnostic harness passing is
  not a candidate pass. Remaining failure families include unbound `assert`,
  nested/rest destructuring, zero-argument addition, threaded join arity,
  simultaneous symbol replacement, random readback, and seeded UUID fidelity.
  The common pair remains planning-only and is not installed. Use this full
  pair to drive shared rule fixes rather than interpreting old native consumer
  green counts as historical parity.

- Reconciled semantic-reader regeneration in helper source/test and provenance
  test. Exact pre-edit text and SHA-256 before/after values are retained in
  `lang-migration-literal-helper-source-before.edn`,
  `lang-migration-literal-helper-test-before.edn`, and
  `lang-migration-literal-provenance-test-before.edn`. Native candidates pass
  helper 53 assertions and provenance 26 assertions, with no scaffold, missing,
  incomplete, or unchecked findings through the native runtime API. Written
  path-matched tests pass twice in fresh runtimes (39 helper and 25 provenance
  registered facts); direct loading of the written helper followed by its
  written test again passes 53 assertions. All six targets repeat generation
  byte-exactly; all installed test files and five source files match generated
  output. The only remaining installed mismatch is the previously recorded
  emit-template namespace formatting, which remains untouched.
  The installed `hara --project . --offline manage scaffold` launcher crashed
  with a stack overflow; native runtime API scaffold verification succeeded.
  An initial helper transport was truncated inside a long JSON field; it was
  rejected before any native file write, retransferred field-by-field with
  length checks, and then validated successfully. This was transport failure,
  not a Clojure test or source parser defect.

- Added Foundation placeholder-thread lowering using its existing
  `std.lib.foundation/thread-form` implementation. Explicit `%`, ordinary
  callable steps, zero steps, quoted data, sequential single evaluation, and
  idempotence are covered; the prior adapter fails the new tests. Native
  execution confirms event order. One-argument Clojure join is adapted to
  `(str/join "" collection)` after thread expansion; two native join-arity
  regressions fail without this rule. A generated native probe passes three
  checks: direct ternary emission, the pinned invocation-layout fact with real
  newlines, and forced multiline indentation.
  This exposed and fixed a shared literal-fidelity defect:
  `rewrite-owned-source` now reads expression block text with `*read-eval*`
  disabled, instead of treating raw `std.block/value` string escapes as decoded
  semantic values. The escaped newline/tab/quote/backslash/regex regression
  fails with the old reader and passes with the fix, exact block recovery, and
  idempotence. Written migration suites pass 194 checks.
  Six-target regeneration was audited but not installed. Preprocess-base and
  both rewrite pairs remain exact. Emit-template retains its known namespace
  formatting drift. Provenance test and helper source/test now differ in
  map/metadata ordering; read forms compare equal (which does not prove native
  map display/order parity). Validate and reconcile these three generated
  files before claiming byte-exact installed regeneration again. No native
  source/test files were overwritten during this slice. Common remains a
  planning target, not an installed port.

- Common adaptation is now wired into public `plan-unit` through the explicit
  `:foundation/lang-emit-common` rule. Its regression proves that random
  rewriting occurs while unresolved UUID host code remains diagnostic and
  `migrate-unit` rejects installation; disabling routing fails the test.
  Added owned namespace, form predicate, string join/case, error, and scalar
  string-conversion mappings. Prose resolves to `std.lib.format.prose`, util to
  `lang.base.util`, and ordinary common source retains the default prelude.
  Three structural regressions fail against the prior adapter; a fresh native
  probe passes nil/boolean/namespaced keyword/symbol/string/UTF-8 byte contracts.
  A whole-pair planning pass retains 58 historical facts and 86 assertions,
  with eight explicit title adaptations, 23 source block adaptations, and one
  test namespace adaptation. Written migration suites pass 186 checks.
  The working target is REPL-only, marked planning, with expected assertions 86;
  no common source/test files have been installed. Diagnostics are not a native
  load proof: nested loops, placeholder threads, keyword parsing, existing native
  fixes, and other structural contracts still need reconciliation.
  UUID investigation disproved blanket `Base/hash` use: native string hashes
  match Java, but `Base/hash -1` is -31 (Java Long.hashCode is 0), and native
  0.5 hashes to 156 (Java Double.hashCode is 1071644672). Intrinsic IHash did not
  support the probed primitive receiver. Preserve exact numeric seed semantics;
  do not hide them behind a string-only UUID implementation or native hash.

- Began the common-emitter structural adapter with `emit-with-rand`. The
  recipe uses existing `Crypto/random-bytes`: 31-bit rejection sampling gives
  integers in [0, 2147483647), and 53 random bits give doubles in [0, 1).
  Historical docstring/metadata and the configured emitter's grammar/options
  forwarding are preserved. Deterministic entropy fixtures prove zero, upper
  boundaries, excluded-integer retry, forwarding, and rewrite idempotence;
  deliberately accepting the excluded integer makes the test fail. A fresh
  native candidate verifies types, bounds, and nonconstant samples. Nested
  rest destructuring was explicitly lowered after reproducing its native
  protocol error. Written migration regression passes 182 checks. This partial
  recipe is declared with `:implemented-symbols [emit-with-rand]`; it is not
  installed or wired as a completed common-emitter target. Remaining work
  includes seeded JVM hash fidelity for UUIDs, UUID raw-text emission (native
  `str` retains a reader tag), float readback representation, the other common
  operations, title mappings, full source/test recovery, and regeneration.

- Added explicit `:target/fact-descriptions` adaptations to native fact
  migration. Structural block replacement changes only selected titles;
  mappings require a unique refer, a nonempty title, and a reason. Indexed
  before/after blocks retain recovery evidence, while original input/checksums
  and historical assertion counts remain intact. Unmapped collisions still
  fail closed. The new pipeline test fails against the previous implementation
  and passes after restoration. Fresh native generated probes register both
  formerly colliding facts: the deliberate negative reports one pass/one fail,
  and the corrected version reports two passes. Written migration suites pass
  181 checks. The adapter also accepts quoted native refer metadata and proves
  block-level idempotence. Full repeated migration of legacy-format quoted
  metadata still hits the existing Foundation `gather-meta` boundary; this is
  distinct from catalog-selected native fact regeneration. The common emitter
  has not yet been installed or added to the catalog, and its UUID/random
  implementation failures remain outstanding.

- `emit-common` coverage audit found a false-green condition: native
  `code.test.registry/fact-key` keys facts by namespace plus description, and
  duplicate descriptions overwrite earlier facts. A fresh two-fact probe with
  an intentionally failing first fact and passing second fact reports only one
  passing fact. The pinned common test has 58 facts/86 assertions; the existing
  native file has 71 facts/101 assertions, but four duplicated descriptions
  reduce native registration to 67. The UUID assertion is among the overwritten
  facts: a qualified fresh probe fails because source still returns `uuid-*`.
  Random emission also remains a fixed-value placeholder, with narrowed native
  expectations compared to the historical integer?/float? assertions. Previous
  common consumer green counts cover only registered survivors, not the whole
  file; do not use them as full parity evidence.
  Native fact migration now reports deterministic
  `:migration/duplicate-native-fact-description` diagnostics, and public
  migration rejects these collisions before staging. The old generator fails
  the new regression; unique descriptions remain accepted. Seven written
  migration suites pass 176 checks. Common is not in the catalog or regenerated
  yet: resolve registration identity without dropping original assertions or
  titles, then preserve real seeded UUID/random contracts and existing native
  emission fixes in the full pair.

- Added the reusable `normalize-fact-checkers` block rule to native fact
  migration and declared `:foundation/native-fact-checkers` in the test rule
  library. It constructs bare `throws` only after assertion markers in
  top-level facts, including metadata wrappers; comments, strings, explicit
  and reader quotes, already constructed checkers, actual expressions, and
  nested lexical facts remain unchanged. Original input/checksums and assertion
  counts are retained, and a clean second normalization is identical. Two
  regressions fail with normalization disabled; the generated native throwing
  and exact-value probe passes both checks. Seven written migration suites
  pass 174 checks. All six installed test files remain byte-exact; all source
  files except the previously recorded emit-template formatting difference
  remain exact too. No new generated-file drift was introduced. The helper's
  existing explicit checker adaptation remains compatible with this shared
  rule; its removal is not needed to activate the rule for subsequent pairs.

- Isolated the quoted-vector failure below the migration layer. Native facts
  preserve a vector in isolation, but a fresh file containing equal quoted
  list and vector literals reports `List` for both; both direct `Test/check`
  and `fact` probes fail their representation assertions. The VM compiler's
  `constant_index: HashMap<Value, u32>` and `constant_index_of` deduplicate by
  language value equality, which conflates the two sequence representations.
  Removing only the helper vector-constructor adaptation reproduces one
  failing historical assertion out of 53. Restored the saved adapter in the
  REPL afterward; both installed helper files still regenerate exactly, and
  public/adapter regressions pass 36 checks. The workaround is required by
  constant pooling, not by native fact syntax. No compiler runtime change has
  been made. Fixing the compiler's representation-sensitive constant identity
  would allow removing the fixture adaptation; do not change language `=`
  semantics to fix constant pooling.

- Installed `emit-helper` as the sixth pinned source/test pair. Public rule
  selection, declared rule, catalog ownership, source/test additions, and
  recovery are written. The source retains five native helper APIs; the
  existing empty-input `drop-last-one` failure is guarded. Ten new assertions
  cover helper boundaries, exact default grammar data, simultaneous symbol
  replacement, and Cons/Seq form parity. Native `form?`, not `list?`, preserves
  the authority's sequence domain. All 53 candidate assertions pass, and all
  ten deliberately incorrect expectations fail against the written source.
  Scaffold/missing/incomplete/unchecked inventories are empty. Exact prior
  files and indexed adaptation evidence live in
  `plans/lang-migration-helper-before.edn`. Both installed files match pinned
  regeneration byte-for-byte, and repeated generation is identical.
  Seven written migration suites pass 170 checks. Fresh written helper/common/
  data/function tests pass 138 registered tests (39/67/22/10); consumers are
  compatibility-verified, not newly regenerated. No authority language files
  were changed. The pointer contract decision, remaining emitter/core closure,
  real JS/Lua/Python processes, and emitted bootstraps remain open.

- Implemented `code.migrate.lang/rewrite-emit-helper-form` and its written
  regression tests. The adapter preserves the literal `+default+` and
  `+sym-replace+` blocks, routes existing dependencies to native owners, lowers
  string replacement without cascading character substitutions, and explicitly
  lowers destructured loop state plus nil options. Source reconstruction and
  idempotence pass; disabling the adapter fails three checks. Seven written
  migration suites pass 169 checks.
  The complete candidate, including five retained native helper functions and
  all eighteen existing native checks, passes all 43 assertions (25 historical
  plus 18 native) in a fresh native session. The historical throws checker is
  constructed explicitly, and the quoted-vector negative fixture uses an
  ordinary vector constructor while list fixtures remain lists; the matcher
  checks collection type rather than relying on Clojure sequential equality.
  Scaffold reports seven fact obligations: five retained helper functions,
  `+sym-replace+`, and `+default+`. Strengthen those contracts before native
  installation. Public adapter selection, catalog target, recovery evidence,
  generated native files, and written native/consumer verification are still
  pending; the current catalog remains five targets. No emitter pair was
  replaced by this candidate work.

- `emit-helper` pinned source/test planning inventories 25 historical
  assertions. Its `emit-symbol-full` contains executable `(. sym-str
  (replaceAll ...))`, which previously escaped migration diagnostics because
  bare dot symbols were exempted for language fixtures. The engine now detects
  dot/chained-dot call forms structurally while retaining quoted fixtures and
  literal strings. Two new assertions fail against the old diagnostic; all
  three pass with the fix, and the actual helper plan now reports the host
  boundary. Seven written migration suites pass 165 checks. Helper namespace
  routing/prelude handling, string replacement, typed-argument idioms, native
  helper preservation, and complete native validation remain before installing
  this pair. Its candidate target exists only in the REPL, not the catalog.

- Pinned workflow fixture staging now forwards the same authority root and
  revision used for source/tests. Historical text fixtures are read from git
  blobs, not mutable working-tree paths, and their manifest entries record the
  authority revision. All fixture paths and blobs are prepared before the
  first fixture write; a missing later blob or invalid pin rejects the batch
  without partial fixture writes. Legacy unpinned calls retain their existing
  three-argument entry point. New tests exercise a real pinned block fixture,
  exact staged content/checksum, missing-blob rejection, nil-pin rejection,
  and caller routing. The old caller fails the routing regression; the restored
  candidate and written implementation pass. Seven written migration suites
  now pass 162 checks. This closes text-fixture pinning, not binary fixture
  support, transactional installation, or dependency snapshot guarding.

- Pointer ownership follow-up found an explicit native contract, not merely
  an omitted implementation: `hara-native/core/rust/src/lang/data/pointer.rs`
  documents pointers as immutable descriptors with runtime resolution owned
  by the active evaluator, deliberately excluding embedded runtime/resolver
  ownership. Foundation's `pointer-default` instead prioritizes `*runtime*`,
  `:context/rt`, and `:context/fn` before space lookup. Restoring that behavior
  inside builtin pointer dispatch would change a documented native semantic
  boundary. Obtain the user's choice before changing builtin dispatch or
  substituting a separate pointer abstraction; neither is a neutral spelling
  adaptation. No native runtime edit has been made.

- Next dependency audit: current native `lang.base.util` lacks the authority's
  `lang-rt-list`, `lang-rt`, `lang-rt-default`, and `lang-pointer`. The native
  space implementation exists, and pointers are builtin rather than owned by
  a `std.lib.context.pointer` source file. A fresh, state-isolated native probe
  confirms two relevant mismatches: `IApplicable/apply-default` bypasses the
  retained `:context/fn` resolver and selects the space runtime; keyword lookup
  of `:context` returns nil while `IPointer/ptr-context` returns the context.
  The lookup parity expectation failed; an explicit six-value characterization
  then passed, including the resolver call counter. The probe uses registered
  `:null`; an unregistered context throws before the resolver is considered.
  Evidence agrees with `hara-native/core/rust/src/core/protocol.rs`'s
  `pointer_default`. This is a runtime/pointer seam, not a missing symbol rule;
  merely replacing `ptr/pointer` with `pointer` would not preserve invocation
  semantics. No util source/test or runtime implementation was changed by this
  audit. Resolve this seam explicitly while continuing dependency-ordered work;
  do not discard those historical functions or claim util parity from its
  thirteen currently passing native tests.

- Installed provenance as the fifth pinned source/test pair. Public adapter
  selection, catalog ownership, historical assertions, all twelve existing
  native checks, and two field-contract assertions are now written. Complete
  candidate validation passes 26 assertions across 25 registered tests; the
  nine-field negative control fails both new assertions. Scaffold, missing,
  incomplete, and unchecked inventories are empty. The installed source and
  test regenerate byte-for-byte; exact previous native files and indexed
  adaptation evidence are retained in `plans/lang-migration-provenance-before.edn`.
  Seven written Clojure migration suites pass 160 checks. Fresh written
  provenance/util/emit-helper/emit-common runs pass 123 registered tests total;
  the consumer passes are compatibility evidence, not completed regeneration.

- Repeated current-layout staging with all five catalog pairs in an isolated
  new stage. Fresh native verification passes 13 preprocessing, 18 template,
  6 common rewrite, 4 destructuring, and 25 provenance registered tests: 66
  total. All ten staged files match a second pinned generation byte-for-byte.
  `plans/lang-migration-five-stage-audit.edn` records stage identity, hashes,
  native counts, and exact-regeneration results. No stage install was performed.
  Foundation `src/tahto` and `test/tahto` remain unchanged. This supersedes the
  earlier four-target/provenance-unwritten progress entries below.

- Following the user's protocol-registry correction, removed `IPeekLast` from
  the native-class allowlist. The engine now reads a checksummed exact copy of
  `01-lang/002-protocol/draft/protocol-spec.edn`: 55 descriptors and 102 declared
  methods, indexed under both short and canonical names with fixed/variadic
  arities retained. Undeclared methods such as `IPeekLast/not-declared` are not
  accepted. Disabling the protocol inventory fails two regression checks.
  All 18 copied authority files match their recorded hashes; the protocol copy
  also matches the registry byte-for-byte. Seven written migration suites pass
  159 checks. This replaces the earlier one-off protocol allowlist change.
  Provenance public selection and its field-contract test are evaluated REPL
  candidates, but remain unwritten; the native provenance files and active
  four-target catalog have not yet been replaced.

- Implemented and REPL-verified the provenance structural adapter in
  `code.migrate.lang`, including canonical data keys, native namespace lookup,
  Throwable/IObj boundaries, existing extra fields, payload sanitization,
  threaded apply handling, and native cause preservation. The block adapter
  retains indexed before-images; tests prove exact reconstruction and
  idempotence. Corrected the shared renderer to preserve top-level fact
  metadata explicitly and avoid unsupported namespaced-map reader syntax.
  The old renderer fails two regression checks. All 12 historical `:refer`
  entries now survive. Native candidate validation passes 24 checks (12
  historical plus all 12 existing native checks), including error wrapping.
  Scaffold identifies one remaining obligation: a behavioral `+field-keys+`
  fact. The candidate is NOT installed or in the active catalog yet; public
  adapter selection, that fact, guarded recovery, and exact regeneration still
  precede installation. Written migration regressions pass 155 checks.
  Added the verified preloaded `IPeekLast` owner to diagnostics and fixed
  detection of fully qualified Java/Javax calls; JVM calls remain rejected.

- Traced the shared emitter dependency chain through `emit-helper` and `util`
  to `provenance`. Corrected the existing JVM exception constructor rule so it
  emits native `ex`, not forbidden `ex-info`; added selected
  `:clojure/ex-info-native` lowering for message/data/optional-cause forms.
  A function application preserves caller scope and left-to-right, exactly-once
  argument evaluation. Quoted forms and comments remain unchanged. The prior
  rule fails the new regression checks. Written engine checks pass 93, public
  catalog activation and repeated generation pass, and the seven migration
  suites pass 147 checks. A fresh native probe verifies message, payload,
  cause identity/message, and observable argument evaluation order.
  Provenance itself is not regenerated yet: namespace/IObj handling, canonical
  keyword routing, six existing native field additions, and native exception
  payload sanitization require explicit reviewed adaptations. No Foundation
  authority language source or test was changed.

- Added `lang.core.rewrite.destructure` as the fourth catalog target using only
  existing namespace and anonymous-function structural rules. All four functions
  and four historical assertions remain, including sorted bindings, custom key
  naming, empty-set nil behavior, and canonical `x:get-key` forms. Native
  candidate and written checks pass; replacing key access with index access
  makes two assertions fail. Scaffold, missing, incomplete, and unchecked
  reports are empty; both installed files regenerate byte-exactly.
  Previous files are retained in `plans/lang-migration-destructure-before.edn`.
  Written migration regressions remain 143 passing checks. The pin's consumers
  are Ruby/R/Julia rewrite modules, not current native JS/Lua/Python consumers;
  this is core-library coverage, not process-pipeline completion.
  Next shared-emitter work must follow `emit-common`'s actual dependencies:
  `emit-helper`, preprocessing, `util`, and collection/string support. Its
  existing 67 native tests do not establish faithful source/test regeneration.

- Current-layout staging and real native verification now work for all three
  catalog pairs. Pinned `prepare!` copies the project's declared source and
  extension roots, preserves `project.edn`, includes its declared recipe, and
  requires an empty stage beneath Hara's target directory. The first real run
  exposed the omitted recipe; the corrected fresh stage passes preprocessing
  (13 registered tests), emit-template (18), and rewrite-common (6), each in a
  fresh native process. Verification rejects empty/skipped/failing summaries
  and retains checksum guards; legacy command routing remains separate.
  All six generated source/test files match a second pinned generation exactly.
  `plans/lang-migration-stage-audit.edn` records the stage, checksums, manifests,
  and native counts. No stage was installed over the working tree.
  Seven written migration suites pass 143 checks. Pinned test-data reads,
  transactional installation/recovery, catalog closure, remaining language
  migration, real processes, and emitted bootstraps remain incomplete.

- Connected `workflow/prepare-target!` to pure `generate-target`: pinned catalogs
  now use the checked pinned-pair API, reject substituted target definitions,
  and record authority revision plus original source/test checksums in staged
  manifests. Legacy unpinned source-only/test-only behavior remains supported.
  The new staging tests mock every filesystem mutation and check exact generated
  bytes, provenance, existing-target guards, and rejection before writes.
  Running against the old `prepare-target!` detects its mutable reader; restoring
  the candidate passes. Written regression evidence: seven suites, 138 checks,
  zero failures/throws/timeouts; Foundation diff whitespace check is clean and
  `src/tahto`/`test/tahto` remain unchanged.
  End-to-end staging is not yet operational: `prepare!` still uses legacy
  layout/bootstrap paths, test-data reads are not yet pinned, and verification
  still emits the old CLI shape. The current host advertises
  `hara-native test --project PATH --file PATH` with fresh runtimes. No staging
  install or broader language completion is claimed from mocked tests.

- Persisted `code.migrate/read-pinned-unit` and `migrate-pinned-pair` so callers
  generate directly from the catalog's exact Git commit without manually
  fetching authority text into the REPL. Invalid revisions, paths, object
  types, unavailable commits, ambiguous targets, and historical assertion
  count changes are rejected. The focused suite passes 13 checks; a deliberately
  broken reader is detected. The six migration suites pass 134 checks together.
  All three catalog targets regenerate deterministically. Preprocessing and
  rewrite-common source/tests, and the emit-template test, match installed bytes;
  emit-template's source-formatting difference remains explicit.
  Example, from Foundation's REPL:
  `(code.migrate/migrate-pinned-pair "." (code.migrate/load-catalog "resources/code/migrate/lang.edn") :migration/lang-base-preprocess-base)`.
  The old staging workflow still reads working-tree inputs and uses the former
  `lib/src` layout; it has not been used to install this migration. Connecting
  pinned reads and updating guarded staging/verification remain required work.

- The full migration goal is active, including real JS/Lua/Python processes and
  emitter-generated bootstraps. The next completed source/test slice is
  `lang.core.rewrite.common`, a dependency of walk, hoist, and inline-do rewrites.
  It uses the existing `:clojure/iobj-native` rule. The previous collection-only
  metadata predicate lost symbol metadata; the generated IObjType predicate
  preserves it and clears old metadata correctly. Both new symbol assertions
  fail the pre-change source. All five historical checks and six prior native
  checks remain, with three new metadata assertions (14 total).
  The candidate has no missing, incomplete, unchecked, or scaffold obligations.
  Source and test match persisted-catalog generation exactly; before-images
  are retained in `plans/lang-migration-rewrite-common-before.edn`.
  Fresh written tests pass 19 registered tests across common (6), walk (8),
  inline-do (3), and hoist (2). These consumer passes do not imply those
  consumers are already faithfully regenerated.
- Added catalog-owned `:target/namespace-config` support to namespace rewriting
  so the generated common namespace retains its existing internal role.
  The written engine suite passes 90 checks; the three-target catalog suite
  passes 13 checks. Pinned Foundation language authority remains unchanged.

- Dependency-ordered execution started with `lang.base.preprocess-base`, a
  source dependency of `emit-common` with no language-library requirements.
  `plans/lang-migration-dependency-audit.edn` records 103 pinned source files
  across base/common/core/runtime.basic, declared dependencies, symbols, paired
  test availability, and existing native paths. This is an initial inventory:
  typed/model/external edges and two files without namespace declarations remain
  explicit boundaries, not assumed complete migrations.
- The preprocessing source/test pair is generated using the active upstream
  rules and the existing unused-require rule; no bespoke source adapter was
  added. All four historical assertions and four previous native checks remain.
  Five explicit dynamic-Var facts test baselines, overrides, nested option
  bindings, and exception restoration. The prior `false` skip-deps baseline
  fails the new regression; the pin's `nil` baseline passes. All five new facts
  reject deliberately wrong dynamic baselines. Scaffold, missing, incomplete,
  and unchecked findings are empty. The written pair regenerates byte-exactly.
  Recovery before-images are in `plans/lang-migration-preprocess-base-before.edn`.
  Fresh native checks pass 110 registered tests across preprocess-base (13),
  preprocess-resolve (12), emit-common (67), and emit-template (18). Consumer
  passes are compatibility evidence, not proof those consumers are regenerated.
  The written catalog suite passes 13 assertions and language authority remains
  unchanged. The language catalog now has two targets, with preprocessing first.

- Copied specs are now imported by both Foundation migration catalogs through
  `:migration/upstream-paths`. All 38 rule records are retained; 34 map to
  existing Clojure handlers and four fail explicitly if selected: disj,
  resource/context semantic recipes, and the generic JVM function resolver.
  Existing local adaptations retain the original policy in `:rule/upstream`;
  upstream function symbols are not evaluated and upstream target plans do not
  replace local ownership. Six written focused suites pass 127 checks; the
  regenerated native pilot passes 23 checks. Disabled-loader and pass-through
  selection controls fail. All copied snapshot checksums remain unchanged.
- `emit-template` was a bounded generator pilot, not evidence that its
  dependencies are fully migrated. Earlier `emit-common` repairs and focused
  tests do not establish pinned regeneration of that namespace. The next slices
  require an explicit dependency/source-test parity audit before claiming the
  core pipeline is recreated. The latest generated pilot differs from the
  current source only in namespace indentation; its test output is exact, and
  no target source was overwritten during catalog activation.

- The installed Agent Flow REPL skill and evaluation hook executable match
  their repository sources. Restored the missing pre/post evaluation entries
  in Codex hooks, preserving the shell gate. Prior configuration is retained
  at `~/.codex/hooks.json.lang-migration-before`. The existing isolated hook
  tests pass: valid candidates, rejected invalid candidates, written reloads,
  invalid post-write content, and unrelated-file handling.
- Foundation nREPL is available on port 58892. Candidate forms are evaluated
  before writes; written namespaces are reloaded and checked again. Use
  `code.test/run:current` for registered unwritten test candidates: ordinary
  `run` can execute zero checks when the candidate file does not yet exist.
- Added catalog-selected `:target/test-format :fact`. Unlike the existing
  `Test/run` path, it retains complete fact bodies and fixture placement.
  Structural routing includes quoted namespace coordinates and metadata,
  while strings/comments/whitespace remain unchanged. Unsupported assertion
  markers block instead of silently disappearing. Original source/checksums
  support reconstruction; repeated generation is unchanged.
- Corrected host diagnostics to ignore quoted data while still reporting
  executable JVM calls. The pinned fixture's quoted `JS.core` was previously
  a false positive. Negative controls reproduce missing routing, the old
  metadata rewrite, and the old diagnostic error.
- Written focused suites pass: `code.migrate.engine-test` 55 facts / 86 checks;
  `code.migrate-test` 5 facts / 8 checks; `code.migrate.lang-test` 3 facts / 7
  checks; `code.migrate.source-test` 1 fact / 3 checks; and
  `code.migrate.test-test` 3 facts / 6 checks. Earlier `:cumulative` values in
  task summaries were elapsed-time data, not assertion counts.
- `resources/code/migrate/lang.edn` is explicitly an **emit-template pilot**,
  not the full closure catalog. It reproduces all six original facts and
  eleven assertions from the pin with no generator diagnostics. Native
  execution originally passed five facts and failed the exact code-state key
  order assertion. The structural source adapter now uses `Algo/ordered-map`,
  preserves the original brief-display context, and replaces ThreadLocal state
  with scoped dynamic binding. All six historical facts / eleven checks pass
  without changing their expectations.
- The catalog now retains two existing native helpers and eight native test
  blocks as explicit, reasoned additions with original-path/checksum provenance.
  Public `plan-unit` includes them and rejects colliding definitions or namespace
  declarations. Negative controls fail when this preservation step is disabled;
  the restored written suites pass. Generation from the persisted catalog
  exactly matches the candidate. Added four explicit facts for the configuration
  Vars and helper correspondence; each fails a deliberately wrong expectation.
  The final candidate passes 18 tests / 23 checks, with no missing, incomplete,
  or unchecked findings and no further scaffold changes.
- Installed the generated pair at `src/lang/base/emit_template.hal` and
  `test/lang/base/emit_template_test.hal`. The fresh written-file test passes
  all 18 registered tests; candidate execution confirms 23 passing checks.
  A second generation from the persisted catalog is byte-exact for both files.
  Exact target before-images and verified checksums are retained in
  `plans/tool-migrate-lang-pilot-before.edn`; the target files were checked for
  drift before application. Foundation language authority is unchanged.
  This pilot used a checked `apply_patch` installation; the general transactional
  pair installer remains outstanding. `git diff --check` also identifies a
  trailing space emitted by the current pretty-printer in the generated source;
  it must be corrected in the generator, not hand-edited in the artifact.

Next: correct the generator's emitted trailing whitespace and extend
the catalog in dependency order through core emission and emitted process
bootstraps. Full inventory, current-layout verification, reversible pair
installation, and JS/Lua/Python execution remain required and incomplete.

Execution evidence (2026-09-06):

- Reviewed definition recipes now run through the shared `std.block` matcher
  and replacement engine in `tool.migrate.policy.foundation.lang`. Recipes
  require an exact namespace, unique definition name, before block, after
  block, and adaptation reason. Missing, duplicate, drifted, or ambiguous
  definitions fail before any replacement; reader conditionals require an
  explicit branch decision. This pass does not perform incidental dependency
  normalization. Tests prove unchanged surrounding Unicode/comments/literals,
  exact block reversal, retained provenance, and idempotence. The written suite
  passes eight facts / twelve native assertions; all three new regression facts
  fail deliberately broken implementations. Scaffold and exact-unit checks
  report no missing, TODO, or unchecked tests.
- The unwritten core emission candidate has been exercised with the original
  standalone-fragment/dependency fixture. It exposed a separate port defect in
  `lang.base.preprocess-resolve/fragment-template-args`: the native code assumes
  a named declaration and drops the anonymous `fn` argument vector. This is
  now corrected through an exact definition recipe. The complete written
  resolution suite passes twelve tests; both new anonymous-fragment regression
  tests fail against the previous implementation. The recipe in
  `plans/tool-migrate-lang-adaptations.edn` is explicitly a reviewed native
  intermediate adaptation, not yet a full Foundation-to-target reconstruction.
  Replay is idempotent, inverse block replacement reconstructs the complete
  previous file, and all source definitions have real corresponding tests.
  Original test blocks retain their prior order and contents after scaffolding.
  The core fixture now passes an exact comparison of the original multi-form
  output, full dependency output, and dependency order after restoring
  `emit-common/emit-invoke` to use Foundation's callee-wrapping operation.
  The written common-emitter suite passes 67 facts / 96 native assertions.
  Its chained-call regression fails the old source, and all ten new facts
  fail deliberately wrong expectations. Scaffolding filled nine existing
  configuration-Var obligations; exact-unit missing/TODO/unchecked checks are
  clean. Pre-existing changes to wrapping and empty collection handling were
  retained. The invocation recipe is also in the native-intermediate ledger;
  its full-file repeat/reversal check now passes: already-applied matching is
  empty, inverse application reconstructs the exact previous checksum, forward
  application reconstructs the written file, and repeat matching is empty.
  No historical expected output is being weakened. Source-private helpers
  remain unmarked in native code; they do not acquire `:public true` merely
  because they are migrated.
- Macro rewrite cause located read-only in the consuming native runtime:
  `technology/hara-native/core/rust/src/kernel/generated/rewrite.rs`,
  `GeneratedNamespaceConfig::resolve_symbol`. The `symbol.contains('/')`
  branch resolves bare `/` through the namespace registry and returns the
  owning Var coordinate `std.foundation//`. Recursive rewriting applies this
  before macro expansion, including to macro arguments and `&form`.
  The user approved the narrow Rust exception. The rewriter now preserves bare
  `/` after explicit refer handling, matching `Symbol::parse`. The native
  regression reproduces the exact argument and `&form` corruption before the
  fix and passes afterward; ordinary division, explicit `std.foundation//`,
  and namespace aliases remain functional. `cargo test --offline --test
  native-lang` passes all nine tests (the socket test requires localhost access).
  The consuming release binary was rebuilt with `cargo build --offline
  --release --bin hara-native`; formatting and diff checks pass. Existing
  dependency and host-emission suites pass 25 and 11 facts respectively.
  No language alias, source division operator, or historical expectation was
  changed to hide the defect.
- Written dependency-order slice: `lang.core.impl-deps` now orders modules via
  the native Book dependency protocol and entries by module index, priority,
  line, and time. Its module lookup retains linked modules with no selected
  entries; scalar-symbol requests work; module cycles are rejected. The written
  suite passes 25 facts / 26 assertions, and exact-unit correspondence reports
  no missing, TODO, or unchecked tests. All five new ordering/closure regression
  facts fail against the previous implementation. The newly scaffolded
  `collect-script` test also fails a deliberately incorrect implementation and
  verifies script-state restoration.
- Scaffolding exposed a byte/character offset defect in
  `code.manage.unit.structure/span-text`. Its reader contract explicitly uses
  byte offsets, so projection now slices UTF-8 bytes before decoding. Unicode
  comments, a Unicode docstring, and an emoji literal retain exact source text.
  The new Unicode case fails before the fix. The written native-register suite
  passes 51 checks, including an exact fact-head constant check whose negative
  control fails. Full-definition reference inventory is clean. The ordinary
  scaffold produced duplicate pending facts for existing `Test/register` tests;
  only those newly generated duplicates were removed, preserving all original
  assertions and checking the native-test references with the bootstrap reader.
  Other byte-offset edit consumers have not yet been audited comprehensively.
- Connected validation now passes all six facts in
  `test/lang/core/impl_test.hal`, including the color-script dependency closure.
  The first run after rebuilding still consumed stale bytecode: the cache key
  includes runtime version and source, not the rebuilt binary. The old generated
  `target/hara/source-bytecode/v3/native-0.1.26` directory was moved recoverably
  to `/private/tmp/hara-division-cache.tGzR84/native-0.1.26`; a cold source run
  rebuilt the cache and passed without source or expectation changes.
  Before the fix, full module traversal encountered `std.foundation//` in
  registered XTalk definitions whose source contains `/`. The minimal repro was:
  `(defmacro capture [form] (list 'quote form))` followed by
  `(capture (/ 6 2))` returns `(std.foundation// 6 2)`. Its `&form` is altered too.
  Ordinary quotes, `read-string`, and `read-forms-spanned` preserve `/`, isolating
  this to macro argument handling rather than the plain reader or source file.
  No expectation or source operator was changed to hide this failure. The native
  macro blocker is resolved; this focused result does not establish complete
  migration, deterministic regeneration, or emitted process-bootstrap parity.
- Written emission slice: `lang.base.emit-special` now evaluates embedded host
  forms and dereferences embedded Vars, then stages and emits their values.
  Native `:namespace` selects `eval-in-ns` for unqualified host names; qualified
  forms retain the direct native `eval` path. The core emission candidate still
  needs to propagate caller namespace context through its complete option path.
  `emit-with-preprocess` removes native protected-head markers at the emission
  boundary using metadata-preserving `prewalk`.
- The written path-matched suite passes 11 facts (16 assertions under native
  `Test/run`). Full candidate and written source were evaluated in fresh native
  sessions. Scaffold preview/write report no remaining additions; exact-unit
  `code.manage.unit` checks return `{:missing [] :todos [] :unchecked []}`.
  Pre-change host stubs fail the new host contracts. A deliberately incorrect
  operation filter fails its regression. The returned-form/metadata regression
  fails before marker removal and passes afterward, restoring the script
  snapshot in `finally` and verifying the restored baseline.
- With that written slice, the still-unwritten core emission candidate passes
  four grouped checks: options/arithmetic, direct controls, raw/staged host
  dereference, and vector-only bulk metadata. The bulk fixtures construct their
  lists/vectors at runtime; the separate quoted-collection runtime discrepancy
  remains unresolved. This is not full core or bootstrap parity.
- These edits were produced with `std.block` definition/namespace replacement;
  reusable source-owned adaptation recipes and deterministic regeneration from
  the pinned Foundation files remain required. Passing this slice does not
  establish that the whole `emit-special` namespace is a faithful regeneration.
- Emission candidate checkpoint (not written to `src/lang`): structural routing
  and namespace replacement preserve all six existing implementation functions.
  Fresh isolated evaluation passed the original five-slot options shape,
  staged Lua arithmetic (`1 + 2 + 3\n\n4 + 5 + 6`), and direct suppression.
  Separate probes produced the historical trimmed `1 + 2 + 3` and transformed
  `7 + 8`. These probes do not establish complete emission parity.
- Before the written slice above, expanded emission checks were red: the historical staged host-dereference
  case `(+ @1 2 3)` reaches `!:eval`, then the current
  `lang.base.emit-special/emit-with-eval` deliberately threw an unavailable
  host-evaluation error. Foundation's owner evaluates the embedded form and
  stages/emits its value. The restored slice preserves the historical expected
  arithmetic rather than changing it to an expected error.
- The expanded probe restored the script snapshot on exit, including failure.
  A separate native literal probe also exposed inconsistent list/vector
  identity for equal quoted collections; investigate the consuming runtime
  before treating the bulk-form result as an emitter defect or changing its
  historical vector-only contract.
- Dependency-order audit confirmed another remaining semantic difference:
  Foundation orders modules by dependencies, then entries by module index,
  priority, line, and time. The current native implementation topologically
  orders individual entries instead. Historical exact emission order must be
  retained when restoring `emit-entry-deps`.
- Compatible host: `hara-native` 0.1.26, native repository revision
  `45721e2d8a4079d60c7f8ebbf2ac63a33de43446`. From this repository, use
  `../hara-native/core/rust/target/release/hara-native test --project . --file <test>`.
- Initial user diff retained separately at
  `/private/tmp/hara-lang-fidelity-initial.patch`; no user changes discarded.
- Baseline tests: block 39 passed, rule 6 passed, compiler policy 34 passed.
- Reproduced two failing fidelity regressions: routing alters literal text;
  blanket alias removal corrupts `math/value` and changes `float?` expectations.
- Added the structural language-policy owner and its five corresponding tests.
  All five pass from disk; all five fail with deliberately incorrect expectations.
- Fixed native virtual-root validation required by test scaffolding. Its original
  behavior fails the root regression; the written CLI suite passes 33 tests.
- Compiler planner integration now uses the structural language policy. It
  preserves historical expectation literals and distinguishes informational
  diagnostics from blocking diagnostics when planning, validating, and applying.
- Written focused suites: shared rule 43, language policy 5, compiler policy 51,
  and scaffolding CLI 33 (132 tests total). All 43 shared-rule tests, the first
  10 compiler-policy regressions, and the seven root regressions also failed
  deliberately wrong expectations.
- Scoped correspondence is clean for the shared rule and language policy;
  bootstrap coverage is clean for the CLI. Compiler policy still has 140 missing
  symbol tests at the earlier checkpoint. The latest inventory reports 138
  missing symbols, including 96 functions/macros. Its source change is not
  complete under the correspondence gate; passing 51 tests does not close them.
- Revalidated the staged `src/tahto` and `test/tahto` trees against the read-only
  Foundation checkout with `diff -qr` (identical). The scoped checkout files,
  including the core/runtime root source and test files, have no diff against
  pinned revision `fe54cc866473a888bfbd4ce2d3de6d8011f09f72`.
- Confirmed inventory selection defect: directory prefixes omit
  `src/tahto/core.clj`, `src/tahto/runtime/basic.clj`, and their root tests.
  Existing compiler path scopes also do not route these files. They must be
  accounted for explicitly, not treated as an empty or completed migration.
- Full inventory, dependency-ordered pipeline restoration, process bootstrap
  ports, and final regeneration/parity remain outstanding. No language pipeline
  source or process bootstrap had been ported at that inventory checkpoint;
  the later written emission slice is recorded above.
- Provenance pilot structural probe passed against staged Foundation source:
  4,289 source characters, 2,237 blocks, 41 namespace/key matches, 13 definitions
  (one constant and 12 functions), and 12 historical facts before and after
  routing. A separate passing assertion verified original-source retention and
  routing idempotence. These are structural checks, not native behavior parity.
- Provenance still requires explicit host adaptations for namespace/IObj checks,
  `apply`/`peek`, Throwable metadata, exception construction, and exception
  message/class access. No historical expected result was changed by the probe.
- The first bulk inventory probe was deliberately stopped after the pilot showed
  the cost of repeated structural scans; it unnecessarily requested full
  core-symbol analysis merely to obtain a namespace name. Its replacement
  inventories definitions and facts first. Detailed dependency/symbol accounting
  remains required per slice; it is not waived by the cheaper initial inventory.
- Initial definition/fact inventory completed over 265 paths: 105 source files
  and 160 test files, including the four explicit roots. The reader successfully
  extracted 1,223 definitions and 1,188 facts from the analyzable subset, but
  reported 16 failures. These counts are incomplete, not a parity denominator.
  Failures include discarded forms (`#_`) in compiler sources, namespaced-map
  syntax in staging tests, a readme character form, and range/protocol failures
  in JavaScript/Dart/Python process test analysis. Structural block inventory
  must account for these files without deleting or rewriting historical tests.
- Exact root selection/routing is now written and verified: the complete
  written compiler-policy suite passes 51 tests, including seven new root
  regressions. All seven new regressions fail deliberately wrong expectations.
  The staged-discovery fixture verifies unchanged source bytes, deterministic
  repeated discovery, and cleanup in `finally`. Broader policy correspondence
  gaps remain; this does not complete the whole policy namespace.
- The initial inventory is retained in
  [`tool-migrate-lang-inventory.edn`](tool-migrate-lang-inventory.edn).
  Its native generator verified 265 unique records, each either analyzed or
  carrying its analysis error. Routing fields describe the pre-root-fix scan;
  the artifact explicitly marks this basis and does not claim complete closure
  or a complete historical fact denominator.
- Fresh native checks verified that current staged discovery selects 265 paths
  and that the written inventory artifact round-trips through its deterministic
  EDN renderer, retaining 265 unique records and all 16 analysis errors.

## Outcome

Build `tool.migrate` so that it reproducibly recreates Foundation's core
language pipeline in native Hara using the original `std.block` structural
matching and replacement engine. Given pinned Foundation source and tests,
the tool must produce reviewable Hara source and corresponding tests, with an
explanation for every changed form and evidence for every preserved behavior.

The priority is `lang.base`, `lang.core`, and the core
`lang.runtime.basic` execution paths. Preserve the Foundation algorithm,
declaration structure, grammar construction, stage ordering, and test
contracts while applying the established Hara namespace and host boundaries.
Current Hara behavior is comparison evidence, not the authority used to rewrite
historical expectations.

JavaScript, Lua, and Python process modules are required migration outputs.
Their bootstrap programs must be generated through the language emitter from
the original quoted client forms and shared XTalk return/evaluation helpers.
This requirement applies to the bootstrap that is actually launched, as well
as to oneshot wrappers; calling an emitter while launching a separate script
does not satisfy it.

## Inspected baseline

| Input | Revision or finding |
| --- | --- |
| Original engine | `technology/hara-archive-v1` at `02fcad8fe74d2fa40145b25b7b49a0ab76c2a622` |
| Hara target | `technology/hara` at `3bd94b9166fd0435b9965cbee2f819b1e37ed2ed`, plus existing local edits |
| Existing compiler authority | Foundation `fe54cc866473a888bfbd4ce2d3de6d8011f09f72` |
| Foundation checkout | `fbba5b865f452152d7e00c4f987963a2df7da510`; no diff from the existing authority in the four priority source/test directories |
| Initial directory inventory | 103 tracked `.clj` implementation files and 158 tracked `.clj` test files across `tahto/base`, `tahto/common`, `tahto/core`, and `tahto/runtime/basic` |
| Additional roots | `src/tahto/core.clj` and `src/tahto/runtime/basic.clj` exist outside those directory counts; include their tests and dependency closure |
| Validation prerequisite | Installed `hara` fails loading the migration tests with `Unsupported :config option: :role`; establish a compatible native runtime before interpreting test results |

The original engine already provides block paths, source ranges, structural
matching, alias handling, ordered rules, replacements, and match evidence in
`tool.migrate.common.block` and `.rule`. Its tests include preservation of
unrelated comments and whitespace. The archived migration catalog declares
zero unresolved cases, zero manual fixups, deterministic generation, source
evaluation, and passing tests as promotion requirements for the zipper and
block-check candidates. These declarations are precedent; rerun the evidence
before relying on them in the current runtime.

The current compiler policy bypasses this machinery in important places:

- `rewrite-foundation-lang-compiler-source` performs source-wide text routing.
- `normalize-foundation-test-source` performs source-wide alias replacements
  and specific expectation replacements.
- `foundation-common-fact-count` counts the substring `(fact` rather than
  parsed facts and assertions.
- Base tests and other compiler units follow different migration paths.
- Existing ledger classifications can describe a redesigned target as adapted;
  that classification alone does not prove the original behavior survived.

These are concrete starting points, not a completed audit of all migration
rules or the current language implementation.

The process bootstrap inspection also establishes a specific source/target gap:

- Foundation's `process-js`, `process-lua`, and `process-python` own quoted
  `+client-basic+` programs and `default-basic-client` factories.
- These factories assemble `impl/emit-entry-deps` for
  `xt.lang.common-lib/return-eval`, `impl/emit-as` for the client forms, and
  `impl/emit-as` for the host/port invocation. JavaScript also exposes
  `make-bootstrap` and emitted `+oneshot-require-bootstrap+` forms.
- The current Hara process modules configure their bootstraps using
  `source-js/transport-source`, `source-lua/transport-source`, and
  `source-python/transport-source`. Those owners assemble handwritten target
  programs, including separate return/evaluation helper implementations.
- Current `lang.core.impl` provides entry and script emission, but does not
  define the original `emit-as`, `emit-entry-deps-collect`, or
  `emit-entry-deps` contracts. Restoring those contracts and their dependency
  closure is a prerequisite for the required process ports.

## Fidelity contract

1. Preserve all source definitions and historical assertions in the selected
   closure, or record a specific disposition with source evidence and a
   behavioral justification. A missing implementation remains unfinished.
2. Preserve untouched source spans byte for byte: comments, whitespace,
   docstrings, string and regex literals, declaration order, and metadata.
3. Preserve literal group Vars such as `+op-*+`, their contents, merge order,
   and references. Do not flatten grammar construction into generated snapshots.
4. Preserve arities, branches, failures, macro expansion, dynamic bindings,
   dependency ordering, lifecycle transitions, and emitted target text where
   these are observable contracts.
5. Permit established namespace routing and explicit native host adaptations.
   Keep an exact record of what each adaptation changes and why. Adding a
   public semantic contract is a separate design decision.
6. Retain original source blobs and replacement evidence so the original can
   be reconstructed. Prove deterministic generation, idempotent
   canonicalization, and restoration after an interrupted write.
7. Generate runtime bootstrap behavior from the migrated process forms,
   language models, and shared `return-eval` dependency closure through `emit`.
   Keep literal target snippets only where the authoritative source explicitly
   owns one, such as Lua's small `cjson` import preamble, with provenance.
   Handwritten replacement evaluators, encoders, and complete client programs
   are not an accepted bootstrap adaptation.

## Ownership and architecture

Reuse the current implementation locations; port fixes from the archive only
when a regression fixture establishes their necessity.

| Owner | Responsibility |
| --- | --- |
| `tool.migrate.common.block` | Lossless block navigation and local replacement |
| `tool.migrate.common.rule` | Matching, rule ordering, conflicts, application, and source evidence |
| `tool.migrate.common.clojure` and `.transform` | Existing language-aware resolution and structural transformation entrypoints |
| `tool.migrate.common.corpus` and `.project` | Source/test inventory, dependency closure, and reproducible plans |
| `tool.migrate.policy.foundation.*` | Foundation namespace routes and explicit semantic adaptation rules |
| Migration catalog | Rule selection, priorities, evidence, and target declarations |
| `tool.migrate.common.write` | Validated, recoverable application of planned edits |
| `code.framework` | Pure source inspection and edit operations |
| `code.manage` | Test correspondence and task integration |

Create a focused internal language-policy module only if extracting the
compiler policy establishes a useful boundary. Do not build a second matcher,
parser, writer, or compatibility facade. All new migration automation is native
`.hal`.

## Implementation sequence

### 1. Establish the executable baseline and authority manifest

- Identify a compatible Hara Native build and record its revision alongside
  the mounted Hara source revision. Diagnose launcher, source, and embedded
  package failures separately.
- Run the archived block/rule regression cases against the current engine.
  Establish focused `code.test` and `code.manage` operation before migration.
- Pin the existing Foundation compiler revision explicitly. Read Git blobs
  from that revision, including dependencies outside the priority directories.
  Do not mix independent migration profiles silently.
- Snapshot the existing target diff and file hashes. Preserve overlapping
  local edits as an independent input to reconciliation.

Exit: a reproducible validation command, identified runtime, pinned inputs,
and classified baseline failures.

### 2. Build the complete source/test inventory

Extend the existing corpus machinery to inventory namespaces, definitions,
macros, arities, metadata, literal groups, dependencies, historical facts,
individual assertion clauses, setup, teardown, and source locations.

Pair path-matched tests, then discover additional tests by their `:refer`
metadata and dependencies. The authority has split emitter tests and auxiliary
preparation fixtures; a filename-only pairing would lose coverage. Give facts
stable identities using their source path, explicit ID where present, and
source location so duplicate `:refer` values cannot overwrite one another.

Build a dependency graph from implementation and test dependencies, preserving
cycles as migration units. Include compiler frontends, protocols, support
libraries, and the language models required to execute the selected tests.

For every source item record the proposed target owner, applicable rules,
disposition, and validation evidence. Separate changed behavior from unresolved
behavior. A reviewed missing item must not become a completed port.

Exit: every selected definition and assertion has an inventory record; the
first dependency-ordered slices and all external dependencies are visible.

### 3. Prove the original structural engine's guarantees

Add narrow, failure-detecting tests before changing engine behavior:

- Exact symbol and namespace-prefix matches with namespace boundaries.
- Alias resolution that respects local bindings and dependency options such
  as `:refer`, `:exclude`, and `:rename`.
- Structural replacement inside quoted, syntax-quoted, metadata, and discarded
  forms according to an explicit rule context.
- No accidental edits inside comments, strings, regex literals, unrelated
  keywords, or symbols sharing a textual prefix.
- Multiple edits in one container, overlapping matches, rule precedence, and
  deterministic application without stale paths after structural edits.
- Reader forms that need unsupported semantics produce located diagnostics
  and prevent partial publication.
- Parse/render round trips, no-op identity, and original-source restoration.

Use the archive's `nav/replace-at` and rule application model. Fix only the
engine gaps demonstrated by these tests.

Exit: exact before/after fixtures prove structural preservation and diagnostic
behavior on the runtime that will perform the migration.

### 4. Express the language migration as structural rules

Route base, core, runtime, and their tests through the shared transformation
entrypoint. Replace compiler-specific source-wide substitutions with rules
that match the actual block type and syntactic context.

Rules must cover:

- `tahto.base.*` and `tahto.common.*` to their established `lang.base.*` owners;
  `tahto.core.*` to `lang.core.*`; and `tahto.runtime.basic.*` to
  `lang.runtime.basic.*`, including root frontends and cross-stage references.
- Explicit per-symbol moves where target ownership differs. Do not infer a
  symbol's owner from its containing file alone.
- Namespace declarations, requires, aliases, metadata references, and declared
  semantic keyword routes. Ordinary string data is preserved unless a specific
  rule identifies it as an executable namespace representation.
- Prelude and builtin companion use, native constructors and protocols,
  structured exceptions, and source-backed collection arities.
- Clojure macro, record, publication, dynamic binding, and host interop forms
  actually encountered in the inventory. Quoted XTalk programs require their
  own context; a host-language rewrite must not rewrite a target-language call.
- Quoted runtime client programs, callable factories expressed as `def` plus
  closures, emitted dependency references, body transforms, and process
  configuration maps. Preserve their construction topology and metadata.
- Test syntax and fixture adaptation while preserving the original inputs,
  expected values, assertion count, fact IDs, and setup/teardown behavior.

Each rule needs an ID, match context, structural replacement, priority,
preconditions, source and target evidence, and positive and negative fixtures.
An unhandled host operation becomes a blocking diagnostic. Native adapters must
have explicit behavior tests; the generator must not compensate by weakening
the historical assertion.

Exit: one common structural migration path emits dry plans with rule-level
evidence for all priority families. Repeated manual target edits are unnecessary.

### 5. Complete a dependency-light pilot

Select the first small pair from the dependency inventory; `tahto.base.util`
is an initial candidate, subject to checking its closure. Migrate its complete
source and tests into an isolated validation project using canonical relative
paths.

Run the full native candidate/test/write cycle and compare the migrated tests
with the original assertions. Run scaffold preview and write immediately after
the source stabilizes, fill every test obligation, and demonstrate that changed
tests detect an incorrect implementation or expectation.

Regenerate from the same authority twice and verify identical output. Compare
that output with the current Hara owner and classify every difference.

Exit: a complete source/test pair is reproduced by rules alone, its behavioral
tests pass, correspondence is complete, and the second generation is clean.

### 6. Recreate the pipeline in dependency order

The inventory determines exact ordering and cycle groups. The planned slices
are:

| Slice | Source families | Required evidence |
| --- | --- | --- |
| Grammar and data | `tahto.common.book*`, base utilities, `grammar*` | Literal groups, merge topology, entries, modules, grammar and reserved lookup behavior |
| Preparation | `preprocess-base`, input, value, resolve, assignment, staging, provenance, and the preprocess publication surface | Exact stage outputs, macro bindings, symbols, fragments, dependency sets, and failures |
| Emission | `emit-helper`, common, data, assignment, functions, blocks, special forms, templates, top-level forms, rewrite, and emit entrypoints | Historical output strings, recursion and binding behavior, precedence, indentation, and stage order |
| Core compilation | Registry, library/snapshot, pointers, implementation/dependencies/lifecycle, rewrite modules, compile and links | Entry resolution, module order, imports, exports, lifecycle code, snapshots and restoration |
| Script integration | Script definition, macro, control, lint, annex, runtime and public core entrypoints, following their actual dependency cycles | Registration, macro expansion, module loading, compile/evaluate behavior, cache and restart behavior |
| Basic runtime | Common configuration, oneshot, twostep, verify, basic execution and the required server/process modules | Option precedence, setup, execution results, process errors, timeout behavior, teardown and restart |
| Required process ports | `tahto.runtime.basic.impl.process-js`, `.process-lua`, `.process-python`, their tests, quoted clients, wrappers, and shared XTalk dependencies | Original process contracts and emitter-generated bootstrap programs running under Node, Lua/LuaJIT, and CPython |

Use JavaScript for the first executable vertical slice, followed by Lua and
Python. All three are mandatory; they are not optional parity samples. Include
a compiled target where twostep behavior requires one. Inventory every
definition and program/context registration in the three process modules,
including alternative executables, Lua nginx variants, websocket clients, and
Python remote-port configuration. Stage these modes explicitly and retain
their tests and dispositions; passing the initial `:oneshot`, `:verify`, and
`:basic` slice does not establish a complete process-module port. Other runtime
directory families remain separately tracked obligations.

For each slice preserve every historical test, including split test files.
Move from fine-grained function checks to connected pipeline checks before
accepting the next slice.

Exit per slice: no unexplained source or assertion loss, passing relevant
historical behavior, explicit tested host adaptations, and clean regeneration.

### 6a. Required JavaScript, Lua, and Python bootstrap milestone

Port the three Foundation process modules and corresponding test families
through `tool.migrate` structural rules. Preserve the original form data and
fact assertions, and add behavioral checks where historical tests only assert
that generated output is a string.

The runtime source generation must retain the original composition:

```text
Installed language book + shared XTalk library entries
    -> impl/emit-entry-deps(return-eval, language, :layout :flat)
Quoted +client-basic+ forms
    -> impl/emit-as(language, forms)
Client invocation forms with host, port, and original options
    -> impl/emit-as(language, invocation)
    -> assembled bootstrap -> configured native process launch
```

Implement and validate this milestone in the following order:

1. Restore the source-backed `lang.core.impl/emit-as`,
   `emit-entry-deps-collect`, and `emit-entry-deps` behavior, together with
   their option resolution, dependency collection, entry emission, and tests.
   Required models and `xt.lang.common-lib/return-eval`, `return-wrap`, and
   `return-encode` must be registered and available before generating a client.
2. Port the quoted client forms, oneshot wrappers, body transforms, executable
   configuration, context registrations, and bootstrap factories. Connect
   `:bootstrap` to these factories. If existing `source-*` namespaces retain
   consumers, reconcile them as delegates to the source-owned factory and
   remove their independently authored target-language program bodies.
3. Resolve initialization ordering: install grammar/books and shared library
   entries, emit dependencies and client definitions, emit the invocation,
   then spawn the process. Bootstrap generation must work without an already
   running target process. Any cache must be tied to its language/library and
   options and have deterministic invalidation and an idempotent reset.
4. Reconcile the host bridge with the original emitted client's line protocol,
   JSON input, return envelope, and connection lifecycle. Current transport
   request objects and response profiles are explicit adaptation points; do
   not change target evaluation semantics to fit them. Necessary target-side
   adaptations must be declared forms that also pass through emission.
5. Launch the generated bootstrap for all three languages and run migrated
   runtime tests through the public script/evaluation path. Preserve original
   host/port options and body-transform behavior.

Language-specific acceptance evidence:

| Process | Required checks |
| --- | --- |
| JavaScript | `createRequire` and project module lookup, `NODE_PATH`, emitted oneshot imports, shared return helper dependencies, generated client invocation, and persistent evaluation |
| Lua | `cjson` and socket setup, original executable defaults/options, body normalization, inner function metadata, globalized dependency declarations, ping handling, and propagation of client-loop errors |
| Python | Original return/body transforms including assignment results and `OUT`, shared return helpers, byte buffering before UTF-8 decoding, UTF-8 response encoding, ping/EOF handling, and persistent evaluation |

For each language, verify the captured emitted dependency order, client forms,
invocation, and final bootstrap against the pinned reference. Test with fixed
host/port inputs for reproducible comparisons and allocated ports for actual
execution. Restore libraries, registries, caches, processes, and sockets on
every test exit.

Tests must also prove that emission owns the launched program: changing a
client form or a shared helper in an isolated test changes the generated
bootstrap and its observable behavior, and a missing dependency or emission
failure prevents startup with a located diagnostic. Restore the original form
and observe the same test pass. Execute exact value, structured error,
Unicode, repeated-request state, stop/restart, and startup-failure cases;
bootstrap generation returning a string is insufficient acceptance evidence.

Exit: all three process ports launch emitter-generated clients and pass their
required mode tests; dependency and source regeneration are deterministic;
every remaining mode or executable limitation has an explicit disposition.
No process-module completion claim is made while one of its required modes is
still unported.

### 7. Compare complete execution traces

Create a native Hara parity runner that invokes the pinned Foundation reference
and native Hara in isolated processes using the same inputs. It should compare:

1. Input and macro-expanded forms.
2. Resolved/staged forms, dependency sets, and provenance.
3. Canonical rewrite results and selected grammar entries.
4. Emitted source text and module assembly order.
5. Actual target execution values and structured failures.
6. State before execution, after execution, and after reset or teardown.

For the three required process ports, also record the bootstrap's input forms,
resolved helper entries, emitted source, and the source supplied to process
launch. The parity trace must demonstrate that the program executed is the
program produced by the restored emission pipeline.

Use an explicit observation schema for values crossing runtimes. Preserve
types, collection shape, and semantic metadata. Normalize only identified
incidental differences such as temporary paths or host stack details, retaining
raw results and the normalization rule. Do not normalize emitted text merely to
make it compare equal. Seed or inject nondeterministic capabilities where the
contract permits it; otherwise compare their documented behavioral properties.

The runner must identify the first differing stage and the responsible
source/test/rule evidence. Maintain fixtures where final outputs agree but an
intermediate stage is deliberately wrong to prove that stage comparisons run.
Classify reference failures separately; do not count a broken reference test as
a passing port.

Exit: representative programs and failure cases exercise the connected
Foundation and Hara pipelines with explained differences at every observation.

### 8. Reconcile and apply the reproducible result

Compare generated output against both the target baseline and existing local
edits. Incorporate required Hara adaptations into source-owned rules and rerun
generation before applying changes to the working tree.

Use the existing writer with validated target ownership, source and target
hash preconditions, duplicate-target checks, and a reversible transaction.
The Foundation authority remains read-only. Retain enough before-content to
restore overwritten targets; do not interpret an authority source flag as
permission to bypass target-path checks.

Update the migration record with input revisions, selected closure, per-item
dispositions, exact commands, assertion coverage, stage comparison results,
and remaining failures. Update owned package inventories from their generator
where necessary, and validate the rebuilt consuming artifact before making an
installed-runtime parity claim.

Exit: the canonical source/test tree can be recreated from pinned input and
rules, with no manual post-generation edits and no unexplained target changes.

## Definition of done

- Every definition, fact, and assertion in the agreed pipeline closure is
  accounted for. No silent omissions or placeholder substitutes remain.
- Untouched source spans survive structural migration exactly.
- All meaningful transformations have rule evidence and regression tests;
  changed tests are shown to detect failure.
- The migrated source and path-matched tests pass the fresh native validation
  cycle and correspondence checks.
- Connected stage and target execution checks prove the original contracts,
  with explicit tested host adaptations.
- JavaScript, Lua, and Python process ports use emitted shared dependencies,
  client forms, and invocation forms for their launched bootstraps. Required
  oneshot, verify, and basic execution tests pass for all three, and the
  remaining original process-module modes are accounted for explicitly.
- Generation is deterministic; canonicalization is idempotent; original
  sources and overwritten targets can be restored.
- Stateful tests restore the baseline on every exit path.
- Missing target executables, unsupported hosts, and remaining behavior gaps
  are reported explicitly and do not count as passing coverage.
- The existing public surface is reconciled deliberately; no convenience
  aliases or new semantics are introduced to hide an incomplete port.

The first implementation milestone is steps 1–5: a working structural engine,
a complete inventory, and one faithfully regenerated source/test pair. The
overall task is finished only after the agreed pipeline closure also satisfies
steps 6–8.
