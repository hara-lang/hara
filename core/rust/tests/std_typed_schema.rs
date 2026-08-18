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
