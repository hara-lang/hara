#!/usr/bin/env python3
"""Apply the guarded runtime-closeout patch in the isolated validation branch."""

from pathlib import Path


def replace_once(path: Path, old: str, new: str, label: str) -> None:
    source = path.read_text()
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"{label} matched {count} times, expected exactly once")
    path.write_text(source.replace(old, new, 1))


bundle_path = Path("core/rust/src/vm/bundle.rs")
replace_once(
    bundle_path,
    """\
            if source.is_none() && standard_library_namespace(&module.resource) {
                continue;
            }
""",
    "",
    "source-free loader guard",
)
replace_once(
    bundle_path,
    """\
            let mut runtime = Runtime::core();
            for &(name, _, source) in EMBEDDED_HAL_RESOURCES {
                runtime.register_resource(name, source);
            }
            eval_bytecode_bundle(&mut runtime, &bytes).expect("load foundation bundle");
""",
    """\
            let mut runtime = Runtime::core();
            assert!(runtime.resources.is_empty());
            eval_bytecode_bundle(&mut runtime, &bytes)
                .expect("load source-free foundation bundle");
""",
    "source-free Foundation regression",
)
replace_once(
    bundle_path,
    """\
    #[test]
    fn eager_failure_rolls_back_the_whole_bundle() {
""",
    """\
    #[test]
    fn stale_eager_bytecode_is_rejected_when_registered_source_differs() {
        let mut compiler = Runtime::core();
        compiler.use_namespace("example.eager");
        let artifact = compiler
            .compile_bytecode_artifact("(def answer 41)")
            .expect("compile eager fixture");
        let bytes = encode_bytecode_bundle(&[BytecodeBundleModule {
            resource: "example.eager".into(),
            namespace_form: "(ns example.eager)".into(),
            source_digest: Sha256::digest(
                b"(ns example.eager) (def answer 41)",
            )
            .into(),
            dependencies: vec![],
            eager: true,
            artifact,
        }])
        .expect("encode eager fixture");
        let mut runtime = Runtime::core();
        runtime.register_resource(
            "example.eager",
            "(ns example.eager) (def answer 42)",
        );

        let error = eval_bytecode_bundle(&mut runtime, &bytes).unwrap_err();

        assert_eq!(
            error,
            "stale eager bytecode bundle module: example.eager"
        );
        assert!(!runtime.bytecode_resources.contains_key("example.eager"));
        assert!(!runtime.loaded_resources.contains("example.eager"));
        assert!(runtime.namespace_registry.find("example.eager").is_none());
    }

    #[test]
    fn eager_failure_rolls_back_the_whole_bundle() {
""",
    "stale eager regression insertion point",
)

functions_path = Path("core/rust/src/vm/compiler/functions.rs")
replace_once(
    functions_path,
    """\
                        "quote" => {}
                        "." => {
""",
    """\
                        "quote" => {}
                        "set!" => {
                            if let Some(place) = children.get(1) {
                                match place.form {
                                    Form::List(forms)
                                        if matches!(
                                            forms.first(),
                                            Some(Form::Symbol(operation))
                                                if operation == "field"
                                        ) =>
                                    {
                                        let place_children = self.list_children(
                                            forms,
                                            place.span,
                                            place.children,
                                        );
                                        if let Some(receiver) = place_children.get(1) {
                                            self.collect_free(receiver, bound, free);
                                        }
                                    }
                                    Form::Symbol(_) => {}
                                    _ => self.collect_free(place, bound, free),
                                }
                            }
                            if let Some(value) = children.get(2) {
                                self.collect_free(value, bound, free);
                            }
                        }
                        "field" => {
                            if let Some(instance) = children.get(1) {
                                self.collect_free(instance, bound, free);
                            }
                        }
                        "." => {
""",
    "closure special-form insertion point",
)

execution_path = Path("core/rust/src/vm/execution_tests.rs")
replace_once(
    execution_path,
    """\
    assert_eq!(
        eval("(do (defmutable Cursor [x]) (instance? Cursor (Cursor 1)))"),
        "true"
    );
""",
    """\
    assert_eq!(
        eval("(do (defmutable Cursor [x]) (defn move! [cursor value] (set! (field cursor :x) value)) (defn position [cursor] (field cursor :x)) (let [cursor (Cursor 1)] [(move! cursor 10) (position cursor)]))"),
        "[10 10]"
    );
    assert_eq!(
        eval("(do (defmutable Cursor [x]) (instance? Cursor (Cursor 1)))"),
        "true"
    );
""",
    "mutable closure regression insertion point",
)

# Revert the source rewrite used to diagnose the compiler failure. The generic
# closure analysis fix above is the canonical repair.
dom_path = Path("core/lib/src/std/dom/common.hal")
replace_once(
    dom_path,
    """\
(defn dom-field-set!
  [dom key value]
  (cond
    (= key :tag) (set! (field dom :tag) value)
    (= key :props) (set! (field dom :props) value)
    (= key :item) (set! (field dom :item) value)
    (= key :parent) (set! (field dom :parent) value)
    (= key :handler) (set! (field dom :handler) value)
    (= key :shadow) (set! (field dom :shadow) value)
    (= key :cache) (set! (field dom :cache) value)
    (= key :extra) (set! (field dom :extra) value)
    :else (throw (ex-info "Unknown DOM field" {:field key})))
  dom)
""",
    """\
(defn dom-field-set!
  [dom key value]
  (case key
    :tag (set! (field dom :tag) value)
    :props (set! (field dom :props) value)
    :item (set! (field dom :item) value)
    :parent (set! (field dom :parent) value)
    :handler (set! (field dom :handler) value)
    :shadow (set! (field dom :shadow) value)
    :cache (set! (field dom :cache) value)
    :extra (set! (field dom :extra) value)
    (throw (ex-info "Unknown DOM field" {:field key})))
  dom)
""",
    "exploratory std.dom rewrite",
)

template_path = Path("core/lib/src/tool/cli/template.hal")
replace_once(
    template_path,
    '(def schema-id "tool.cli.template/0-alpha")\n\n',
    """\
(def schema-id "tool.cli.template/0-alpha")

;; Template handlers are live runtime functions. Keep their provenance in a
;; runtime-owned identity registry rather than attaching functions and work
;; graphs to portable metadata.
(def +handlers+ (atom []))

""",
    "CLI handler registry insertion point",
)
replace_once(
    template_path,
    """\
(defn handler
  "Returns the closed-registry function [request] -> tool.cli.model/result."
  [raw-definition]
  (let [definition (normalise-definition raw-definition)
        command-work (work definition)
        runtime-fn (:runtime-fn definition)]
    (with-meta
      (fn [request]
        (std-work/run (runtime-fn) command-work request))
      {:tool.cli.template/schema schema-id
       :tool.cli.template/definition definition
       :tool.cli.template/work command-work})))

(defn handler?
  [value]
  (= schema-id
     (:tool.cli.template/schema (meta value))))
""",
    """\
(defn handler
  "Returns the closed-registry function [request] -> tool.cli.model/result."
  [raw-definition]
  (let [definition (normalise-definition raw-definition)
        command-work (work definition)
        runtime-fn (:runtime-fn definition)
        handler-fn
        (fn [request]
          (std-work/run (runtime-fn) command-work request))]
    (swap! +handlers+ conj handler-fn)
    handler-fn))

(defn handler?
  [value]
  (and
   (fn? value)
   (boolean
    (some
     (fn [handler-fn]
       (= value handler-fn))
     (deref +handlers+)))))
""",
    "function-metadata CLI handler",
)

template_test_path = Path("core/lib/test/tool/cli/template_test.hal")
registry_fact = """\
(fact "every closed-registry handler is template-produced"
  (every?
   template/handler?
   (vals (handlers/registry)))
  => true)
"""
replace_once(
    template_test_path,
    registry_fact,
    registry_fact
    + """\

(fact "ordinary functions are not mistaken for template handlers"
  (template/handler? (fn [request] request))
  => false)
""",
    "CLI template identity regression insertion point",
)
