
#[test]
fn portable_schema_registry_resolves_recursive_references() {
    let mut runtime = Runtime::new();
    assert_eq!(
        runtime
            .eval_native(
                "(ns typed-registry-rust-probe \
                   (:require [std.typed.registry :as registry] \
                             [std.typed.schema :as typed])) \
                 (def nodes \
                   (registry/local \
                    (quote demo) \
                    {(quote Node) \
                     (quote [:map \
                             [:value :int] \
                             [:next [:maybe Node]]])})) \
                 (def cycle \
                   (registry/local \
                    (quote cycle) \
                    {(quote A) (quote B) \
                     (quote B) (quote A)})) \
                 [(registry/qualify nodes (quote Node)) \
                  (typed/valid? \
                   (quote Node) \
                   {:value 1 :next {:value 2 :next nil}} \
                   nodes) \
                  (typed/valid? \
                   (quote Node) \
                   {:value 1 :next {:value \"two\" :next nil}} \
                   nodes) \
                  (typed/unresolved-references (quote Node) nodes) \
                  (:finding/type \
                   (first (typed/validate (quote A) 1 cycle))) ]"
            )
            .unwrap(),
        "[demo/Node true false [] :std.typed.schema/cyclic-reference]"
    );
}
