from pathlib import Path

path = Path("core/rust/src/fiber.rs")
source = path.read_text()

old_route = '        Some("def") | Some("set!") | Some("var/set") => bind_form(v, env, k),\n'
new_route = (
    '        Some("def") | Some("var/set") => bind_form(v, env, k),\n'
    '        Some("set!") => set_form(v, env, k),\n'
)
if source.count(old_route) != 1:
    raise SystemExit(f"expected one set! route, found {source.count(old_route)}")
source = source.replace(old_route, new_route)

bind_anchor = "fn bind_form(v: Vec<Form>, env: Rc<RefCell<HashMap<String, Value>>>, k: Cont) -> Step {\n"
set_form = r'''fn set_form(v: Vec<Form>, env: Rc<RefCell<HashMap<String, Value>>>, k: Cont) -> Step {
    if v.len() != 3 {
        return k(Err("set! expects a place and value".into()));
    }
    if matches!(&v[1], Form::Symbol(_) | Form::Metadata(_, _)) {
        return bind_form(v, env, k);
    }
    let Form::List(place) = &v[1] else {
        return k(Err("set! expects a name symbol or field place".into()));
    };
    if !matches!(place.first(), Some(Form::Symbol(operation)) if operation == "field") {
        return k(Err("set! expects a name symbol or field place".into()));
    }
    if place.len() != 3 {
        return k(Err(
            "set! field place expects a receiver and field".into(),
        ));
    }
    let field = match &place[2] {
        Form::Keyword(field) | Form::Symbol(field) if !field.contains('/') => field.clone(),
        _ => {
            return k(Err(
                "set! field place expects an unqualified literal field".into(),
            ))
        }
    };
    let receiver = place[1].clone();
    let replacement = v[2].clone();
    let replacement_env = env.clone();
    one(
        receiver,
        env,
        Box::new(move |receiver_result| match receiver_result {
            Ok(receiver) => one(
                replacement,
                replacement_env,
                Box::new(move |replacement_result| match replacement_result {
                    Ok(replacement) => {
                        k(crate::core::mutable_field_set(&receiver, &field, replacement))
                    }
                    Err(error) => k(Err(error)),
                }),
            ),
            Err(error) => k(Err(error)),
        }),
    )
}

'''
if source.count(bind_anchor) != 1:
    raise SystemExit(f"expected one bind_form anchor, found {source.count(bind_anchor)}")
source = source.replace(bind_anchor, set_form + bind_anchor)

test_anchor = '''    #[test]
    fn anonymous_namespace_form_is_a_session_local_noop() {
'''
tests = r'''    #[test]
    fn mutable_field_set_place_updates_and_returns_replacement() {
        let registry = crate::kernel::NamespaceRegistry::new("user");
        crate::core::with_namespace_registry(&registry, || {
            let mut fiber = EvalFiber::start(
                "(do (defmutable Cursor [x y]) \
                 (def cursor (Cursor 1 2)) \
                 (if (= (set! (field cursor :x) 42) 42) \
                   (field cursor :x) \
                   -1))",
                HashMap::new(),
            )
            .unwrap();
            assert_eq!(fiber.drive_sync(), Ok(Value::Number(42)));
        });
    }

    #[test]
    fn mutable_field_set_place_resumes_after_replacement_suspends() {
        let registry = crate::kernel::NamespaceRegistry::new("user");
        crate::core::with_namespace_registry(&registry, || {
            let promise = Promise::new();
            let mut environment = HashMap::new();
            environment.insert("replacement".into(), Value::Promise(promise.clone()));
            let mut fiber = EvalFiber::start(
                "(do (defmutable Cursor [x]) \
                 (def cursor (Cursor 1)) \
                 (set! (field cursor :x) (deref replacement)) \
                 (field cursor :x))",
                environment,
            )
            .unwrap();
            assert_eq!(fiber.state(), EvalFiberState::Suspended);
            promise.resolve(Value::Number(42));
            assert_eq!(
                fiber.resume(promise.state()),
                EvalFiberState::Completed(Value::Number(42))
            );
        });
    }

'''
if source.count(test_anchor) != 1:
    raise SystemExit(f"expected one test anchor, found {source.count(test_anchor)}")
source = source.replace(test_anchor, tests + test_anchor)
path.write_text(source)
