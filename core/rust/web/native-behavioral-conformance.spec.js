import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { test, expect } from "@playwright/test";

const corpusUrl = new URL(
  "../../lib/test-fixtures/std/foundation/native_method_conformance.hal",
  import.meta.url
);

test("browser Wasm consumes the source-owned native behavioral corpus", async ({ page }) => {
  const corpus = await readFile(fileURLToPath(corpusUrl), "utf8");

  await page.goto("/rust/web/index.html");
  const observed = await page.evaluate(async ({ corpus }) => {
    const { start } = await import(
      "/rust/web/packages/browser/dist/hara-wasm-full/hara.mjs"
    );
    const hara = await start();
    if (
      !hara.raw ||
      typeof hara.eval !== "function" ||
      typeof hara.compileBytecode !== "function" ||
      typeof hara.evalBytecode !== "function"
    ) {
      throw new Error("full browser runtime is absent");
    }

    const valid = String(hara.eval(`${corpus}\n(native-corpus-valid?)`));
    const keys = String(hara.eval(`${corpus}\n(native-method-keys)`));
    const methods =
      keys.match(/[A-Z][A-Za-z0-9]*\/[A-Za-z0-9?!+*._-]+/g) ?? [];
    const cases = methods
      .map((method) => `(native-method-result '${method} nil)`)
      .join(" ");
    const allResults = String(hara.eval(`${corpus}\n[${cases}]`));
    const boundary = String(hara.eval(`${corpus}\n(native-boundary-report)`));
    const summary = String(
      hara.eval(`${corpus}\n(native-classification-summary)`)
    );
    const probe = "[(Maths/abs -2) (Bits/and 6 3) (Num/long 4.0)]";
    const interpreted = String(hara.eval(probe));
    const artifact = hara.compileBytecode(probe);
    const compiled = String(hara.evalBytecode(artifact));
    return {
      valid,
      methods,
      allResults,
      boundary,
      summary,
      interpreted,
      compiled
    };
  }, { corpus });

  expect(observed.valid).toBe("true");
  expect(observed.methods.length).toBeGreaterThan(0);
  expect(new Set(observed.methods).size).toBe(observed.methods.length);
  expect(observed.allResults).not.toContain(":pass false");
  expect(observed.allResults.match(/:pass true/g)?.length ?? 0).toBe(
    observed.methods.length
  );
  expect(observed.boundary).toBe(
    "[true true true true true true true true true true true true]"
  );
  expect(observed.summary).toContain(":portable");
  expect(observed.summary).toContain(":capability-specific");
  expect(observed.summary).toContain(":inventory-only");
  expect(observed.compiled).toBe(observed.interpreted);
});
