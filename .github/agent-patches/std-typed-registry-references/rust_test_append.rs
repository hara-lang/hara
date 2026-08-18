
fn registry_runtime() -> Runtime {
    let mut runtime = Runtime::new();
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
                 (quote B) (quote A)}))"
        )
        .unwrap();
    runtime
}

#[test]
fn portable_schema_registry_qualifies_names() {
    let mut runtime = registry_runtime();
    assert_eq!(
        runtime
            .eval_native("(registry/qualify nodes (quote Node))")
            .unwrap(),
        "demo/Node"
    );
}

#[test]
fn portable_schema_registry_validates_recursive_success() {
    let mut runtime = registry_runtime();
    assert_eq!(
        runtime
            .eval_native(
                "(typed/valid? \
                   (quote Node) \
                   {:value 1 :next {:value 2 :next nil}} \
                   nodes)"
            )
            .unwrap(),
        "true"
    );
}

#[test]
fn portable_schema_registry_validates_recursive_failure() {
    let mut runtime = registry_runtime();
    assert_eq!(
        runtime
            .eval_native(
                "(typed/valid? \
                   (quote Node) \
                   {:value 1 :next {:value \"two\" :next nil}} \
                   nodes)"
            )
            .unwrap(),
        "false"
    );
}

#[test]
fn portable_schema_registry_reports_unresolved_references() {
    let mut runtime = registry_runtime();
    assert_eq!(
        runtime
            .eval_native("(typed/unresolved-references (quote Node) nodes)")
            .unwrap(),
        "[]"
    );
}

#[test]
fn portable_schema_registry_reports_alias_cycles() {
    let mut runtime = registry_runtime();
    assert_eq!(
        runtime
            .eval_native(
                "(:finding/type \
                   (first (typed/validate (quote A) 1 cycle)))"
            )
            .unwrap(),
        ":std.typed.schema/cyclic-reference"
    );
}
