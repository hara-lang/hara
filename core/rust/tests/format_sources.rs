use hara_wasm::Runtime;
use std::fs;
use std::path::{Path, PathBuf};

const STRING: &str = "lib/src/std/foundation/string.hal";
const COMMON: &str = "lib/src/std/format/common.hal";
const TABLE: &str = "lib/src/std/format/table.hal";
const REPORT: &str = "lib/src/std/format/report.hal";
const TERMINAL: &str = "lib/src/std/format/terminal.hal";
const FORMAT: &str = "lib/src/std/format.hal";
const WORK_REPORT: &str = "lib/src/std/work/report.hal";

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

fn evaluate(paths: &[&str], tail: &str) -> Result<String, String> {
    let mut source = String::new();
    for path in paths {
        source.push_str(&read_source(path));
        source.push('\n');
    }
    source.push_str(tail);
    let mut runtime = Runtime::new();
    runtime.eval_native(&source)
}

#[test]
fn every_changed_hal_source_evaluates_in_a_fresh_native_runtime() {
    let candidates: &[&[&str]] = &[
        &[STRING],
        &[COMMON],
        &[COMMON, TABLE],
        &[COMMON, TABLE, REPORT],
        &[TERMINAL],
        &[COMMON, TABLE, REPORT, TERMINAL, FORMAT],
        &[COMMON, TABLE, REPORT, TERMINAL, WORK_REPORT],
    ];

    for paths in candidates {
        evaluate(paths, "nil")
            .unwrap_or_else(|error| panic!("candidate {paths:?} failed: {error}"));
    }
}

#[test]
fn portable_string_contract_is_exercised_inside_hal() {
    let output = evaluate(
        &[STRING],
        r#"
[(= (std.foundation.string/tag :a "/" :ns/b) "a/ns/b")
 (= (std.foundation.string/tag
     (std.foundation.string/encode-utf8 "hé"))
    "hé")
 (= (std.foundation.string/tag 'hello) "hello")
 (= (std.foundation.string/tag nil) "nil")
 (std.foundation.string/blank? nil)
 (= (std.foundation.string/trim-newlines "hello\r\n") "hello")
 (std.foundation.string/caseless= "heLLo" "HellO")
 (= (std.foundation.string/replace-at "hλra" 1 "a") "hara")
 (= (std.foundation.string/insert-at "hλra" 2 "!") "hλ!ra")
 (= (std.foundation.string/camel-case "hello__--  world") "helloWorld")
 (= (std.foundation.string/pascal-case "hello_world") "HelloWorld")
 (= (std.foundation.string/snake-case "version2Value") "version2_value")
 (= (std.foundation.string/spear-case "hello_world") "hello-world")
 (= (std.foundation.string/dot-case "hello-world") "hello.world")
 (= (std.foundation.string/snake-case "hello|World") "hello|world")
 (= (std.foundation.string/upper-case "hara") "HARA")
 (= (std.foundation.string/lower-case "HARA") "hara")
 (= (std.foundation.string/capital-case "hELLO") "Hello")
 (nil? (resolve 'std.foundation.string/joinl))
 (nil? (resolve (symbol "std.foundation.string/|")))
 (nil? (resolve 'std.foundation.string/truncate))]
"#,
    )
    .expect("portable string contract should evaluate");

    assert_eq!(
        output,
        "[true true true true true true true true true true true true true true true true true true true true true]"
    );
}

#[test]
fn layered_formatting_and_work_event_projection_are_exact() {
    let output = evaluate(
        &[STRING, COMMON, TABLE, REPORT, TERMINAL, FORMAT, WORK_REPORT],
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
  (std.work.report/complete
   (std.work.report/add-section
    (std.work.report/add-section
     (std.work.report/add-section
      (std.work.report/add-section
       (std.work.report/document
        :example/transform
        {:title "TRANSFORM CODE"
         :status :warning
         :annotations ["Done"]
         :profile
         {:sections [:items :warnings :results :summary]}})
       (std.work.report/section
        :items :progress [item-a item-b]
        {:section/title "ITEMS"}))
      (std.work.report/section
       :warnings :diagnostics [item-b]
       {:section/title "WARNINGS"}))
     (std.work.report/section
      :results :table rows
      {:section/title "RESULTS"
       :section/columns columns}))
    (std.work.report/section
     :summary :summary [summary]
     {:section/title "SUMMARY"
      :section/fields
      [:items :results :warnings :errors :cumulative :elapsed]}))
   :warning))

(def expected
  (std.foundation.string/join
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
  (std.format/emit-lines!
   [(std.format/line "first" :success)
    (std.format/line "second" :warning)]
   {:emit
    (fn [text]
      (swap! emitted-output conj text))}))

(def live-output (atom []))
(def live-observer
  (std.work.report/observer
   {:ansi false
    :emit
    (fn [text]
      (swap! live-output conj text))}))

(def _live-events
  (reduce
   (fn [count event]
     (std.work.protocol/work-event live-observer event)
     (inc count))
   0
   events))

(def replay-output (atom []))
(def replay-count
  (std.work.report/replay!
   events
   {:ansi false
    :emit
    (fn [text]
      (swap! replay-output conj text))}))

(def ansi-title
  (first
   (std.format/render-lines
    (std.format/report-lines document)
    {:ansi true})))

[(= (std.format/report document) expected)
 (= (std.work.report/render-events events) expected)
 (= emitted-count 2)
 (= (deref emitted-output) ["first" "second"])
 (std.foundation.string/starts-with? ansi-title "\u001b[1;36m")
 (std.foundation.string/ends-with? ansi-title "\u001b[0m")
 (= (std.foundation.string/join "\n" (deref live-output)) expected)
 (= (std.foundation.string/join "\n" (deref replay-output)) expected)
 (= replay-count (count (deref replay-output)))
 (= (std.format.report/report
     document
     {:include-sections [:summary]
      :include-title false
      :include-annotations false})
    (std.foundation.string/join
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
