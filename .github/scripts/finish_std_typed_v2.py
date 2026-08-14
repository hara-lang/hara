from pathlib import Path


schema_path = Path("core/lib/src/std/typed/schema.hal")
schema = schema_path.read_text()
explicit_schema_header = """(ns std.typed.schema
  (:config {:blank true})
  (:require [std.foundation
             :refer [+ any? assoc boolean boolean? bytes? char? concat conj
                     count empty? every? ex-data ex-info ex-message filter
                     first fn? get get-in has? inc integer? into keys keyword?
                     list? map map? merge name namespace nil? not nth number?
                     reduce rest second set? str string? symbol symbol? true?
                     vec vector?]]))"""
minimal_schema_header = """(ns std.typed.schema
  (:config {:blank true})
  (:require [std.foundation :refer :all :exclude [resolve]]))"""
if schema.count(explicit_schema_header) != 1:
    raise SystemExit("std.typed.schema transformed header changed unexpectedly")
schema_path.write_text(schema.replace(explicit_schema_header, minimal_schema_header, 1))


infer_path = Path("core/lib/src/std/typed/infer.hal")
infer = infer_path.read_text()
explicit_infer_header = """(ns std.typed.infer
  (:config {:blank true})
  (:require [std.foundation
             :refer [any? assoc boolean? char? conj count drop empty? every?
                     filter first get get-in has? integer? keyword? list? map
                     map? name namespace nil? not number? reduce rest second
                     set? str string? symbol symbol? vec vector vector? when]]
            [std.typed.schema :as schema]))"""
minimal_infer_header = """(ns std.typed.infer
  (:config {:blank true})
  (:require [std.typed.schema :as schema]))"""
if infer.count(explicit_infer_header) != 1:
    raise SystemExit("std.typed.infer transformed header changed unexpectedly")
infer_path.write_text(infer.replace(explicit_infer_header, minimal_infer_header, 1))
