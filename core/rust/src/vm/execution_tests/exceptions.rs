use super::*;

// ------------------------------------------------------------------
// Exceptions (issue #203): try/catch/finally and guest throw.
// ------------------------------------------------------------------

#[test]
fn throw_and_catch_basics() {
    assert_eq!(
        eval("(try (throw 41) (catch Exception error (+ error 1)))"),
        "42"
    );
    // The implicit (catch name body) form matches Exception.
    assert_eq!(eval("(try (throw :failed) (catch error error))"), ":failed");
    // First matching catch wins; later clauses do not run.
    assert_eq!(
        eval("(try (throw 41) (catch Exception a 41) (catch Exception b 42))"),
        "41"
    );
    // A non-matching class falls through to the next clause.
    assert_eq!(
        eval("(try (throw 41) (catch Problem error 0) (catch Exception error (+ error 1)))"),
        "42"
    );
    // A body value passes through an unmatched-catch try unchanged.
    assert_eq!(eval("(try 7 (catch Exception e 0))"), "7");
}

#[test]
fn catch_binds_runtime_error_messages() {
    // Runtime errors bind the message string.
    assert_eq!(
        eval("(try (/ 1 0) (catch Exception error error))"),
        "\"division by zero\""
    );
    // Errors crossing a closure call bind the bare message string, not a
    // rendered composite.
    assert_eq!(
        eval("(try ((fn [] (/ 1 0))) (catch Exception e e))"),
        "\"division by zero\""
    );
}

#[test]
fn uncaught_throws_propagate() {
    assert_eval_error("(throw :failed)", "thrown: :failed");
    assert_eval_error("(try (throw 41) (catch Problem error 0))", "thrown: 41");
    assert_eval_error(
        "(try (try (throw 41) (catch Problem error 0)) (catch Problem error 0))",
        "thrown: 41",
    );
}

#[test]
fn finally_semantics() {
    // Finally results are discarded on the success path.
    assert_eq!(eval("(try 42 (finally 0))"), "42");
    assert_eq!(eval("(try 42 43 (finally 0 1))"), "43");
    // Finally runs after a caught error without changing the outcome.
    assert_eq!(
        eval("(try (throw 41) (catch Exception error (+ error 1)) (finally 0))"),
        "42"
    );
    // An in-flight error rethrows with its identity after finally.
    assert_eq!(
        eval("(try (try (throw :original) (finally 0)) (catch Exception e e))"),
        ":original"
    );
    // An error in finally replaces the in-flight outcome (first error
    // short-circuits, matching the fiber).
    assert_eval_error("(try 1 (finally (throw 2)))", "thrown: 2");
    assert_eval_error("(try (throw 1) (catch Exception e (throw 2)))", "thrown: 2");
    assert_eval_error("(try (throw 1) (finally (throw 2)))", "thrown: 2");
}

#[test]
fn exceptions_cross_function_boundaries() {
    // try inside a function body.
    assert_eq!(
        eval("((fn [] (try (throw 1) (catch Exception e 42))))"),
        "42"
    );
    // A throw inside a called function unwinds to the caller's catch.
    assert_eq!(
        eval("(try ((fn [] (throw 41))) (catch Exception e (+ e 1)))"),
        "42"
    );
}

#[test]
fn recur_through_catch_only_try() {
    // recur in the body of a catch-only try stays in tail position.
    assert_eq!(
        eval("(loop [i 0] (try (if (< i 3) (recur (+ i 1)) i) (catch Exception e -1)))"),
        "3"
    );
    // recur in a catch body of a catch-only try.
    assert_eq!(
        eval("(loop [i 0] (try (throw 1) (catch Exception e (if (< i 3) (recur (+ i 1)) i))))"),
        "3"
    );
}

#[test]
fn try_compile_errors() {
    // Body forms cannot follow catch/finally clauses.
    let (kind, message) = compile_error("(try 1 (catch Exception e 2) 3)");
    assert_eq!(kind, CompileErrorKind::Arity);
    assert!(
        message.contains("try clauses must follow body"),
        "{message}"
    );
    // Malformed catch clauses are compile errors. The evaluator silently
    // treats a non-symbol class as non-matching; the VM rejects the
    // source instead (documented divergence).
    let (_, message) = compile_error("(try 1 (catch 42 e 0))");
    assert!(
        message.contains("catch class must be symbol [line 1, column 15]"),
        "{message}"
    );
    let (_, message) = compile_error("(try 1 (catch Exception 42 0))");
    assert!(message.contains("catch name must be symbol"), "{message}");
    let (_, message) = compile_error("(try 1 (catch))");
    assert!(
        message.contains("catch expects class, name, and body"),
        "{message}"
    );
    // throw takes exactly one value.
    let (kind, message) = compile_error("(throw)");
    assert_eq!(kind, CompileErrorKind::Arity);
    assert!(message.contains("throw expects one value"), "{message}");
    // recur cannot cross a finally boundary (checked before the tail
    // check, because the try itself suppresses tail propagation).
    let (kind, message) =
        compile_error("(loop [i 0] (try (if (< i 3) (recur (+ i 1)) i) (finally 0)))");
    assert_eq!(kind, CompileErrorKind::Recur);
    assert!(
        message.contains("recur cannot cross a finally boundary"),
        "{message}"
    );
}

#[test]
fn uncaught_throw_carries_position() {
    let program = compile_source("(try 1 (finally 0)) (throw :failed)").expect("compiles");
    let error = execute_program(std::rc::Rc::new(program)).expect_err("uncaught throw");
    let text = error.to_string();
    assert!(
        text.starts_with("thrown: :failed [line 1, column 21]"),
        "{text}"
    );
    assert!(text.contains("(instruction"), "{text}");
}

#[test]
fn global_forms_issue_223() {
    assert_eq!(eval("(ns+)"), "nil");
    assert_eq!(eval("(def player 1)"), "#'user/player");
    assert_eq!(eval("(= (def player 1) #'player)"), "true");
    assert_eq!(eval("(do (def answer 42) answer)"), "42");
    assert_eq!(
        eval("(do (def answer 19) (def answer (+ answer 23)) answer)"),
        "42"
    );
    // defn interns a real var and evaluates to it; display is qualified.
    assert_eq!(eval("(defn f [x] x)"), "#'user/f");
    assert_eq!(eval("(do (defn f [x] (+ x 1)) (f 41))"), "42");
    // Late binding: redefinition resets the shared cell.
    assert_eq!(
        eval("(do (defn f [x] 1) (defn g [] (f 0)) (defn f [x] 2) (g))"),
        "2"
    );
    assert_eq!(
        eval("(do (defn f [x] 1) (def v (var f)) (defn f [x] 2) (= v (var f)))"),
        "true"
    );
    // var / #' reads the var itself.
    assert_eq!(eval("(do (defn f [x] x) #'f)"), "#'user/f");
    assert_eq!(eval("(do (defn f [x] x) (var f))"), "#'user/f");
    // set! resets a global root and evaluates to the value.
    assert_eq!(eval("(do (def c 0) (set! c (+ c 42)) c)"), "42");
    assert_eq!(eval("(do (def c 0) (set! c 42))"), "42");
    // declare interns a nil var and evaluates to nil.
    assert_eq!(eval("(declare future)"), "nil");
    assert_eq!(eval("(declare a b)"), "nil");
    // defn- compiles like defn (private metadata).
    assert_eq!(eval("(do (defn- p [] 42) (p))"), "42");
}

#[test]
fn defstruct_forms_issue_223() {
    assert_eq!(eval("(do (defstruct Point [x y]) nil)"), "nil");
    assert_eq!(
        eval("(do (defstruct Point [x y]) (:y (->Point 19 23)))"),
        "23"
    );
    assert_eq!(
        eval("(do (defstruct Point [x y]) [(get (map->Point {:x 1 :extra 9}) :x) (get (map->Point {:x 1 :extra 9}) :y)])"),
        "[1 nil]"
    );
    assert_eq!(
        eval("(do (defstruct Point [x y]) (let [original (Point 1 2) updated (assoc original :x 10)] [(:x original) (:x updated) (instance? Point updated)]))"),
        "[1 10 true]"
    );
    assert_eq!(
        eval("(do (defstruct Point [x y]) (instance? Point (->Point 1 2)))"),
        "true"
    );
    assert_eq!(
        eval("(do (defstruct Point [x y]) (instance? Point 42))"),
        "false"
    );
    // Constructor vars are ordinary globals: late-bound and replaceable.
    assert_eq!(
        eval("(do (defstruct Point [x y]) (def make ->Point) (:x (make 1 2)))"),
        "1"
    );
}

#[test]
fn defmutable_forms_use_reference_identity_and_settable_fields() {
    assert_eq!(eval("(do (defmutable Cursor [x y]) nil)"), "nil");
    assert_eq!(
        eval("(do (defmutable Cursor [x y]) (field (->Cursor 19 23) :y))"),
        "23"
    );
    assert_eq!(
        eval("(do (defmutable Cursor [x y]) (let [cursor (map->Cursor {:x 1 :extra 9})] [(get cursor :x) (:y cursor) (count cursor)]))"),
        "[1 nil 2]"
    );
    assert_eq!(
        eval("(do (defmutable Cursor [x]) (let [cursor (Cursor 1) alias cursor result (set! (field cursor :x) 10)] [result (field alias :x) (= cursor alias) (= cursor (Cursor 10))]))"),
        "[10 10 true false]"
    );
    assert_eq!(
        eval("(do (def order []) (defmutable Cursor [x]) (def cursor (Cursor 1)) (set! (field (do (set! order (conj order :receiver)) cursor) :x) (do (set! order (conj order :replacement)) 10)) [order (field cursor :x)])"),
        "[[:receiver :replacement] 10]"
    );
    assert_eq!(
        eval("(do (defmutable Cursor [x]) (instance? Cursor (Cursor 1)))"),
        "true"
    );
}

#[test]
fn variadic_and_multi_arity_issue_223() {
    assert_eq!(eval("((fn [left & more] left) 42 1 2)"), "42");
    assert_eq!(eval("((fn [left & more] more) 42 1 2)"), "(1 2)");
    assert_eq!(eval("((fn [left & more] more) 42)"), "()");
    assert_eval_error(
        "((fn [l r & more] l) 1)",
        "function expects at least 2 arguments",
    );
    assert_eq!(
        eval("(do (defn choose ([v] v) ([l r] (+ l r))) (+ (choose 19) (choose 20 3)))"),
        "42"
    );
    assert_eq!(
        eval("(do (defn sum3 ([a b] (+ a b)) ([a b c & more] (+ a b c))) (sum3 19 20 3))"),
        "42"
    );
    assert_eq!(
        eval("(do (defn rest-args [f & r] r) (rest-args 42 1 2))"),
        "(1 2)"
    );
}

#[test]
fn global_form_errors_issue_223() {
    let (kind, message) = compile_error("(set! missing 1)");
    assert_eq!(kind, CompileErrorKind::UnboundSymbol);
    assert!(message.contains("unbound var: missing"), "{message}");
    let (kind, message) = compile_error("(var missing)");
    assert_eq!(kind, CompileErrorKind::UnboundSymbol);
    assert!(message.contains("unbound var: missing"), "{message}");
    let (_, message) = compile_error("(let [x 1] (set! x 2))");
    assert!(message.contains("set! targets a global var"), "{message}");
    // Mutable-field errors surface at runtime, not compile time.
    assert_eval_error(
        "(do (defmutable P [x]) (field (->P 1) :z))",
        "unknown mutable field: z",
    );
    assert_eval_error(
        "(do (defstruct P [x]) (field (->P 1) :x))",
        "field expects a mutable value",
    );
    assert_eval_error(
        "(do (defmutable P [x]) (field 42 :x))",
        "field expects a mutable value",
    );
    assert_eval_error(
        "(do (defmutable P [x]) (set! (field (P 1) :z) 2))",
        "unknown mutable field: z",
    );
    assert_eval_error(
        "(do (defmutable P [x]) (assoc (P 1) :x 2))",
        "assoc does not support mutable values",
    );
    assert_eval_error(
        "(do (defmutable P [x]) (dissoc (P 1) :x))",
        "dissoc does not support mutable values",
    );
    assert_eval_error(
        "(do (defstruct P [x]) (instance? 42 1))",
        "instance? expects a struct or mutable type",
    );
    // Referred foundation Vars are protected; declare is forward visibility only.
    let message = Runtime::new()
        .compile_bytecode("(do (defn count [n] 42) (count 5))")
        .expect_err("referred foundation Var must be protected");
    assert!(
        message.contains("Cannot replace referred Var without ns omission: count"),
        "{message}"
    );
    // Uninitialized let-style errors keep their shape.
    let (_, message) = compile_error("(fn [a &] a)");
    assert!(
        message.contains("rest parameter must be the last"),
        "{message}"
    );
}

#[test]
fn async_metadata_and_await_lowering_are_explicit() {
    let program = compile_source("(defn ^:async delayed [p] (std.foundation.coroutine/await p))")
        .expect("async function must compile");
    let async_proto = program
        .functions
        .iter()
        .find(|function| function.name.as_deref() == Some("delayed"))
        .expect("named async prototype");
    assert!(async_proto.async_function);
    assert!(async_proto.code.contains(&super::Instruction::Await));
}

#[test]
fn await_infers_a_suspending_synchronous_function() {
    let program = compile_source("(defn delayed [p] (std.foundation.coroutine/await p))")
        .expect("await should infer suspension support");
    let prototype = program
        .functions
        .iter()
        .find(|function| function.name.as_deref() == Some("delayed"))
        .expect("named prototype");
    assert!(
        !prototype.async_function,
        "inferred await must not force a promise wrapper"
    );
    assert!(prototype.code.contains(&super::Instruction::Await));

    compile_source("(defn outer [p] (fn [] (std.foundation.coroutine/await p)))")
        .expect("nested functions infer their own suspension support");
}

#[test]
fn inferred_await_returns_directly_until_it_really_suspends() {
    let registry = NamespaceRegistry::new("user");
    let source = Promise::new();
    registry
        .find_or_create("user")
        .intern("source", Value::Promise(source.clone()));
    let program = compile_source_with(
        "(do (defn delayed [] (std.foundation.coroutine/await source)) (delayed))",
        &registry,
    )
    .unwrap();
    let mut fiber =
        crate::core::with_namespace_registry(&registry, || super::VmFiber::start(Rc::new(program)));
    assert!(matches!(fiber.state(), super::VmFiberState::Suspended));
    source.resolve(Value::Number(9));
    assert!(matches!(
        fiber.poll(),
        super::VmFiberState::Completed(Value::Number(9))
    ));

    let registry = NamespaceRegistry::new("user");
    let ready = Promise::new();
    ready.resolve(Value::Number(42));
    registry
        .find_or_create("user")
        .intern("source", Value::Promise(ready));
    let program = compile_source_with(
        "(do (defn immediate [] (std.foundation.coroutine/await source)) (immediate))",
        &registry,
    )
    .unwrap();
    assert_eq!(
        execute_program_with_globals(Rc::new(program), &registry).unwrap(),
        Value::Number(42)
    );
}

#[test]
fn async_calls_return_promises_and_adopt_direct_values() {
    let program = compile_source("(do (defn ^:async answer [] 42) (answer))").unwrap();
    let Value::Promise(result) = execute_program(Rc::new(program)).unwrap() else {
        panic!("async call must return a promise")
    };
    assert_eq!(result.state(), PromiseState::Fulfilled(Value::Number(42)));
}

#[test]
fn pending_async_child_is_resumed_only_when_the_scheduler_is_polled() {
    let registry = NamespaceRegistry::new("user");
    let source = Promise::new();
    registry
        .find_or_create("user")
        .intern("source", Value::Promise(source.clone()));
    let program = compile_source_with(
        "(do (defn ^:async delayed [] (std.foundation.coroutine/await source)) (delayed))",
        &registry,
    )
    .unwrap();
    let Value::Promise(result) = execute_program_with_globals(Rc::new(program), &registry).unwrap()
    else {
        panic!("async call must return a promise")
    };
    assert_eq!(result.state(), PromiseState::Pending);
    source.resolve(Value::Number(9));
    assert_eq!(result.state(), PromiseState::Fulfilled(Value::Number(9)));
}

#[test]
fn cancelling_async_result_propagates_to_the_pending_host_promise() {
    let registry = NamespaceRegistry::new("user");
    let source = Promise::new();
    let cancelled = Rc::new(Cell::new(false));
    let observed = cancelled.clone();
    source.set_cancel_hook(Rc::new(move || observed.set(true)));
    registry
        .find_or_create("user")
        .intern("source", Value::Promise(source));
    let program = compile_source_with(
        "(do (defn ^:async delayed [] (std.foundation.coroutine/await source)) (delayed))",
        &registry,
    )
    .unwrap();
    let Value::Promise(result) = execute_program_with_globals(Rc::new(program), &registry).unwrap()
    else {
        panic!("async call must return a promise")
    };
    assert!(result.cancel());
    assert!(cancelled.get());
    assert!(matches!(
        result.state(),
        PromiseState::Rejected(error) if error.is_cancelled()
    ));
}

#[test]
fn async_calls_always_return_settled_promises_on_the_fast_path() {
    let value = eval_source("(do (defn ^:async answer [] 42) (answer))")
        .expect("async call must return normally");
    let Value::Promise(promise) = value else {
        panic!("async call returned {value:?}");
    };
    assert_eq!(
        promise.state(),
        crate::core::PromiseState::Fulfilled(Value::Number(42))
    );

    let value = eval_source("(do (defn ^:async fail [] (throw \"boom\")) (fail))")
        .expect("async throw rejects rather than escaping");
    let Value::Promise(promise) = value else {
        panic!("async call returned {value:?}");
    };
    assert!(matches!(
        promise.state(),
        crate::core::PromiseState::Rejected(ref error) if error.message().contains("boom")
    ));
}

#[test]
fn async_calls_retain_and_resume_pending_child_fibers() {
    let registry = crate::kernel::NamespaceRegistry::new("user");
    let pending = crate::core::Promise::new();
    registry
        .current()
        .intern("pending", Value::Promise(pending.clone()));
    let program = super::compile_source_with(
        "(do (defn ^:async delayed [] (std.foundation.coroutine/await pending)) (delayed))",
        &registry,
    )
    .expect("async source must compile");
    let value = super::execute_program_with_globals(std::rc::Rc::new(program), &registry)
        .expect("async call returns its result promise");
    let Value::Promise(result) = value else {
        panic!("async call returned {value:?}");
    };
    assert_eq!(result.state(), crate::core::PromiseState::Pending);
    pending.resolve(Value::Number(42));
    assert_eq!(
        result.state(),
        crate::core::PromiseState::Fulfilled(Value::Number(42))
    );
}

#[test]
fn vm_host_call_returns_a_native_promise_and_resumes_through_await() {
    let pending = Promise::new();
    let provider_promise = pending.clone();
    let observed = Rc::new(RefCell::new(Vec::new()));
    let provider_observed = observed.clone();
    let provider = Rc::new(
        move |service: String, method: String, arguments: Vec<Value>| {
            provider_observed
                .borrow_mut()
                .push((service, method, arguments));
            Ok(Value::Promise(provider_promise.clone()))
        },
    );
    let program = compile_source(
        "(do (defn ^:async delayed [] (std.foundation.coroutine/await (std.native.Host/call \"nginx\" \"sleep\" [25]))) (delayed))",
    )
    .unwrap();
    let value = crate::core::with_host_calls(provider, || execute_program(Rc::new(program)))
        .expect("host call returns its promise");
    let Value::Promise(result) = value else {
        panic!("async host call returned {value:?}");
    };
    assert_eq!(result.state(), PromiseState::Pending);
    assert_eq!(
        observed.borrow().as_slice(),
        &[("nginx".into(), "sleep".into(), vec![Value::Number(25)])]
    );
    pending.resolve(Value::String("done".into()));
    assert_eq!(
        result.state(),
        PromiseState::Fulfilled(Value::String("done".into()))
    );
}

#[cfg(feature = "tracing-jit")]
#[test]
fn typed_numeric_functions_start_guarded_tracing_on_the_first_backedge() {
    use crate::kernel::{FunctionSchema, SchemaType};

    let mut program = compile_source(
        "(do (defn sum-to [n] \
           (loop [i 0 total 0] \
             (if (< i n) (recur (+ i 1) (+ total i)) total))) \
         (sum-to 10))",
    )
    .unwrap();
    program.namespace = Some("user".into());
    program.function_types.insert(
        "user/sum-to".into(),
        SchemaType::Function(vec![FunctionSchema {
            fixed: vec![SchemaType::Primitive("int".into())],
            rest: None,
            output: Box::new(SchemaType::Primitive("int".into())),
        }]),
    );
    let program = Rc::new(program);

    assert_eq!(execute_program(program.clone()).unwrap(), Value::Number(45));
    assert!(super::machine::cached_trace_count(&program) > 0);
    let telemetry = super::machine::cached_jit_telemetry(&program);
    assert_eq!(telemetry.recording_starts, 1);
    assert!(
        telemetry.backedges < 16,
        "typed trace waited for the generic threshold"
    );
}
