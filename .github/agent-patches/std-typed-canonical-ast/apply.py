from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
PAYLOAD = Path(__file__).resolve().parent


def read(path: str) -> str:
    return (ROOT / path).read_text()


def write(path: str, content: str) -> None:
    (ROOT / path).write_text(content)


def payload(name: str) -> str:
    return (PAYLOAD / name).read_text()


def replace_between(path: str, start: str, end: str, replacement: str) -> None:
    text = read(path)
    start_at = text.find(start)
    if start_at < 0:
        raise RuntimeError(f"{path}: start marker not found: {start!r}")
    end_at = text.find(end, start_at)
    if end_at < 0:
        raise RuntimeError(f"{path}: end marker not found: {end!r}")
    write(path, text[:start_at] + replacement.rstrip() + text[end_at:])


def replace_exact(path: str, old: str, new: str, expected: int) -> None:
    text = read(path)
    count = text.count(old)
    if count != expected:
        raise RuntimeError(
            f"{path}: expected {expected} occurrences, found {count}: {old!r}"
        )
    write(path, text.replace(old, new))


def replace_once(path: str, old: str, new: str) -> None:
    replace_exact(path, old, new, 1)


write("core/lib/src/std/typed/schema.hal", payload("schema.hal"))
replace_once(
    "core/lib/src/std/typed/schema.hal",
    "  (normalize (deref schema)))",
    "  (normalize (Schema/ast schema)))",
)
write("core/lib/test/std/typed/schema_native_test.hal", payload("schema_native_test.hal"))
write("core/rust/tests/std_typed_schema.rs", payload("std_typed_schema.rs"))
write(
    "core/java/src/test/java/hara/truffle/StdTypedSchemaTest.java",
    payload("StdTypedSchemaTest.java"),
)
write(".github/workflows/std-typed-schema.yml", payload("std-typed-schema.yml"))

replace_between(
    "core/rust/src/core/protocol.rs",
    "fn schema_kind(schema: &crate::kernel::SchemaType) -> &'static str {",
    "\n\nfn compile_schema_value",
    payload("rust-schema-ast.rs"),
)
replace_between(
    "core/rust/src/kernel/schema.rs",
    "fn normalize_longhand(entries: &[(Form, Form)]) -> Result<SchemaType, String> {",
    "\n\n/// Infers conservative function signatures",
    payload("rust-normalize-longhand.rs"),
)
replace_between(
    "core/java/src/main/java/hara/truffle/HaraContext.java",
    "  private static String schemaKind(HalcSchema.Type ast) {",
    "\n\n  private Object hostCall",
    payload("java-schema-ast.txt"),
)
replace_once(
    "core/java/src/main/java/hara/truffle/HalcSchema.java",
    "import hara.lang.data.types.ILinearType;\n",
    "import hara.lang.data.types.ILinearType;\n"
    "import hara.lang.data.types.IMapType;\n",
)
replace_between(
    "core/java/src/main/java/hara/truffle/HalcSchema.java",
    "    if (schema instanceof hara.lang.data.types.IMapType<?, ?> map) {",
    "    if (schema instanceof hara.lang.data.List<?> reference",
    payload("java-normalize-map-branch.txt"),
)
replace_once(
    "core/java/src/main/java/hara/truffle/HalcSchema.java",
    "  /** Conservative body-derived function facts used by lowering tiers. */",
    payload("java-longhand-helpers.txt").rstrip()
    + "\n\n  /** Conservative body-derived function facts used by lowering tiers. */",
)
replace_once(
    "core/java/src/main/java/hara/truffle/HalcSchema.java",
    "if (schema instanceof Keyword keyword) return new Primitive(keyword.getName());",
    "if (schema instanceof Keyword keyword) return new Primitive(keywordName(keyword));",
)
replace_once(
    "core/java/src/main/java/hara/truffle/HalcSchema.java",
    "    List<Object> arguments = values(vector, 1);\n"
    "    return switch (head.getName()) {",
    "    List<Object> arguments = values(vector, 1);\n"
    "    String headName = keywordName(head);\n"
    "    return switch (headName) {",
)
replace_exact(
    "core/java/src/main/java/hara/truffle/HalcSchema.java",
    "requireCount(head.getName(), arguments, 1);",
    "requireCount(headName, arguments, 1);",
    2,
)
replace_once(
    "core/java/src/main/java/hara/truffle/HalcSchema.java",
    "      default -> arguments.isEmpty()\n"
    "          ? new Primitive(head.getName())\n"
    "          : new Extension(head.getName(), arguments);",
    "      default -> arguments.isEmpty()\n"
    "          ? new Primitive(headName)\n"
    "          : new Extension(headName, arguments);",
)

replace_once(
    "core/spec/std/foundation-base-runtime-schema.md",
    "- canonical longhand data such as `{:kind :map :children [...]}`;",
    "- canonical normalized data such as `{:kind :map :fields [...]}`;\n"
    "- retained longhand input such as `{:kind :map :children [...]}`, which is\n"
    "  immediately converted to the canonical normalized form;",
)
replace_once(
    "core/spec/std/foundation-base-runtime-schema.md",
    "`Schema/kind`, `Schema/form`, `Schema/ast`, and `Schema/origin` inspect schema\n"
    "values; `Schema/instance?` recognizes them. `(type (schema value))` is\n"
    "`:std.native.SchemaType`. Printing is round-trippable as\n"
    "`(schema <canonical-short-form>)`.",
    "`Schema/kind`, `Schema/form`, `Schema/ast`, and `Schema/origin` inspect schema\n"
    "values; `Schema/instance?` recognizes them. `(type (schema value))` is\n"
    "`:std.native.SchemaType`. Printing is round-trippable as\n"
    "`(schema <canonical-short-form>)`. `Schema/ast` returns the portable\n"
    "normalized map rather than a host compiler-node shape. For every valid\n"
    "surface schema, portable normalization, native AST inspection, and\n"
    "re-normalization are structurally equal, and `(schema (Schema/ast value))`\n"
    "reconstructs a `SchemaType` with the same canonical AST. `Schema/form` and\n"
    "`Schema/origin` continue to preserve the inspected value's source form and\n"
    "origin.",
)
