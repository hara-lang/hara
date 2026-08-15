use super::SessionKernel;

#[test]
fn fresh_runtime_loads_foundation_from_embedded_hal_sources() {
    let mut kernel = SessionKernel::new();

    let root = kernel
        .eval("ROOT", "(std.foundation/if-not false 42)")
        .expect("root Foundation source should load");
    assert!(root.contains("42"), "unexpected root result: {root}");

    let eager = kernel
        .eval("ROOT", "(std.foundation.string/upper \"hara\")")
        .expect("eager Foundation child source should load");
    assert!(eager.contains("HARA"), "unexpected eager result: {eager}");

    let lazy = kernel
        .eval("ROOT", "(do (require 'code.vm.model) 42)")
        .expect("lazy canonical HAL source should load");
    assert!(lazy.contains("42"), "unexpected lazy result: {lazy}");
}
