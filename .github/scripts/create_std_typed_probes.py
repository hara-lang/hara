from pathlib import Path


root = Path("core/lib/test/std/typed/probes")
root.mkdir(parents=True, exist_ok=True)

probes = {
    "00_load.hal": """(ns std.typed.probe-load
  (:require [std.typed.schema :as schema]))
(pr-str [{:pass true}])
""",
    "01_normalize.hal": """(ns std.typed.probe-normalize
  (:require [std.typed.schema :as schema]))
(pr-str [{:pass (= {:kind :primitive :name :int}
                   (schema/normalize :integer))}])
""",
    "02_arities.hal": """(ns std.typed.probe-arities
  (:require [std.typed.schema :as schema]))
(pr-str [{:pass (= 2
                   (count
                    (schema/project-arities
                     '[:function [:fn [:int] :int]
                                 [:fn [:str & :any] :str]])))}])
""",
    "03_validation.hal": """(ns std.typed.probe-validation
  (:require [std.typed.schema :as schema]))
(pr-str [{:pass (and (schema/valid? [:map [:name :str] [:count :int]]
                                    {:name \"demo\" :count 2})
                     (not (schema/valid? [:map [:name :str] [:count :int]]
                                         {:name \"demo\" :count \"two\"})))}])
""",
    "04_finding_path.hal": """(ns std.typed.probe-finding-path
  (:require [std.typed.schema :as schema]))
(pr-str [{:pass (= [1]
                   (:finding/path
                    (first (schema/validate [:vector :int] [1 \"two\"]))))}])
""",
    "05_compatibility.hal": """(ns std.typed.probe-compatibility
  (:require [std.typed.schema :as schema]))
(pr-str [{:pass (and (schema/compatible? [:maybe :int] :nil)
                     (schema/compatible? :int :num))}])
""",
    "06_resolution.hal": """(ns std.typed.probe-resolution
  (:require [std.typed.schema :as schema]))
(let [node (schema/normalize
            [:map [:next '(var Node)]]
            {:namespace 'demo})
      definitions {'demo/Node node}]
  (pr-str [{:pass (= {:kind :map
                      :fields [{:name :next
                                :type {:kind :reference :name 'demo/Node}}]}
                     (schema/resolve (schema/normalize '(var demo/Node))
                                     definitions))}]))
""",
    "07_assignment.hal": """(ns std.typed.probe-assignment
  (:require [std.typed.schema :as schema]))
(pr-str [{:pass (and (schema/assignable? :num :int)
                     (not (schema/assignable? :int :num))
                     (schema/assignable? [:fn [:int] :num]
                                         [:fn [:num] :int]))}])
""",
}

for name, source in probes.items():
    (root / name).write_text(source)
