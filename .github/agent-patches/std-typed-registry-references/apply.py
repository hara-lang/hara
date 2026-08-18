from __future__ import annotations

import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
PAYLOAD = Path(__file__).resolve().parent


def read(path: str) -> str:
    return (ROOT / path).read_text()


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content)


def payload(name: str) -> str:
    return (PAYLOAD / name).read_text()


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one marker, found {count}: {old!r}")
    return text.replace(old, new, 1)


def registry_source() -> str:
    return replace_once(
        payload("registry.hal"),
        "(declare find-schema)",
        "(declare find-schema names)",
        "registry forward declarations",
    )


def schema_source() -> str:
    text = read("core/lib/src/std/typed/schema.hal")
    text = replace_once(
        text,
        "(ns std.typed.schema)\n",
        "(ns std.typed.schema\n"
        "  (:require [std.typed.registry :as registry]))\n\n"
        "(def ^{:dynamic true} *registry* nil)\n"
        "(def ^{:dynamic true} *reference-trail* #{})\n",
        "schema namespace",
    )
    text = replace_once(
        text,
        "    (and (list? schema)\n"
        "         (= 2 (count schema))\n"
        "         (= 'var (first schema))\n"
        "         (symbol? (second schema)))\n"
        "    :reference\n\n"
        "    (not (vector? schema))",
        "    (and (list? schema)\n"
        "         (= 2 (count schema))\n"
        "         (= 'var (first schema))\n"
        "         (symbol? (second schema)))\n"
        "    :reference\n\n"
        "    (and *registry* (symbol? schema))\n"
        "    :registry-reference\n\n"
        "    (not (vector? schema))",
        "schema symbol reference dispatch",
    )
    text = replace_once(
        text,
        "(defmethod normalize :reference [schema]\n"
        "  {:kind :reference :name (second schema)})",
        "(defn- registry-reference-name\n"
        "  [target]\n"
        "  (if *registry*\n"
        "    (registry/qualify *registry* target)\n"
        "    target))\n\n"
        "(defmethod normalize :reference [schema]\n"
        "  {:kind :reference\n"
        "   :name (registry-reference-name (second schema))})\n\n"
        "(defmethod normalize :registry-reference [schema]\n"
        "  {:kind :reference\n"
        "   :name (registry-reference-name schema)})\n\n"
        "(defn ^{:schema [:fn [:any :any] :map]}\n"
        "  normalize-with\n"
        "  \"Normalizes schema using an explicit portable schema registry.\"\n"
        "  [schema registry-value]\n"
        "  (binding [*registry* (registry/ensure registry-value)]\n"
        "    (normalize schema)))",
        "schema reference normalization",
    )
    text = replace_once(
        text,
        "(defmethod normalize :default [schema]\n"
        "  {:kind :extension\n"
        "   :head (first schema)\n"
        "   :arguments (vec (rest schema))\n"
        "   :surface schema})\n\n"
        "(defn- primitive-valid?",
        "(defmethod normalize :default [schema]\n"
        "  {:kind :extension\n"
        "   :head (first schema)\n"
        "   :arguments (vec (rest schema))\n"
        "   :surface schema})\n\n"
        + payload("schema_registry_api.hal").rstrip()
        + "\n\n(defn- primitive-valid?",
        "schema registry API insertion",
    )
    text = replace_once(
        text,
        "(defmethod validate-normal :default [schema value path]\n"
        "  [])",
        payload("schema_reference_validation.hal").rstrip()
        + "\n\n(defmethod validate-normal :default [schema value path]\n"
        "  [])",
        "schema reference validation insertion",
    )
    text = replace_once(
        text,
        "(defn ^{:schema [:fn [:any :any] :vector]}\n"
        "  validate\n"
        "  \"Returns deterministic, path-aware findings for value against schema.\"\n"
        "  [schema value]\n"
        "  (validate-normal (normalize schema) value []))",
        "(defn ^{:schema [:function\n"
        "                 [:fn [:any :any] :vector]\n"
        "                 [:fn [:any :any :any] :vector]]}\n"
        "  validate\n"
        "  \"Returns deterministic, path-aware findings for value against schema.\"\n"
        "  ([schema value]\n"
        "   (binding [*reference-trail* #{}]\n"
        "     (validate-normal (normalize schema) value [])))\n"
        "  ([schema value registry-value]\n"
        "   (binding [*registry* (registry/ensure registry-value)\n"
        "             *reference-trail* #{}]\n"
        "     (validate-normal (normalize schema) value []))))",
        "schema validate arities",
    )
    text = replace_once(
        text,
        "(defn ^{:schema [:fn [:any :any] :bool]}\n"
        "  valid?\n"
        "  \"Returns true when value conforms to schema.\"\n"
        "  [schema value]\n"
        "  (empty? (validate schema value)))",
        "(defn ^{:schema [:function\n"
        "                 [:fn [:any :any] :bool]\n"
        "                 [:fn [:any :any :any] :bool]]}\n"
        "  valid?\n"
        "  \"Returns true when value conforms to schema.\"\n"
        "  ([schema value]\n"
        "   (empty? (validate schema value)))\n"
        "  ([schema value registry-value]\n"
        "   (empty? (validate schema value registry-value))))",
        "schema valid arities",
    )
    text = replace_once(
        text,
        "(defn ^{:schema [:fn [:any :any] :bool]}\n"
        "  compatible?\n"
        "  \"Returns true when two surface schemas have an overlapping value domain.\"\n"
        "  [expected actual]\n"
        "  (compatible-normal? (normalize expected) (normalize actual)))",
        "(defn ^{:schema [:function\n"
        "                 [:fn [:any :any] :bool]\n"
        "                 [:fn [:any :any :any] :bool]]}\n"
        "  compatible?\n"
        "  \"Returns true when two schemas have an overlapping value domain.\"\n"
        "  ([expected actual]\n"
        "   (compatible-normal? (normalize expected) (normalize actual)))\n"
        "  ([expected actual registry-value]\n"
        "   (compatible-normal?\n"
        "    (resolve-recursive expected registry-value)\n"
        "    (resolve-recursive actual registry-value))))",
        "schema compatible arities",
    )
    text = replace_once(
        text,
        "(defn ^{:schema [:fn [:any] :vector]}\n"
        "  project-arities\n"
        "  \"Returns normalized function arities from :fn or :function schema.\"\n"
        "  [schema]\n"
        "  (let [normalized (normalize schema)]\n"
        "    (cond\n"
        "      (= :fn (:kind normalized)) [normalized]\n"
        "      (= :function (:kind normalized)) (:arities normalized)\n"
        "      :else [])))",
        "(defn ^{:schema [:function\n"
        "                 [:fn [:any] :vector]\n"
        "                 [:fn [:any :any] :vector]]}\n"
        "  project-arities\n"
        "  \"Returns normalized function arities from :fn or :function schema.\"\n"
        "  ([schema]\n"
        "   (let [normalized (normalize schema)]\n"
        "     (cond\n"
        "       (= :fn (:kind normalized)) [normalized]\n"
        "       (= :function (:kind normalized)) (:arities normalized)\n"
        "       :else [])))\n"
        "  ([schema registry-value]\n"
        "   (let [normalized (resolve-recursive schema registry-value)]\n"
        "     (cond\n"
        "       (= :fn (:kind normalized)) [normalized]\n"
        "       (= :function (:kind normalized)) (:arities normalized)\n"
        "       :else []))))",
        "schema project arities",
    )
    return text


def append_before_final_brace(text: str, addition: str, label: str) -> str:
    marker = "\n}\n"
    index = text.rfind(marker)
    if index < 0:
        raise RuntimeError(f"{label}: final class brace not found")
    return text[:index] + addition.rstrip() + text[index:]


def patch_inventory(text: str) -> str:
    return replace_once(
        text,
        "std.typed.infer\nstd.typed.schema\n",
        "std.typed.infer\nstd.typed.registry\nstd.typed.schema\n",
        "standard library typed inventory",
    )


def patch_bootstrap(text: str) -> str:
    return replace_once(
        text,
        "std.typed.schema\nstd.typed.infer\n",
        "std.typed.registry\nstd.typed.schema\nstd.typed.infer\n",
        "bootstrap typed inventory",
    )


def patch_audit(text: str) -> str:
    text = replace_once(
        text,
        "  'core/lib/src/std/typed/schema.hal'\n"
        "  'core/lib/src/std/typed/infer.hal'",
        "  'core/lib/src/std/typed/registry.hal'\n"
        "  'core/lib/src/std/typed/schema.hal'\n"
        "  'core/lib/src/std/typed/infer.hal'",
        "audit required typed paths",
    )
    text = replace_once(
        text,
        "for namespace in std.typed.schema std.typed.infer; do",
        "for namespace in std.typed.registry std.typed.schema std.typed.infer; do",
        "audit standard typed inventory",
    )
    text = replace_once(
        text,
        "if ! grep -Fxq 'std.typed.schema' core/rust/bootstrap.namespaces; then\n"
        "  echo 'The portable schema core is missing from the embedded runtime catalog.' >&2\n"
        "  failed=1\n"
        "fi",
        "for namespace in std.typed.registry std.typed.schema; do\n"
        "  if ! grep -Fxq \"$namespace\" core/rust/bootstrap.namespaces; then\n"
        "    echo \"Portable schema bootstrap namespace is missing: $namespace\" >&2\n"
        "    failed=1\n"
        "  fi\n"
        "done\n"
        "registry_line=$(grep -n -F 'std.typed.registry' core/rust/bootstrap.namespaces | cut -d: -f1)\n"
        "schema_line=$(grep -n -F 'std.typed.schema' core/rust/bootstrap.namespaces | cut -d: -f1)\n"
        "if [[ -n \"$registry_line\" && -n \"$schema_line\" && \"$registry_line\" -ge \"$schema_line\" ]]; then\n"
        "  echo 'std.typed.registry must bootstrap before std.typed.schema.' >&2\n"
        "  failed=1\n"
        "fi",
        "audit typed bootstrap inventory",
    )
    return text


def patch_workflow(text: str) -> str:
    needle = "      - 'core/lib/test/std/typed/**'\n"
    replacement = (
        needle
        + "      - 'core/rust/bootstrap.namespaces'\n"
        + "      - 'core/rust/standard-library.namespaces'\n"
        + "      - 'scripts/audit-std-typed-boundary.sh'\n"
    )
    if text.count(needle) != 2:
        raise RuntimeError("typed workflow: expected two path-filter insertion points")
    text = text.replace(needle, replacement)
    java_job = "  java:\n    runs-on: ubuntu-latest\n"
    hal_job = (
        "  hal:\n"
        "    runs-on: ubuntu-latest\n"
        "    steps:\n"
        "      - uses: actions/checkout@v4\n"
        "      - uses: actions/setup-java@v5\n"
        "        with:\n"
        "          java-version: '21'\n"
        "          distribution: temurin\n"
        "          cache: maven\n"
        "          cache-dependency-path: core/java/pom.xml\n"
        "      - name: Build the native Hara runtime\n"
        "        run: mvn -B -Ptruffle -DskipTests package --file core/java/pom.xml\n"
        "      - name: Run focused portable registry tests\n"
        "        run: core/hara --offline test-check core/lib/test/std/typed/registry_test.hal\n\n"
    )
    return replace_once(text, java_job, hal_job + java_job, "typed workflow HAL job")


def apply_product() -> None:
    write("core/lib/src/std/typed/registry.hal", registry_source())
    write("core/lib/src/std/typed/schema.hal", schema_source())
    write("core/lib/test/std/typed/registry_test.hal", payload("registry_test.hal"))

    rust_test = read("core/rust/tests/std_typed_schema.rs")
    if "portable_schema_registry_resolves_recursive_references" in rust_test:
        raise RuntimeError("Rust registry parity test already exists")
    write("core/rust/tests/std_typed_schema.rs", rust_test.rstrip() + payload("rust_test_append.rs"))

    java_test = read("core/java/src/test/java/hara/truffle/StdTypedSchemaTest.java")
    if "portableSchemaRegistryResolvesRecursiveReferences" in java_test:
        raise RuntimeError("Truffle registry parity test already exists")
    write(
        "core/java/src/test/java/hara/truffle/StdTypedSchemaTest.java",
        append_before_final_brace(java_test, payload("java_test_method.txt"), "Truffle test"),
    )

    write(
        "core/rust/standard-library.namespaces",
        patch_inventory(read("core/rust/standard-library.namespaces")),
    )
    write(
        "core/rust/bootstrap.namespaces",
        patch_bootstrap(read("core/rust/bootstrap.namespaces")),
    )
    write(
        "scripts/audit-std-typed-boundary.sh",
        patch_audit(read("scripts/audit-std-typed-boundary.sh")),
    )
    write(
        ".github/workflows/std-typed-schema.yml",
        patch_workflow(read(".github/workflows/std-typed-schema.yml")),
    )
    spec_path = "core/spec/std/foundation-base-runtime-schema.md"
    spec = read(spec_path)
    if "## Portable schema registries and named references" in spec:
        raise RuntimeError("registry specification section already exists")
    write(spec_path, spec.rstrip() + "\n" + payload("spec_section.md"))


def write_candidates(directory: Path) -> None:
    directory.mkdir(parents=True, exist_ok=True)
    (directory / "registry.hal").write_text(registry_source())
    (directory / "schema.hal").write_text(schema_source())
    (directory / "probe.hal").write_text(payload("candidate_probe.hal"))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--candidate-dir", type=Path)
    args = parser.parse_args()
    if args.candidate_dir:
        write_candidates(args.candidate_dir)
    else:
        apply_product()


if __name__ == "__main__":
    main()
