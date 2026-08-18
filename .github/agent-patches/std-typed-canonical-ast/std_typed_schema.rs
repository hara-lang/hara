use hara_wasm::Runtime;

#[test]
fn portable_schema_accepts_canonical_and_native_forms() {
    let mut runtime = Runtime::new();
    assert_eq!(
        runtime
            .eval_native(
                "(ns typed-schema-rust-probe \
                   (:require [std.typed.schema :as typed])) \
                 (let [primitive (schema :int) \
                       user (schema [:map [:name :str]])] \
                   [(= (typed/normalize :int) (typed/normalize [:int])) \
                    (= (typed/normalize :int) (typed/normalize primitive)) \
                    (typed/valid? [:int] 42) \
                    (typed/valid? [:int] \"42\") \
                    (typed/valid? user {:name \"Ada\"}) \
                    (typed/valid? user {:name 42}) \
                    (typed/compatible? primitive :int)])"
            )
            .unwrap(),
        "[true true true false true false true]"
    );
}

#[test]
fn native_schema_ast_is_the_portable_normal_form() {
    let mut runtime = Runtime::new();
    assert_eq!(
        runtime
            .eval_native(
                "(ns typed-schema-ast-rust-probe \
                   (:require [std.typed.schema :as typed])) \
                 (defn canonical-ast? [surface] \
                   (let [compiled (schema surface) \
                         normalized (typed/normalize surface) \
                         ast (Schema/ast compiled)] \
                     (and (= normalized ast) \
                          (= ast (typed/normalize ast)) \
                          (= compiled (schema ast))))) \
                 (let [surfaces \
                       [:int \
                        (quote [:or :int :str :int]) \
                        (quote [:vector [:maybe :int]]) \
                        (quote [:tuple :keyword :int :str]) \
                        (quote [:map [:name :str] [:tags [:vector :keyword]]]) \
                        (quote [:fn [:str & :any] :str]) \
                        (quote [:function [:fn [:int] :int] \
                                          [:fn [:str & :any] :str]]) \
                        (quote [:enum :must :may]) \
                        (quote [:test/tagged 42]) \
                        (quote (var demo/Customer))]] \
                   [(every? canonical-ast? surfaces) \
                    (= (typed/normalize \
                        (quote [:map [:name :str] \
                                     [:tags [:vector :keyword]]])) \
                       {:kind :map \
                        :fields \
                        [{:name :name \
                          :type {:kind :primitive :name :str}} \
                         {:name :tags \
                          :type {:kind :vector \
                                 :item {:kind :primitive \
                                        :name :keyword}}}]}) \
                    [(Schema/kind (schema (quote [:or :int :str]))) \
                     (Schema/kind (schema (quote [:fn [:int] :int]))) \
                     (Schema/kind \
                      (schema \
                       (quote [:function [:fn [:int] :int] \
                                         [:fn [:str] :str]])))]]))"
            )
            .unwrap(),
        "[true true [:union :fn :function]]"
    );
}
