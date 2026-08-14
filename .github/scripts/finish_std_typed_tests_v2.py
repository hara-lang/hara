from pathlib import Path


Path("core/lib/test/std/typed/infer_test.hal").write_text(r'''(ns std.typed.infer-test
  (:require [std.typed.infer :as infer]
            [std.typed.schema :as schema]))

(defn check
  [id pass]
  {:id id :pass pass})

(defn form
  [source]
  (Edn/read source))

(pr-str
 [(check
   :literal-local-and-branch-inference
   (and
    (= {:kind :primitive :name :int}
       (infer/infer 1))
    (= {:kind :vector
        :item {:kind :union
               :types [{:kind :primitive :name :int}
                       {:kind :primitive :name :str}]}}
       (infer/infer [1 "two"]))
    (= {:kind :primitive :name :int}
       (infer/infer (form "(let [x 1] (+ x 2))")))
    (= {:kind :union
        :types [{:kind :primitive :name :int}
                {:kind :primitive :name :str}]}
       (infer/infer (form "(if true 1 \"two\")")))
    (= {:kind :union
        :types [{:kind :primitive :name :int}
                {:kind :primitive :name :nil}]}
       (infer/infer (form "(if true 1)")))))

  (check
   :unknown-suppresses-joins
   (and
    (schema/unknown?
     (infer/infer (form "(if true missing 1)")))
    (schema/unknown?
     (infer/infer (form "(+ missing 1)")))))

  (check
   :known-call-results
   (= {:kind :primitive :name :str}
      (infer/infer
       (form "(label 1)")
       {:namespace (symbol "demo")
        :functions {(symbol "demo/label")
                    (schema/normalize [:fn [:int] :str])}})))])
''')


probe_root = Path("core/lib/test/std/typed/probes")
probe_root.mkdir(parents=True, exist_ok=True)
(probe_root / "08_infer_load.hal").write_text(r'''(ns std.typed.probe-infer-load
  (:require [std.typed.infer :as infer]))
(pr-str [{:pass true}])
''')
(probe_root / "09_infer_forms.hal").write_text(r'''(ns std.typed.probe-infer-forms
  (:require [std.typed.infer :as infer]
            [std.typed.schema :as schema]))
(pr-str
 [{:pass
   (and
    (= {:kind :primitive :name :int}
       (infer/infer (Edn/read "(let [x 1] (+ x 2))")))
    (= {:kind :primitive :name :str}
       (infer/infer
        (Edn/read "(label 1)")
        {:namespace (symbol "demo")
         :functions {(symbol "demo/label")
                     (schema/normalize [:fn [:int] :str])}})))}])
''')
