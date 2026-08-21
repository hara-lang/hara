mod base_direct_callable_impl {
    use super::*;
    include!("direct_callable_impl.rs");
}

/// Iterates the one logical direct-callable catalog, including the bounded
/// bootstrap correction partition.
pub(crate) fn direct_callable_catalog() -> impl Iterator<Item = &'static DirectCallableSpec> {
    DIRECT_CALLABLE_CATALOG
        .iter()
        .chain(DIRECT_CALLABLE_BOOTSTRAP_CATALOG.iter())
}

/// Iterates the independent runtime inventory consumed by the closure gate.
pub(crate) fn runtime_callable_inventory() -> impl Iterator<Item = &'static str> {
    RUNTIME_CALLABLE_INVENTORY
        .iter()
        .copied()
        .chain(DIRECT_CALLABLE_BOOTSTRAP_INVENTORY.iter().copied())
}

pub(crate) fn direct_callable_spec(name: &str) -> Option<&'static DirectCallableSpec> {
    direct_callable_catalog().find(|specification| specification.symbol == name)
}

pub(crate) fn validate_direct_callable_catalog() -> Result<(), String> {
    let mut inventory = std::collections::BTreeSet::new();
    let mut duplicate_inventory = Vec::new();
    for symbol in runtime_callable_inventory() {
        if !inventory.insert(symbol) {
            duplicate_inventory.push(symbol);
        }
    }

    let mut catalog = std::collections::BTreeSet::new();
    let mut duplicate_catalog = Vec::new();
    for specification in direct_callable_catalog() {
        match specification.availability {
            DirectCallableAvailability::AllTargets => {}
        }
        if !catalog.insert(specification.symbol) {
            duplicate_catalog.push(specification.symbol);
        }
    }

    let missing = inventory.difference(&catalog).copied().collect::<Vec<_>>();
    let extra = catalog.difference(&inventory).copied().collect::<Vec<_>>();
    if duplicate_inventory.is_empty()
        && duplicate_catalog.is_empty()
        && missing.is_empty()
        && extra.is_empty()
    {
        return Ok(());
    }

    Err(format!(
        "runtime callable inventory/catalog mismatch: missing={missing:?}; extra={extra:?}; duplicate-inventory={duplicate_inventory:?}; duplicate-catalog={duplicate_catalog:?}"
    ))
}

pub(crate) fn direct_callable_values() -> Result<Vec<(&'static str, Value)>, String> {
    validate_direct_callable_catalog()?;
    direct_callable_catalog()
        .map(|specification| {
            direct_callable_value(specification.symbol)
                .map(|value| (specification.symbol, value))
                .ok_or_else(|| {
                    format!(
                        "direct callable catalog entry has no implementation: {}",
                        specification.symbol
                    )
                })
        })
        .collect()
}

pub(crate) fn direct_callable_value(name: &str) -> Option<Value> {
    if let Some(specification) = DIRECT_CALLABLE_BOOTSTRAP_CATALOG
        .iter()
        .find(|specification| specification.symbol == name)
    {
        let specification = *specification;
        return Some(match specification.arity {
            DirectCallableArity::Exact(arity) => {
                native_function(specification.symbol, arity, move |arguments| {
                    invoke_bootstrap_direct_callable(&specification, arguments)
                })
            }
            _ => native_variadic_function(specification.symbol, move |arguments| {
                invoke_bootstrap_direct_callable(&specification, arguments)
            }),
        });
    }
    base_direct_callable_impl::direct_callable_value(name)
}

fn invoke_bootstrap_direct_callable(
    specification: &DirectCallableSpec,
    arguments: Vec<Value>,
) -> Result<Value, String> {
    if !specification.arity.accepts(arguments.len()) {
        return Err(format!(
            "{} expects {} arguments, got {}",
            specification.symbol,
            specification.arity.description(),
            arguments.len()
        ));
    }
    match specification.implementation {
        DirectCallableImplementation::Operation(implementation) => {
            implementation(specification, arguments)
        }
        _ => Err(format!(
            "{} is not a bootstrap catalog operation",
            specification.symbol
        )),
    }
}

#[cfg(feature = "bytecode-vm")]
pub(crate) fn direct_bootstrap_callable_value(name: &str) -> Option<Value> {
    direct_callable_value(name)
        .or_else(|| base_direct_callable_impl::direct_bootstrap_callable_value(name))
}

#[cfg(feature = "bytecode-vm")]
pub(crate) fn bytecode_callable_value(name: &str) -> Result<Value, String> {
    if let Some(value) = direct_callable_value(name) {
        return Ok(value);
    }
    base_direct_callable_impl::bytecode_callable_value(name)
}
