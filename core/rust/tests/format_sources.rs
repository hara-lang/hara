use hara_wasm::SessionKernel;
use std::fs;
use std::path::{Path, PathBuf};

type Resource = (&'static str, &'static str);

const STRING: Resource = (
    "std.foundation.string",
    "lib/src/std/foundation/string.hal",
);
const COMMON: Resource = ("std.format.common", "lib/src/std/format/common.hal");
const TABLE: Resource = ("std.format.table", "lib/src/std/format/table.hal");
const REPORT: Resource = ("std.format.report", "lib/src/std/format/report.hal");
const TERMINAL: Resource = (
    "std.format.terminal",
    "lib/src/std/format/terminal.hal",
);
const FORMAT: Resource = ("std.format", "lib/src/std/format.hal");
const WORK_REPORT: Resource = ("std.work.report", "lib/src/std/work/report.hal");

fn core_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("core/rust has a core parent")
        .to_path_buf()
}

fn read_source(path: &str) -> String {
    fs::read_to_string(core_root().join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn evaluate(resources: &[Resource], prelude: &str, tail: &str) -> Result<String, String> {
    let mut kernel = SessionKernel::new();
    for &(namespace, path) in resources {
        kernel.register_resource(namespace, &read_source(path));
    }
    kernel.eval("ROOT", &format!("{prelude}\n{tail}"))
}

fn assert_source(resources: &[Resource], namespace: &str) {
    eprintln!("=== source-candidate:{namespace} ===");
    let prelude = format!("(require [{namespace} :as candidate])");
    evaluate(resources, &prelude, "nil")
        .unwrap_or_else(|error| panic!("candidate {namespace} failed: {error}"));
}

fn assert_string_case(name: &str, tail: &str, expected: &str) {
    eprintln!("=== string-case:{name} ===");
    let output = evaluate(
        &[STRING],
        "(require [std.foundation.string :as str])",
        tail,
    )
    .unwrap_or_else(|error| panic!("string case {name} failed: {error}"));
    assert_eq!(output, expected, "string case {name}");
}

#[test]
fn every_changed_hal_source_evaluates_in_a_fresh_native_runtime() {
    assert_source(&[STRING], STRING.0);
    assert_source(&[COMMON], COMMON.0);
    assert_source(&[COMMON, TABLE], TABLE.0);
    assert_source(&[COMMON, TABLE, REPORT], REPORT.0);
    assert_source(&[TERMINAL], TERMINAL.0);
    assert_source(&[COMMON, TABLE, REPORT, TERMINAL, FORMAT], FORMAT.0);
    assert_source(
        &[COMMON, TABLE, REPORT, TERMINAL, WORK_REPORT],
        WORK_REPORT.0,
    );
}

#[test]
fn portable_string_contract_is_exercised_inside_hal() {
    let cases = [
        (
            "tag-keywords",
            r#"(= (str/tag :a "/" :ns/b) "a/ns/b")"#,
            "true",
        ),
        (
            "tag-bytes",
            r#"(= (str/tag (str/encode-utf8 "hé")) "hé")"#,
            "true",
        ),
        ("tag-symbol", r#"(= (str/tag 'hello) "hello")"#, "true"),
        ("tag-nil", r#"(= (str/tag nil) "nil")"#, "true"),
        ("blank-nil", "(str/blank? nil)", "true"),
        (
            "trim-newlines",
            r#"(= (str/trim-newlines "hello\r\n") "hello")"#,
            "true",
        ),
        ("caseless", r#"(str/caseless= "heLLo" "HellO")"#, "true"),
        (
            "replace-at",
            r#"(= (str/replace-at "hλra" 1 "a") "hara")"#,
            "true",
        ),
        (
            "insert-at",
            r#"(= (str/insert-at "hλra" 2 "!") "hλ!ra")"#,
            "true",
        ),
        (
            "camel-case",
            r#"(= (str/camel-case "hello__--  world") "helloWorld")"#,
            "true",
        ),
        (
            "pascal-case",
            r#"(= (str/pascal-case "hello_world") "HelloWorld")"#,
            "true",
        ),
        (
            "snake-case",
            r#"(= (str/snake-case "version2Value") "version2_value")"#,
            "true",
        ),
        (
            "spear-case",
            r#"(= (str/spear-case "hello_world") "hello-world")"#,
            "true",
        ),
        (
            "dot-case",
            r#"(= (str/dot-case "hello-world") "hello.world")"#,
            "true",
        ),
        (
            "pipe-is-ordinary",
            r#"(= (str/snake-case "hello|World") "hello|world")"#,
            "true",
        ),
        (
            "upper-case",
            r#"(= (str/upper-case "hara") "HARA")"#,
            "true",
        ),
        (
            "lower-case",
            r#"(= (str/lower-case "HARA") "hara")"#,
            "true",
        ),
        (
            "capital-case",
            r#"(= (str/capital-case "hELLO") "Hello")"#,
            "true",
        ),
        (
            "joinl-absent",
            "(nil? (resolve 'std.foundation.string/joinl))",
            "true",
        ),
        (
            "pipe-helper-absent",
            r#"(nil? (resolve (symbol "std.foundation.string/|")))"#,
            "true",
        ),
        (
            "truncate-absent",
            "(nil? (resolve 'std.foundation.string/truncate))",
            "true",
        ),
    ];

    for (name, tail, expected) in cases {
        assert_string_case(name, tail, expected);
    }
}

#[test]
fn layered_formatting_and_work_event_projection_are_exact() {
    let output = evaluate(
        &[STRING, COMMON, TABLE, REPORT, TERMINAL, FORMAT, WORK_REPORT],
        r#"
(require [std.foundation.string :as str])
(require [std.format :as format])
(require [std.format.report :as format-report])
(require [std.work.report :as report])
(require [std.work.protocol :as protocol])
"#,
        r#"
(def item-a
  {:item/id :alpha
   :item/index 0
   :item/total 2
   :status :return
   :item/display "changed"
   :elapsed 3})

(def item-b
  {:item/id :beta
   :item/index 1
   :item/total 2
   :status :warn
   :data "review"
   :elapsed 4})

(def columns
  [{:key :path :label "PATH" :width 8}
   {:key :functions :label "FUN" :width 3 :align :right}
   {:key :inserts :label "INS" :width 3 :align :right}])

(def rows
  [{:path "a.clj" :functions 2 :inserts 3 :status :return}
   {:path "b.clj" :functions 1 :inserts 0 :status :return}])

(def summary
  {:items 2
   :results 2
   :warnings 1
   :errors 0
   :cumulative 7
   :elapsed 8})

(def document
  (report/complete
   (report/add-section
    (report/add-section
     (report/add-section
      (report/add-section
       (report/document
        :example/transform
        {:title "TRANSFORM CODE"
         :status :warning
         :annotations ["Done"]
         :profile
         {:sections [:items :warnings :results :summary]}})
       (report/section
        :items :progress [item-a item-b]
        {:section/title "ITEMS"}))
      (report/section
       :warnings :diagnostics [item-b]
       {:section/title "WARNINGS"}))
     (report/section
      :results :table rows
      {:section/title "RESULTS"
       :section/columns columns}))
    (report/section
     :summary :summary [summary]
     {:section/title "SUMMARY"
      :section/fields
      [:items :results :warnings :errors :cumulative :elapsed]}))
   :warning))

(def expected
  (str/join
   "\n"
   ["TRANSFORM CODE"
    ""
    "ITEMS"
    "1/2  :alpha  ok  changed  3ms"
    "2/2  :beta  warn  review  4ms"
    ""
    "WARNINGS"
    ":beta  warn  review"
    ""
    "RESULTS"
    "PATH      FUN  INS"
    "--------  ---  ---"
    "a.clj       2    3"
    "b.clj       1    0"
    ""
    "SUMMARY"
    "Items: 2"
    "Results: 2"
    "Warnings: 1"
    "Errors: 0"
    "Cumulative: 7ms"
    "Elapsed: 8ms"
    ""
    "Done"]))

(def events
  [{:event :task/run-started
    :report/title "TRANSFORM CODE"
    :report/profile (:report/profile document)}
   {:event :task/item-completed :record item-a}
   {:event :task/item-completed :record item-b}
   {:event :task/run-completed :report document}])

(def emitted-output (atom []))
(def emitted-count
  (format/emit-lines!
   [(format/line "first" :success)
    (format/line "second" :warning)]
   {:emit
    (fn [text]
      (swap! emitted-output conj text))}))

(def live-output (atom []))
(def live-observer
  (report/observer
   {:ansi false
    :emit
    (fn [text]
      (swap! live-output conj text))}))

(def _live-events
  (reduce
   (fn [count event]
     (protocol/work-event live-observer event)
     (inc count))
   0
   events))

(def replay-output (atom []))
(def replay-count
  (report/replay!
   events
   {:ansi false
    :emit
    (fn [text]
      (swap! replay-output conj text))}))

(def ansi-title
  (first
   (format/render-lines
    (format/report-lines document)
    {:ansi true})))

[(= (format/report document) expected)
 (= (report/render-events events) expected)
 (= emitted-count 2)
 (= (deref emitted-output) ["first" "second"])
 (str/starts-with? ansi-title "\u001b[1;36m")
 (str/ends-with? ansi-title "\u001b[0m")
 (= (str/join "\n" (deref live-output)) expected)
 (= (str/join "\n" (deref replay-output)) expected)
 (= replay-count (count (deref replay-output)))
 (= (format-report/report
     document
     {:include-sections [:summary]
      :include-title false
      :include-annotations false})
    (str/join
     "\n"
     ["SUMMARY"
      "Items: 2"
      "Results: 2"
      "Warnings: 1"
      "Errors: 0"
      "Cumulative: 7ms"
      "Elapsed: 8ms"]))]
"#,
    )
    .expect("layered formatting contract should evaluate");

    assert_eq!(
        output,
        "[true true true true true true true true true true]"
    );
}
