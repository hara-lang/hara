#!/usr/bin/env node
import { access, readFile } from "node:fs/promises";
import { join, resolve } from "node:path";

const root = resolve(process.argv[2] ?? "");
if (!process.argv[2]) {
  throw new Error("usage: verify-bytecode-observation-runtime.mjs <runtime-root>");
}

const payloads = [
  ["rust/bytecode-observation.wasm", "rust/host/bytecode-observation.js"],
  [
    "examples/music/runtime/bytecode-observation.wasm",
    "examples/music/runtime/host/bytecode-observation.js",
  ],
];

for (const [wasmPath, hostPath] of payloads) {
  await access(join(root, wasmPath));
  await access(join(root, hostPath));

  const source = await readFile(join(root, hostPath), "utf8");
  for (const forbidden of ["innerHTML", "insertAdjacentHTML", "new Function", "eval("]) {
    if (source.includes(forbidden)) {
      throw new Error(`${hostPath} contains forbidden browser construction: ${forbidden}`);
    }
  }
  for (const required of [
    "BytecodeObservationRuntime",
    "BytecodeObservationSession",
    "observation_invoke",
    "dispose-all",
  ]) {
    if (!source.includes(required)) {
      throw new Error(`${hostPath} missing ${required}`);
    }
  }

  const bytes = await readFile(join(root, wasmPath));
  const module = await WebAssembly.compile(bytes);
  const imports = WebAssembly.Module.imports(module);
  if (imports.length !== 0) {
    const rendered = imports
      .map(({ module, name, kind }) => `${module}.${name}:${kind}`)
      .join(", ");
    throw new Error(`${wasmPath} unexpectedly imports host bindings: ${rendered}`);
  }
  const instance = await WebAssembly.instantiate(module, {});
  const exports = instance.exports;
  for (const name of [
    "memory",
    "observation_abi_version",
    "observation_alloc",
    "observation_dealloc",
    "observation_invoke",
  ]) {
    if (!(name in exports)) throw new Error(`${wasmPath} missing ${name}`);
  }
  if (exports.observation_abi_version() !== 1) {
    throw new Error(`${wasmPath} has an unsupported observation ABI`);
  }

  const hostModuleUrl = `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
  const { loadBytecodeObservationRuntime } = await import(hostModuleUrl);
  const runtime = await loadBytecodeObservationRuntime({
    wasmBytes: bytes,
    wasmUrl: new URL("file:///bytecode-observation.wasm"),
  });
  const session = runtime.compileNamed(
    "archive/verification",
    "archive-verification.hal",
    "(+ 1 (* 2 3))",
  );
  const trace = session.run(1_000);
  if (trace?.schema !== "hal.bytecode-trace/0-alpha") {
    throw new Error(`${wasmPath} did not emit a versioned live trace`);
  }
  if (session.resultDisplay() !== "7") {
    throw new Error(`${wasmPath} did not execute the packaged bytecode machine`);
  }
  if (session.metrics()?.schema !== "hal.bytecode-metrics/0-alpha") {
    throw new Error(`${wasmPath} did not emit versioned metrics`);
  }
  if (session.events()?.schema !== "hal.bytecode-events/0-alpha") {
    throw new Error(`${wasmPath} did not emit versioned events`);
  }
  if (session.dispose() !== true || runtime.dispose() !== true) {
    throw new Error(`${wasmPath} did not dispose its opaque observation state`);
  }
}

console.log(`verified bytecode observation runtime: ${root}`);
