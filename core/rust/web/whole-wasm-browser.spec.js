import { test, expect } from "@playwright/test";

test("Chromium compiles and executes a Hara whole-Wasm function", async ({ page }) => {
  await page.goto("/rust/web/index.html");
  const result = await page.evaluate(async () => {
    const { start } = await import("/rust/web/packages/browser/dist/hara-wasm-full/hara.mjs");
    const hara = await start();
    const compiled = await hara.compileWholeWasm(
      "(loop [i 0 acc 0] (if (< i 5000) (recur (+ i 1) (+ acc i)) acc))"
    );
    return String(compiled.call());
  });
  expect(result).toBe("12497500");
});

test("Chromium executes the exact HNW0 artifact already run by Wasmtime", async ({ page }) => {
  await page.goto("/rust/web/index.html");
  const observed = await page.evaluate(async () => {
    const { start } = await import("/rust/web/packages/browser/dist/hara-wasm-full/hara.mjs");
    const response = await fetch("/target/whole-wasm-native-browser-parity.hnw");
    if (!response.ok) {
      throw new Error(`unable to fetch parity artifact: ${response.status}`);
    }
    const artifact = new Uint8Array(await response.arrayBuffer());
    const hara = await start();
    const compiled = await hara.loadWholeWasm(artifact);
    return {
      magic: String.fromCharCode(...artifact.subarray(0, 4)),
      byteLength: artifact.byteLength,
      result: String(compiled.call())
    };
  });

  expect(observed.magic).toBe("HNW0");
  expect(observed.byteLength).toBeGreaterThan(40);
  expect(observed.result).toBe("12497500");
});
