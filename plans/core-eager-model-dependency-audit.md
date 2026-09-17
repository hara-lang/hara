# Original core eager-model dependencies — 2026-09-15

Authority: Foundation fe54cc866473a888bfbd4ce2d3de6d8011f09f72,
src/tahto/core.clj and the model namespace declarations at that revision.
The full migration remains incomplete.

The original core facade eagerly requires spec-xtalk, spec-bash, spec-c,
spec-dart, spec-glsl, spec-js, spec-lua, spec-elisp, spec-scheme, spec-python,
spec-sql, and sql.spec-oracle. It does not eagerly require spec-hara.
The original facade therefore has required model work beyond the currently
installed JS/Lua/Python/C/XTalk roots and staged Dart replacement.

Initial source search found no installed model owners for Bash, GLSL, Elisp,
Scheme, SQL, or Oracle. No corresponding migration candidate records were
found by filename search; this is not proof that no partial rule exists.

Dependency order from the pinned namespace declarations:

- Bash: original Book, base emit/common/data/fn/top-level/grammar/util,
  script, and XTalk. Audit its literal template and quoting idioms before porting.
- GLSL: original Book, emit/grammar/util, script, collection and template
  adaptation. This is another branch that does not require the full core facade
  in its source namespace.
- Elisp and Scheme: shared spec-lisp-common plus fn-elisp/fn-scheme XTalk
  helpers, then their model roots. Do not start from the roots alone.
- SQL and Oracle: sql.spec-common.common and form-defenum/form-defn/form-deftype,
  then sql.spec-common, then spec-sql/sql.spec-oracle. Do not infer those roots
  are trivial migrations merely because their source files are short.

The previously staged full Dart root test fixture requires core and therefore
still meets the old facade's missing runtime/bootstrap publication. The original
facade additionally needs compiler, management, and notification publications.
Do not restore bootstrap or omit original eager model requires to bypass this.

Native-only spec-hara and type-hara still consume lang.base.compiler; other
legacy roots consume the catalog registry. Those consumers matter for eventual
grammar-api removal and preserving native work, but the pinned facade's direct
requires do not justify treating native spec-hara as its eager dependency.

No source, tests, gate, or runtime semantics were changed during this audit.
Next safe branches are the Bash/GLSL source-test inventories or the dependency-
light shared Lisp/SQL helpers, followed by their roots and only then the facade.

Update: GLSL is now installed with both historical assertions and eight native
contracts passing. Its saved migration record regenerates both files byte-exactly.
See glsl-original-migration-inventory.md for validation and its completed catalog
registration. The original script owner still needs catalog registration;
Bash, the Lisp branches, SQL and Oracle remain outstanding.
