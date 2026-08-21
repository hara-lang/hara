/// Ordinary callables required before canonical Foundation source has completed
/// loading. They remain part of the one logical closed direct-callable catalog;
/// this bounded partition records the correction independently from unrelated
/// #648 and #844 work.
pub(crate) const DIRECT_CALLABLE_BOOTSTRAP_INVENTORY: &[&str] =
    &["boolean?", "promise/delay", "promise/new"];

pub(crate) const DIRECT_CALLABLE_BOOTSTRAP_CATALOG: &[DirectCallableSpec] = &[
    direct!(
        "boolean?",
        DirectCallableArity::Exact(1),
        BootstrapLibrary,
        Operation(direct_bootstrap_predicate_operation)
    ),
    direct!(
        "promise/delay",
        DirectCallableArity::Exact(2),
        RuntimePrimitive,
        Operation(direct_bootstrap_promise_operation)
    ),
    direct!(
        "promise/new",
        DirectCallableArity::Exact(1),
        RuntimePrimitive,
        Operation(direct_bootstrap_promise_operation)
    ),
];

fn direct_bootstrap_predicate_operation(
    specification: &DirectCallableSpec,
    arguments: Vec<Value>,
) -> Result<Value, String> {
    match specification.symbol {
        "boolean?" => Ok(Value::Bool(matches!(&arguments[0], Value::Bool(_)))),
        operation => Err(format!(
            "missing bootstrap predicate implementation: {operation}"
        )),
    }
}

fn direct_bootstrap_promise_operation(
    specification: &DirectCallableSpec,
    arguments: Vec<Value>,
) -> Result<Value, String> {
    let method = specification
        .symbol
        .strip_prefix("promise/")
        .expect("Promise bootstrap catalog entries use the promise/ prefix");
    native_promise_values(method, arguments)
}
