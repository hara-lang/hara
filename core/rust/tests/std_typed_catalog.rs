use hara_wasm::Runtime;

const GOLDEN_VALUE_HASH: &str =
    "sha256:3fc60b1736332b9f2e9f9e0a7dee75cc19c6287cc4e066970ef97b23a75fd34a";

fn catalog_runtime() -> Runtime {
    let mut runtime = Runtime::new();
    runtime
        .eval_native(
            r#"(ns typed-catalog-rust-probe
                  (:require [std.typed.catalog :as catalog]
                            [std.typed.catalog.codec :as codec]))
                (def base
                  (catalog/catalog
                   [{:schema/id :model/id
                     :schema/version 1
                     :schema/form :int}
                    {:schema/id :model/status
                     :schema/version 1
                     :schema/form '[:enum :active :disabled]}]
                   {:namespace 'model}))
                (def application
                  (catalog/catalog
                   [{:schema/id :app/user
                     :schema/version 1
                     :schema/form
                     '[:map
                       [:id (var id)]
                       [:status (var m/status)]]}
                    {:schema/id :app/user
                     :schema/version 2
                     :schema/form
                     '[:map
                       {:title "User record" :owner :accounts}
                       [:id (var id)]
                       [:status (var m/status)]
                       [:email {:optional true} :str]]}]
                   {:namespace 'app
                    :aliases {'m 'model}
                    :refers {'id 'model/id}
                    :parents [base]}))
                (def recursive
                  (catalog/catalog
                   [{:schema/id :tree/node
                     :schema/version 1
                     :schema/form
                     '[:map
                       [:value :int]
                       [:children [:vector (var node)]]]}]
                   {:namespace 'tree}))"#,
        )
        .unwrap();
    runtime
}

#[test]
fn portable_catalog_hash_lookup_resolution_and_graph_are_canonical() {
    let mut runtime = catalog_runtime();
    let source = format!(
        r#"[(= (codec/content-hash :demo/value 1 :int) {golden:?})
             (= (codec/content-hash :demo/value 1 :int)
                (codec/content-hash :demo/value 1 [:int]))
             (:schema/version (catalog/lookup application :app/user))
             (:schema/version (catalog/lookup application :app/user 1))
             (:schema/id (catalog/lookup application :model/id))
             (vec
              (map (fn [coordinate] (:schema/id coordinate))
                   (:dependencies/direct
                    (catalog/dependencies application :app/user 2))))
             (vec
              (map (fn [coordinate] (:schema/id coordinate))
                   (:dependencies/recursive
                    (catalog/dependencies recursive :tree/node))))
             (:kind (:schema/resolved
                     (catalog/resolve application :app/user 2)))
             (:valid (catalog/verify application))]"#,
        golden = GOLDEN_VALUE_HASH
    );
    assert_eq!(
        runtime.eval_native(&source).unwrap(),
        "[true true 2 1 :model/id [:model/id :model/status] [:tree/node] :map true]"
    );
}

#[test]
fn portable_catalog_rejects_alias_only_cycles() {
    let mut runtime = catalog_runtime();
    let error = runtime
        .eval_native(
            r#"(catalog/catalog
                 [{:schema/id :cycle/a
                   :schema/version 1
                   :schema/form 'b}
                  {:schema/id :cycle/b
                   :schema/version 1
                   :schema/form 'a}]
                 {:namespace 'cycle})"#,
        )
        .unwrap_err();
    assert!(
        error.contains("alias-only cycle"),
        "unexpected catalog error: {error}"
    );
}
