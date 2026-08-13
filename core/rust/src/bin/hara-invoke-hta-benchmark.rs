#![cfg(not(target_arch = "wasm32"))]

use hara_wasm::core::Value;
use hara_wasm::{hta, Runtime};
use std::time::Instant;

fn main() -> Result<(), String> {
    let iterations = std::env::var("HARA_INVOKE_HTA_ITERATIONS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(10_000);
    let mut runtime = Runtime::core();
    runtime.eval_native("(ns invoke.benchmark) (defn add [a b] (+ a b))")?;
    let arguments = hta::encode(&Value::Vector(
        vec![Value::Number(20), Value::Number(22)].into(),
    ))?;

    for _ in 0..100 {
        runtime
            .invoke_hta("invoke.benchmark/add", &arguments)
            .map_err(|error| error.to_string())?;
    }
    let started = Instant::now();
    let mut result = Vec::new();
    for _ in 0..iterations {
        result = runtime
            .invoke_hta("invoke.benchmark/add", &arguments)
            .map_err(|error| error.to_string())?;
    }
    let elapsed = started.elapsed();
    if hta::decode(&result)? != Value::Number(42) {
        return Err("invoke HTA benchmark checksum failed".into());
    }
    let nanos = elapsed.as_nanos() as f64 / iterations as f64;
    println!(
        "invoke_hta iterations={iterations} total_ns={} ns_per_call={nanos:.2}",
        elapsed.as_nanos()
    );
    Ok(())
}
