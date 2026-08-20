import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { test, expect } from "@playwright/test";

const corpusUrl = new URL(
  "../../../../hara-specs-registry/01-lang/004-foundation/draft/conformance/fixtures/foundation_behavioral.hal",
  import.meta.url
);
const foundationModuleUrls = [
  "../../lib/src/std/foundation/bytes.hal",
  "../../lib/src/std/foundation/coroutine.hal",
  "../../lib/src/std/foundation/pretty.hal",
  "../../lib/src/std/foundation/promise.hal",
  "../../lib/src/std/foundation/string.hal"
].map((relative) => new URL(relative, import.meta.url));

test("browser Wasm consumes the specs-owned Foundation behavioral corpus", async ({ page }) => {
  const corpus = await readFile(fileURLToPath(corpusUrl), "utf8");
  const modules = (
    await Promise.all(
      foundationModuleUrls.map((url) => readFile(fileURLToPath(url), "utf8"))
    )
  ).join("\n");

  await page.goto("/rust/web/index.html");
  const observed = await page.evaluate(async ({ modules, corpus }) => {
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

    hara.eval(modules);
    hara.eval(corpus);
    const summary = String(hara.eval("(foundation-summary-report)"));
    const valid = String(hara.eval("(foundation-corpus-valid?)"));
    const portablePass = String(
      hara.eval("(every? :pass (:results (foundation-profile-report)))")
    );
    const boundaryPass = String(
      hara.eval("(every? :pass (foundation-boundary-results))")
    );
    const calibrationPass = String(
      hara.eval("(every? :pass (foundation-calibration-results))")
    );
    const derivedTotals = String(
      hara.eval(
        "(let [report (foundation-summary-report)] (and (= (:surface report) (:classified report)) (= (:surface report) (+ (:portable report) (:capability-specific report) (:inventory-only report))) (= (:portable report) (+ (:passed report) (:failed report))) (= (:skipped report) (+ (:capability-specific report) (:inventory-only report)))))"
      )
    );
    const probe = JSON.parse(
      String(
        hara.eval(
          "(get (get foundation-calibration-snippets :compact-tuple-type-boundary) :source)"
        )
      )
    );
    const probeExpected = String(
      hara.eval(
        "(get (get foundation-calibration-snippets :compact-tuple-type-boundary) :expected)"
      )
    );
    const interpreted = String(hara.eval(probe));
    const artifact = hara.compileBytecode(probe);
    const compiled = String(hara.evalBytecode(artifact));
    return {
      summary,
      valid,
      portablePass,
      boundaryPass,
      calibrationPass,
      derivedTotals,
      probeExpected,
      interpreted,
      compiled
    };
  }, { modules, corpus });

  expect(observed.valid).toBe("true");
  expect(observed.portablePass).toBe("true");
  expect(observed.boundaryPass).toBe("true");
  expect(observed.calibrationPass).toBe("true");
  expect(observed.derivedTotals).toBe("true");
  expect(observed.summary).toContain(":portable");
  expect(observed.summary).toContain(":capability-specific");
  expect(observed.summary).toContain(":inventory-only");
  expect(observed.interpreted).toBe(observed.probeExpected);
  expect(observed.compiled).toBe(observed.probeExpected);
});
