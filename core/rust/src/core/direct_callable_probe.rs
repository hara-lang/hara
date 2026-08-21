#[cfg(test)]
pub(crate) fn direct_callable_probe_arguments(specification: &DirectCallableSpec) -> Vec<Value> {
    let count = match specification.arity {
        DirectCallableArity::Exact(count) => count,
        DirectCallableArity::Between { minimum, .. }
        | DirectCallableArity::AtLeast(minimum)
        | DirectCallableArity::EvenAtLeast(minimum)
        | DirectCallableArity::OddAtLeast(minimum) => minimum,
        DirectCallableArity::Even | DirectCallableArity::Any => 0,
    };
    vec![Value::Nil; count]
}

#[cfg(test)]
#[test]
fn removed_structural_dispatch_paths_cannot_reappear() {
    let sources = [
        include_str!("value.rs"),
        include_str!("../runtime/runtime.rs"),
        include_str!("../vm/machine/dispatch.rs"),
        include_str!("../fiber.rs"),
        include_str!("../kernel/generated.rs"),
    ];
    for forbidden in [
        "structural_function_value",
        "STRUCTURAL_NATIVE_DISPATCH",
        "canonical_native_call",
        "structural_callable_names",
    ] {
        assert!(
            sources.iter().all(|source| !source.contains(forbidden)),
            "removed structural dispatch path reappeared: {forbidden}"
        );
    }
    assert!(
        !include_str!("value.rs").contains("eval(&Form::List"),
        "value-level callable dispatch must not rebuild a call form"
    );
}
