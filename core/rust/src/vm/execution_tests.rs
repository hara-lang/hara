//! End-to-end compiler and synchronous-machine execution tests, including
//! control flow, arithmetic, loop/recur behavior, and source diagnostics.

use super::error::CompileErrorKind;
use super::{
    compile_source, compile_source_with, disassemble, eval_source, execute_program,
    execute_program_with_globals,
};
use crate::core::{Promise, PromiseState, Value};
use crate::kernel::NamespaceRegistry;
use crate::Runtime;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[path = "execution_tests/bindings.rs"]
mod bindings;

fn eval(source: &str) -> String {
    eval_source(source)
        .map(|value| value.display())
        .expect("evaluation must succeed")
}

#[test]
fn embedded_foundation_reduce_executes_in_bytecode() {
    let registry = crate::embedding_namespace_registry();
    let program = compile_source_with(
        "(reduce (fn [sum values] (std.foundation/+ sum (reduce std.foundation/+ 0 values))) 0 [[1 2] [3 4]])",
        &registry,
    )
    .expect("reduce compiles against the embedded Foundation registry");
    assert_eq!(
        execute_program_with_globals(Rc::new(program), &registry).unwrap(),
        Value::Number(10)
    );
}

#[test]
fn assoc_accepts_a_bytecode_closure_as_the_replacement() {
    assert_eq!(
        eval("(let [f (fn [value] (+ value 1)) m (assoc {} :f f)] ((get m :f) 41))"),
        "42"
    );
}

fn eval_error(source: &str) -> String {
    eval_source(source).expect_err("evaluation must fail")
}

/// Runtime errors append `(instruction NNNN)` to the display; compare the
/// stable message-and-position prefix.
fn assert_eval_error(source: &str, expected_prefix: &str) {
    let message = eval_error(source);
    assert!(
        message.starts_with(expected_prefix),
        "{source}: {message} does not start with {expected_prefix}"
    );
}

fn compile_error(source: &str) -> (CompileErrorKind, String) {
    match compile_source(source) {
        Ok(program) => panic!("expected compile error, got {}", disassemble(&program)),
        Err(error) => (error.kind(), error.to_string()),
    }
}

#[test]
fn literals() {
    assert_eq!(eval("nil"), "nil");
    assert_eq!(eval("true"), "true");
    assert_eq!(eval("false"), "false");
    assert_eq!(eval("42"), "42");
    assert_eq!(eval("-7"), "-7");
    assert_eq!(eval("1.5"), "1.5");
    // `Value::display` renders whole floats without a trailing fraction,
    // matching the existing evaluator's output.
    assert_eq!(eval("2.0"), "2");
    assert_eq!(eval("\"hello\""), "\"hello\"");
    assert_eq!(eval(":hara/name"), ":hara/name");
    assert_eq!(eval("\\a"), "\\a");
    assert!(compile_source("42N").is_err());
    assert!(compile_source("1.25M").is_err());
    assert_eq!(eval("#\"\\d+\""), "#\"\\d+\"");
    assert_eq!(eval("()"), "()");
    assert_eq!(eval("^:private (+ 1 2)"), "3");
}

#[test]
fn dynamic_collections_and_short_circuit_forms() {
    assert_eq!(eval("(let [x 19 y 23] [x y])"), "[19 23]");
    assert_eq!(eval("(let [x 42] {:answer x})"), "{:answer 42}");
    assert_eq!(eval("(let [x 42] #{x 1})"), "#{42 1}");
    assert_eq!(eval("(and true 42)"), "42");
    assert_eq!(eval("(and 19 false (/ 1 0))"), "false");
    assert_eq!(eval("(or nil false 42)"), "42");
    assert_eq!(eval("(or 42 (/ 1 0))"), "42");
    assert_eq!(eval("(cond false 1 (= 1 1) 42 :else 0)"), "42");
    assert_eq!(eval("'(a [1 2])"), "(a [1 2])");
}

#[test]
fn compiled_execution_can_return_an_immutable_value_directly() {
    let mut runtime = Runtime::core();
    let program = runtime
        .compile_bytecode("{:answer 42}")
        .expect("map must compile");
    let result = runtime
        .execute_compiled_bytecode_value(program)
        .expect("map must execute");

    assert!(matches!(
        result,
        Value::Map(_) | Value::OrderedMap(_) | Value::SortedMap(_) | Value::Trie(_)
    ));
    assert_eq!(result.display(), "{:answer 42}");
}

#[test]
fn registry_only_execution_remains_visible_to_later_interpreter_entries() {
    let mut runtime = Runtime::core();
    let program = runtime
        .compile_bytecode("(def prepared-answer 42)")
        .expect("definition must compile");
    let definition = runtime
        .execute_compiled_bytecode_registry_value(program)
        .expect("definition must execute");
    assert_eq!(definition.display(), "#'user/prepared-answer");

    // eval_native refreshes from the authoritative namespace registry, so
    // omitting the eager compatibility copy does not make definitions stale.
    assert_eq!(runtime.eval_native("prepared-answer"), Ok("42".into()));
}

#[test]
fn runtime_native_array_and_object_calls_lower_to_vm_primitives() {
    let mut runtime = Runtime::core();
    let source = "(let [a (std.native.Arr/new 1 2) \
                        o (std.native.Obj/new \"count\" 3)] \
                    (std.native.Arr/set a 0 7) \
                    (std.native.Obj/set o \"count\" 11) \
                    [(std.native.Arr/get a 0) \
                     (std.native.Obj/get o \"count\") \
                     (number? (std.native.Arr/get a 0))])";
    let program = runtime
        .compile_bytecode(source)
        .expect("native calls must compile");
    let disassembly = crate::vm::disassemble(&program);
    for operator in [
        "std.native.Arr/new",
        "std.native.Arr/get",
        "std.native.Arr/set",
        "std.native.Obj/new",
        "std.native.Obj/get",
        "std.native.Obj/set",
        "number?",
    ] {
        assert!(disassembly.contains(operator), "{operator}:\n{disassembly}");
    }
    assert_eq!(
        runtime
            .execute_compiled_bytecode_registry_value(program)
            .map(|value| value.display()),
        Ok("[7 11 true]".into())
    );
}

#[test]
fn runtime_bytecode_defmacro_registers_and_expands() {
    let mut runtime = Runtime::core();
    assert_eq!(
        runtime.eval_bytecode_native("(defmacro unless [test body] `(if ~test nil ~body))"),
        Ok("<fn>".into())
    );
    assert_eq!(
        runtime.eval_bytecode_native("(unless false 42)"),
        Ok("42".into())
    );
    assert_eq!(runtime.eval_native("(unless false 42)"), Ok("42".into()));
}

#[test]
fn foundation_source_compiles_to_bytecode() {
    let source = include_str!("../../hal-src/std/foundation.hal");
    let body = source
        .split_once("(ns std.foundation)")
        .expect("foundation namespace declaration")
        .1;
    let mut runtime = Runtime::core();
    assert!(runtime.use_namespace("std.foundation"));
    runtime.prepare_foundation_bytecode();
    let artifact = runtime
        .compile_bytecode_artifact(body)
        .unwrap_or_else(|error| panic!("foundation compile failed: {error}"));

    let mut loaded = Runtime::core();
    assert!(loaded.use_namespace("std.foundation"));
    loaded.prepare_foundation_bytecode();
    crate::core::with_definition_origin(crate::kernel::VarOrigin::HalFallback, || {
        loaded
            .eval_bytecode_artifact(&artifact)
            .unwrap_or_else(|error| panic!("foundation execute failed: {error}"));
    });
    assert!(loaded.use_namespace("std.foundation"));
    assert_eq!(loaded.eval_native("(map inc [1 2 3])").unwrap(), "[2 3 4]");
    assert_eq!(loaded.eval_native("(if-not false 42)").unwrap(), "42");
}

#[test]
fn multiple_top_level_forms() {
    assert_eq!(eval("1 2 3"), "3");
    assert_eq!(eval("(+ 1 2) (+ 3 4)"), "7");
}

#[test]
fn if_branches() {
    assert_eq!(eval("(if true 1 2)"), "1");
    assert_eq!(eval("(if false 1 2)"), "2");
    assert_eq!(eval("(if nil 1 2)"), "2");
    // Everything except nil and false is truthy, including 0 and "".
    assert_eq!(eval("(if 0 1 2)"), "1");
    assert_eq!(eval("(if \"\" 1 2)"), "1");
    assert_eq!(eval("(if false 1)"), "nil");
    assert_eq!(eval("(if (< 19 20) 42 0)"), "42");
}

#[test]
fn do_sequences() {
    assert_eq!(eval("(do)"), "nil");
    assert_eq!(eval("(do 1)"), "1");
    assert_eq!(eval("(do 1 2 3)"), "3");
    assert_eq!(eval("(do (do 1 2) (do 3 4))"), "4");
}

#[test]
fn arithmetic() {
    assert_eq!(eval("(+ 19 23)"), "42");
    assert_eq!(eval("(+ 1 2 3 4)"), "10");
    assert_eq!(eval("(+ 5)"), "5");
    assert_eq!(eval("(- 10 3)"), "7");
    assert_eq!(eval("(* 6 7)"), "42");
    assert_eq!(eval("(/ 17 5)"), "3");
    assert_eq!(eval("(/ -17 5)"), "-3");
    assert_eq!(eval("(% 17 5)"), "2");
    assert_eq!(eval("(mod 17 5)"), "2");
}

#[test]
fn arithmetic_errors() {
    assert_eval_error("(+)", "+ expects arguments [line 1, column 1]");
    assert_eval_error("(/ 1 0)", "division by zero [line 1, column 1]");
    assert_eval_error("(% 1 0)", "division by zero [line 1, column 1]");
    assert_eval_error("(mod 1 0)", "division by zero [line 1, column 1]");
    assert_eval_error(
        "(+ 9223372036854775807 1)",
        "integer overflow [line 1, column 1]",
    );
    assert_eval_error(
        "(- -9223372036854775808 1)",
        "integer overflow [line 1, column 1]",
    );
    assert_eval_error(
        "(* 9223372036854775807 2)",
        "integer overflow [line 1, column 1]",
    );
    assert_eval_error("(+ 1 \"a\")", "+ expects numbers [line 1, column 1]");
    assert_eq!(eval("(+ 1 1.5)"), "2.5");
    // `mod` reports its operator as `%`, matching the evaluator.
    assert_eval_error("(mod \"a\" 1)", "% expects numbers [line 1, column 1]");
}

#[test]
fn comparisons() {
    assert_eq!(eval("(< 1 2)"), "true");
    assert_eq!(eval("(< 2 1)"), "false");
    assert_eq!(eval("(< 1 2 3)"), "true");
    assert_eq!(eval("(< 1 3 2)"), "false");
    assert_eq!(eval("(<= 1 1)"), "true");
    assert_eq!(eval("(> 2 1)"), "true");
    assert_eq!(eval("(>= 2 3)"), "false");
}

#[test]
fn comparison_errors() {
    assert_eval_error(
        "(< 1)",
        "< expects at least two arguments [line 1, column 1]",
    );
    assert_eval_error("(< 1 \"a\")", "< expects numbers [line 1, column 1]");
    assert_eval_error("(= 1)", "= expects at least 2 arguments [line 1, column 1]");
}

#[test]
fn equality() {
    assert_eq!(eval("(= 1 1)"), "true");
    assert_eq!(eval("(= 1 2)"), "false");
    assert_eq!(eval("(= 1 1 1 1)"), "true");
    assert_eq!(eval("(= nil nil)"), "true");
    assert_eq!(eval("(= nil false)"), "false");
    assert_eq!(eval("(= \"a\" \"a\")"), "true");
    assert_eq!(eval("(= :a :a)"), "true");
    assert_eq!(eval("(= \\a \\a)"), "true");
    assert_eq!(eval("(= 1.5 1.5)"), "true");
    // Number and Float are distinct values, matching the evaluator.
    assert_eq!(eval("(= 1 1.0)"), "false");
}

#[test]
fn loop_zero_iterations() {
    assert_eq!(eval("(loop [i 0] (if (< i 0) (recur (+ i 1)) i))"), "0");
}

#[test]
fn loop_iterations() {
    assert_eq!(eval("(loop [i 0] (if (< i 1) (recur (+ i 1)) i))"), "1");
    assert_eq!(eval("(loop [i 0] (if (< i 100) (recur (+ i 1)) i))"), "100");
}

#[test]
fn loop_multiple_bindings() {
    assert_eq!(
        eval("(loop [i 0 acc 0] (if (< i 10) (recur (+ i 1) (+ acc i)) acc))"),
        "45"
    );
}

#[test]
fn recur_updates_are_simultaneous() {
    // Each iteration must compute both new values from the old bindings.
    assert_eq!(
        eval("(loop [x 0 y 1] (if (< x 3) (recur (+ x 1) (+ x y)) y))"),
        "4"
    );
    // Swapping two bindings through recur: one swap exchanges them.
    assert_eq!(
        eval("(loop [x 1 y 2 n 0] (if (< n 1) (recur y x (+ n 1)) (- x y)))"),
        "1"
    );
}

#[test]
fn nested_loops() {
    // Inner loop sums i*j for j in 0..3 per outer step: 3i; total 18.
    assert_eq!(
        eval("(loop [i 0 t 0] (if (< i 4) (recur (+ i 1) (+ t (loop [j 0 s 0] (if (< j 3) (recur (+ j 1) (+ s (* i j))) s)))) t))"),
        "18"
    );
}

#[test]
fn loop_body_sequences_like_do() {
    assert_eq!(eval("(loop [i 0] 1 2)"), "2");
    assert_eq!(eval("(loop [i 0] (+ i 1) i)"), "0");
    assert_eq!(eval("(loop [] 7)"), "7");
}

#[test]
fn recur_through_tail_positions() {
    // Tail `let` and `do` bodies and `if` branches are recur positions.
    assert_eq!(
        eval("(loop [i 0] (let [next (+ i 1)] (if (< i 5) (do (recur next)) i)))"),
        "5"
    );
}

#[test]
fn recur_errors() {
    let (kind, message) = compile_error("(recur 1)");
    assert_eq!(kind, CompileErrorKind::Recur);
    assert!(message.contains("recur must be inside loop"), "{message}");
    assert!(message.contains("[line 1, column 1]"), "{message}");

    let (kind, message) = compile_error("(recur)");
    assert_eq!(kind, CompileErrorKind::Recur);
    assert!(message.contains("recur must be inside loop"), "{message}");

    assert_eq!(eval("(loop [] 42)"), "42");

    let (kind, message) = compile_error("(loop [i 0] (recur 1 2))");
    assert_eq!(kind, CompileErrorKind::Recur);
    assert!(message.contains("loop recur arity mismatch"), "{message}");

    let (kind, message) = compile_error("(loop [i 0] (+ 1 (recur 2)))");
    assert_eq!(kind, CompileErrorKind::Recur);
    assert!(
        message.contains("recur must be in tail position"),
        "{message}"
    );

    let (kind, message) = compile_error("(loop [i 0] (if (recur 1) 2 3))");
    assert_eq!(kind, CompileErrorKind::Recur);
    assert!(
        message.contains("recur must be in tail position"),
        "{message}"
    );

    let (kind, message) = compile_error("(loop [i 0] (do (recur 1) i))");
    assert_eq!(kind, CompileErrorKind::Recur);
    assert!(
        message.contains("recur must be in tail position"),
        "{message}"
    );
}

#[test]
fn compile_arity_errors_match_evaluator_messages() {
    for (source, expected) in [
        ("(if)", "if expects 2 or 3 arguments [line 1, column 1]"),
        (
            "(if 1 2 3 4)",
            "if expects 2 or 3 arguments [line 1, column 1]",
        ),
        (
            "(let)",
            "let expects bindings and a body [line 1, column 1]",
        ),
        (
            "(let [x 1])",
            "let expects bindings and a body [line 1, column 1]",
        ),
        (
            "(let 1 x)",
            "let expects a binding list or vector [line 1, column 6]",
        ),
        (
            "(let [x] x)",
            "let bindings require name/value pairs [line 1, column 6]",
        ),
        (
            "(loop [i 0])",
            "loop expects bindings and a body [line 1, column 1]",
        ),
        (
            "(loop 1 2)",
            "loop expects a binding list or vector [line 1, column 7]",
        ),
        (
            "(loop [i] i)",
            "loop bindings require name/value pairs [line 1, column 7]",
        ),
    ] {
        let (kind, message) = compile_error(source);
        assert_eq!(kind, CompileErrorKind::Arity, "{source}");
        assert_eq!(message, expected, "{source}");
    }
}

#[test]
fn unbound_symbols_are_compile_errors_with_positions() {
    let (kind, message) = compile_error("unknown");
    assert_eq!(kind, CompileErrorKind::UnboundSymbol);
    assert_eq!(message, "unbound symbol: unknown [line 1, column 1]");
    let (kind, message) = compile_error("(let [x 1] (+ x y))");
    assert_eq!(kind, CompileErrorKind::UnboundSymbol);
    assert_eq!(message, "unbound symbol: y [line 1, column 17]");
    assert_eq!(eval("(first [1 2])"), "1");
}

#[test]
fn literal_collections_and_collection_primitives() {
    assert_eq!(eval("[1 2 3]"), "[1 2 3]");
    assert_eq!(eval("{:a 1}"), "{:a 1}");
    assert_eq!(eval("#{1 2}"), "#{1 2}");
    assert_eq!(eval("(nth [10 20 30] 1)"), "20");
    assert_eq!(eval("(assoc {} :answer 42)"), "{:answer 42}");
    assert_eq!(
        eval("(let [before {:a 1} after (assoc before :b 2)] (+ (if (= nil (get before :b)) 40 0) (get after :b)))"),
        "42"
    );
    assert_eq!(eval("(first (rest [1 2]))"), "2");
}

#[test]
fn tail_recur_assoc_moves_dead_local_without_mutating_persistent_aliases() {
    assert_eq!(
        eval(
            "(let [original {:seed 1}
                   built (loop [i 0 value original]
                           (if (< i 500)
                             (recur (+ i 1) (assoc value i (+ i 1)))
                             value))]
               [(count original) (get original 499) (count built) (get built 499)])"
        ),
        "[1 nil 501 500]"
    );
}

#[test]
fn mutable_collections_build_in_place_and_freeze_once() {
    assert_eq!(
        eval(
            "(let [m (to-mutable {})]
                (do
                  (loop [i 0]
                    (if (< i 500)
                      (do (assoc m i (+ i 1)) (recur (+ i 1)))
                      nil))
                  (let [p (to-persistent m)]
                    (+ (count p) (get p 499)))))"
        ),
        "1000"
    );
    assert_eval_error(
        "(let [m (to-mutable {}) p (to-persistent m)] (do p (assoc m :late 1)))",
        "mutable collection used after to-persistent",
    );
}

#[test]
fn mutable_conversion_is_not_constant_folded_across_executions() {
    let program = Rc::new(
        compile_source(
            "(loop [i 0 m (to-mutable {})]
           (if (< i 10)
             (recur (+ i 1) (assoc m i (+ i 1)))
             (get (to-persistent m) 9)))",
        )
        .unwrap(),
    );
    assert_eq!(execute_program(program.clone()).unwrap().display(), "10");
    assert_eq!(execute_program(program).unwrap().display(), "10");
}

#[test]
fn fn_values_and_direct_calls() {
    assert_eq!(eval("(fn [x] x)"), "<fn>");
    assert_eq!(eval("((fn [x] x) 1)"), "1");
    assert_eq!(eval("((fn [x y] (+ x y)) 19 23)"), "42");
    assert_eq!(eval("(let [f (fn [x] (+ x 1))] (f 41))"), "42");
    assert_eq!(eval("(let [f (fn [x] x)] (= f f))"), "true");
    assert_eq!(eval("(= (fn [x] x) (fn [x] x))"), "false");
    // Zero-argument functions.
    assert_eq!(eval("((fn [] 42))"), "42");
}

#[test]
fn immediate_fixed_arity_closures_inline_into_lexical_slots() {
    let program = compile_source("((fn [x] (+ x 1)) 41)").expect("compiles");
    let listing = disassemble(&program);
    assert!(!listing.contains("Closure"), "{listing}");
    assert!(!listing.contains("Call"), "{listing}");
    assert!(listing.contains("StoreLocal"), "{listing}");
    assert_eq!(eval("((fn [x] (+ x 1)) 41)"), "42");
    assert_eq!(eval("(let [x 40] ((fn [x y] (+ x y)) 19 23))"), "42");
    // Arguments resolve before the inlined parameter scope is introduced.
    assert_eq!(eval("(let [x 20] ((fn [x y] (+ x y)) 19 (+ x 3)))"), "42");
    // A recur nested in the function body retains its own call boundary.
    assert_eq!(
        eval("((fn [n] (loop [i n] (if (< i 1) 42 (recur (- i 1))))) 10000)"),
        "42"
    );
}

#[test]
fn closures_capture_lexical_environment() {
    assert_eq!(eval("(let [x 19] ((fn [y] (+ x y)) 23))"), "42");
    // Captures are by value at closure-creation time.
    assert_eq!(eval("(let [x 1 f (fn [] x)] (let [x 2] (+ (f) x)))"), "3");
    // Nested closures capture through intermediate scopes.
    assert_eq!(eval("(((fn [x] (fn [y] (+ x y))) 19) 23)"), "42");
    // Loop bindings are capturable.
    assert_eq!(
        eval("(loop [i 0 acc 0] (if (< i 5) (recur (+ i 1) ((fn [x] (+ x i)) acc)) acc))"),
        "10"
    );
}

#[test]
fn defn_lowering_binds_direct_calls() {
    assert_eq!(eval("(do (defn f [x] (+ x 1)) (f 41))"), "42");
    // Later defns shadow earlier ones under early binding.
    assert_eq!(
        eval("(do (defn f [x] (+ x 1)) (defn f [x] (+ x 2)) (f 40))"),
        "42"
    );
    // A defn body sees earlier defns.
    assert_eq!(
        eval("(do (defn g [x] (* x 2)) (defn h [x] (+ (g x) 1)) (h 20))"),
        "41"
    );
    // Self-recursion compiles to a direct static call.
    assert_eq!(
        eval("(do (defn countdown [n] (if (< n 1) 0 (+ 1 (countdown (- n 1))))) (countdown 100))"),
        "100"
    );
    let program = compile_source(
        "(do (defn countdown [n] (if (< n 1) 0 (countdown (- n 1)))) (countdown 10))",
    )
    .unwrap();
    let listing = disassemble(&program);
    assert!(listing.contains("CallStatic 0001 1"), "{listing}");
}

#[test]
fn vm_global_recursion_uses_stackless_frames() {
    assert_eq!(
        eval("(do (defn countdown [n] (if (< n 1) 0 (countdown (- n 1)))) (countdown 10000))"),
        "0"
    );
}

#[test]
fn call_errors() {
    // Arity mismatch reports through the shared native-function boundary.
    assert_eval_error(
        "((fn [x] x) 1 2)",
        "function expects 1 arguments [line 1, column 1]",
    );
    // Calling a non-function value.
    assert_eval_error("(1 2)", "value is not callable [line 1, column 1]");
}

#[test]
fn fn_shape_errors_are_compile_errors() {
    let (kind, message) = compile_error("(fn x x)");
    assert_eq!(kind, CompileErrorKind::Arity);
    assert!(
        message.contains("function parameters must be a vector"),
        "{message}"
    );
}

#[test]
fn parse_errors_are_compile_errors() {
    let (kind, message) = compile_error("(+ 1");
    assert_eq!(kind, CompileErrorKind::Parse);
    assert!(message.contains("EOF while reading list"), "{message}");
}

#[test]
fn runtime_errors_carry_instruction_and_position() {
    let program =
        compile_source("(+ 1 2) (loop [i 0] (if (< i 3) (recur (/ 1 0)) i))").expect("compiles");
    let error = execute_program(std::rc::Rc::new(program)).expect_err("division by zero");
    let text = error.to_string();
    // The runtime error points at the failing primitive call, not the
    // enclosing `recur`.
    assert!(
        text.starts_with("division by zero [line 1, column 40]"),
        "{text}"
    );
    assert!(text.contains("(instruction"), "{text}");
    let position = error.position.expect("source position");
    assert_eq!((position.line, position.column), (1, 40));
}

#[test]
fn loop_workload_executes() {
    assert_eq!(
        eval("(loop [i 0 acc 0] (if (< i 5000) (recur (+ i 1) (+ acc (mod i 17))) acc))"),
        "39985"
    );
}

#[test]
fn multiline_source_positions() {
    let (_, message) = compile_error("(let [x 1]\n  (+ x y))");
    assert!(message.contains("[line 2, column 8]"), "{message}");
}

#[test]
fn compiled_programs_are_reusable() {
    let program = std::rc::Rc::new(compile_source("(let [x 19 y 23] (+ x y))").expect("compiles"));
    for _ in 0..3 {
        let value = execute_program(program.clone()).expect("executes");
        assert!(matches!(value, Value::Number(42)));
    }
}

#[test]
fn declare_supplies_forward_visibility_only() {
    assert_eq!(eval("(declare answer)"), "nil");
    assert_eq!(
        eval("(declare answer) (defn answer [n] (+ n 1)) (answer 41)"),
        "42"
    );
    // declare is top-level only and takes name symbols.
    let (_, message) = compile_error("(let [x 1] (declare y) x)");
    assert!(
        message.contains("declare is only supported as a top-level statement"),
        "{message}"
    );
    let (_, message) = compile_error("(declare 1)");
    assert!(
        message.contains("declare expects name symbols"),
        "{message}"
    );
}

#[test]
fn workload_disassembly_is_deterministic() {
    let source = "(loop [i 0 acc 0] (if (< i 5000) (recur (+ i 1) (+ acc (mod i 17))) acc))";
    let first = disassemble(&compile_source(source).expect("compiles"));
    let second = disassemble(&compile_source(source).expect("compiles"));
    assert_eq!(first, second);
    assert!(first.contains("JumpIfFalse ->"), "{first}");
    assert!(first.contains("StoreLocal 1"), "{first}");
    assert!(
        first.contains("PrimitiveLocalConst < local 0 constant 1"),
        "{first}"
    );
}

#[path = "execution_tests/exceptions.rs"]
mod exceptions;
