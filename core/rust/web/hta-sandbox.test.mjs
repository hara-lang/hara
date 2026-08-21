import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

import {
  BROWSER_WASM_SANDBOX_PROTOCOL,
  BrowserWasmSandbox,
  MCP_PURE_PROFILE,
  SANDBOX_EVAL_TARGET,
  projectBrowserSandboxValue,
  validateBrowserSandboxRequest,
} from "./packages/hta/sandbox.js";
import { HtaHandle, HtaKeyword } from "./packages/hta/index.js";

function cancellable(value) {
  let rejectPromise;
  const promise = new Promise((resolve, reject) => {
    rejectPromise = reject;
    queueMicrotask(() => resolve(value));
  });
  promise.cancel = () => {
    rejectPromise(new Error("cancelled"));
    return true;
  };
  return promise;
}

function harness({ result = 42, ready = Promise.resolve() } = {}) {
  const calls = [];
  const contexts = [];
  const workers = [];
  const workerFactory = (url) => {
    const worker = {
      url,
      terminations: 0,
      terminate() {
        this.terminations += 1;
      },
    };
    workers.push(worker);
    return worker;
  };
  const contextFactory = (options) => {
    const context = {
      options,
      ready,
      closes: 0,
      call(target, arguments_) {
        calls.push([target, arguments]);
        return cancellable(result);
      },
      close() {
        this.closes += 1;
        options.worker.terminate();
      },
    };
    contexts.push(context);
    return context;
  };
  return { calls, contexts, workers, workerFactory, contextFactory };
}

function sandbox(overrides = {}) {
  return new BrowserWasmSandbox({
    workerUrl: new URL("file:///hta-worker.js"),
    moduleBytes: new Uint8Array([0, 97, 115, 109]),
    ...overrides,
  });
}

test("validates the one-operation closed request", () => {
  const request = validateBrowserSandboxRequest({ operation: "sandbox.eval", source: "(+ 40 2)" });
  assert.equal(request.source, "(+ 40 2)");
  assert.equal(request.limits.wallMs, 5000);
  assert.throws(
    () => validateBrowserSandboxRequest({ operation: "sandbox.call", source: "(+ 40 2)" }),
    { code: "sandbox/capability-unsupported" },
  );
  assert.throws(
    () => validateBrowserSandboxRequest({ operation: "sandbox.eval", source: "1", mode: "ROOT" }),
    { code: "sandbox/request-invalid" },
  );
});


test("rejects ambiguous runtime configuration and source overflow", () => {
  assert.throws(
    () => new BrowserWasmSandbox({ workerUrl: "worker.js" }),
    { code: "sandbox/config-invalid" },
  );
  assert.throws(
    () =>
      new BrowserWasmSandbox({
        workerUrl: "worker.js",
        moduleUrl: "hara.wasm",
        moduleBytes: new Uint8Array([0]),
      }),
    { code: "sandbox/config-invalid" },
  );
  assert.throws(
    () =>
      validateBrowserSandboxRequest({
        operation: "sandbox.eval",
        source: "x".repeat(65_537),
      }),
    { code: "sandbox/source-limit" },
  );
});

test("the deadline also covers worker initialization", async () => {
  let timer;
  let rejectReady;
  const state = harness();
  state.contextFactory = (options) => ({
    options,
    ready: new Promise((_resolve, reject) => {
      rejectReady = reject;
    }),
    close() {
      rejectReady(new Error("sandbox closed"));
    },
  });
  const instance = sandbox({
    workerFactory: state.workerFactory,
    contextFactory: state.contextFactory,
    setTimer(callback) {
      timer = callback;
      return 1;
    },
    clearTimer() {},
  });
  const pending = instance.run({ operation: "sandbox.eval", source: "(+ 40 2)" });
  await new Promise((resolve) => setImmediate(resolve));
  timer();
  await assert.rejects(pending, { code: "sandbox/timed-out" });
  assert.equal(instance.snapshot().state, "closed");
});

test("runs only the sandbox target with no host authority and closes the worker", async () => {
  const state = harness();
  const instance = sandbox({ workerFactory: state.workerFactory, contextFactory: state.contextFactory });
  const result = await instance.run({ operation: "sandbox.eval", source: "(+ 40 2)" });

  assert.deepEqual(state.calls, [[SANDBOX_EVAL_TARGET, ["(+ 40 2)"]]]);
  assert.deepEqual(state.contexts[0].options.hostCalls, {});
  assert.equal(state.contexts[0].options.filesystemHost, null);
  assert.equal(state.contexts[0].options.kernelId, null);
  assert.equal(state.contexts[0].closes, 1);
  assert.ok(state.workers[0].terminations >= 1);
  assert.deepEqual(result, {
    protocol: BROWSER_WASM_SANDBOX_PROTOCOL,
    profile: MCP_PURE_PROFILE,
    status: "completed",
    value: { text: "42", json: 42 },
    cleanup: "completed",
  });
  assert.equal(instance.snapshot().state, "closed");
  await assert.rejects(
    instance.run({ operation: "sandbox.evalˆ°Í½ÕÉ”è€ˆ ¬€Ä€Ä¤ˆô¤°(€€€ì½‘”è€‰Í…¹‘‰½à½¹½ÐµÉ•ÕÍ…‰±”ˆô°(€€¤ì)ô¤ì()Ñ•ÍÐ ‰…¹•±±…Ñ¥½¸É•…¡•ÌÑ¡”…Ñ¥Ù”!QÑ…Í¬…¹±½Í•ÌÑ¡”¥¹ÍÑ…¹”ˆ°…Íå¹Œ€ ¤€ôøì(€±•ÐÉ•©•ÑAÉ½µ¥Í”ì(€±•Ð…¹•±±•€ô€Àì(€½¹ÍÐÍÑ…Ñ”€ô¡…É¹•ÍÌ ¤ì(€ÍÑ…Ñ”¹½¹Ñ•áÑ…Ñ½Éä€ô€¡½ÁÑ¥½¹Ì¤€ôø€¡ì(€€€½ÁÑ¥½¹Ì°(€€€É•…‘äèAÉ½µ¥Í”¹É•Í½±Ù” ¤°(€€€±½Í” ¤íô°(€€€…±° ¤ì(€€€€€½¹ÍÐÁÉ½µ¥Í”€ô¹•ÜAÉ½µ¥Í” ¡}É•Í½±Ù”°É•©•Ð¤€ôøì(€€€€€€€É•©•ÑAÉ½µ¥Í”€ôÉ•©•Ðì(€€€€€ô¤ì(€€€€€ÁÉ½µ¥Í”¹…¹•°€ô€ ¤€ôøì(€€€€€€€…¹•±±•€¬ô€Äì(€€€€€€€É•©•ÑAÉ½µ¥Í”¡¹•ÜÉÉ½È ‰…¹•±±•ˆ¤¤ì(€€€€€€€É•ÑÕÉ¸ÑÉÕ”ì(€€€€€ôì(€€€€€É•ÑÕÉ¸ÁÉ½µ¥Í”ì(€€€ô°(€ô¤ì(€½¹ÍÐ½¹ÑÉ½±±•È€ô¹•Ü‰½ÉÑ½¹ÑÉ½±±•È ¤ì(€½¹ÍÐ¥¹ÍÑ…¹”€ôÍ…¹‘‰½à¡ìÝ½É­•É…Ñ½ÉäèÍÑ…Ñ”¹Ý½É­•É…Ñ½Éä°½¹Ñ•áÑ…Ñ½ÉäèÍÑ…Ñ”¹½¹Ñ•áÑ…Ñ½Éäô¤ì(€½¹ÍÐÁ•¹‘¥¹œ€ô¥¹ÍÑ…¹”¹ÉÕ¸ (€€€ì½Á•É…Ñ¥½¸è€‰Í…¹‘‰½à¹•Ù…°ˆ°Í½ÕÉ”è€ˆ¡±½½Àmt¤ˆô°(€€€ìÍ¥¹…°è½¹ÑÉ½±±•È¹Í¥¹…°ô°(€€¤ì(€…Ý…¥Ð¹•ÜAÉ½µ¥Í” ¡É•Í½±Ù”¤€ôøÍ•Ñ%µµ•‘¥…Ñ”¡É•Í½±Ù”¤¤ì(€½¹ÑÉ½±±•È¹…‰½ÉÐ ¤ì(€…Ý…¥Ð…ÍÍ•ÉÐ¹É•©•ÑÌ¡Á•¹‘¥¹œ°ì½‘”è€‰Í…¹‘‰½à½…¹•±±•ˆô¤ì(€…ÍÍ•ÉÐ¹•ÅÕ…°¡…¹•±±•°€Ä¤ì(€…ÍÍ•ÉÐ¹•ÅÕ…°¡¥¹ÍÑ…¹”¹Í¹…ÁÍ¡½Ð ¤¹ÍÑ…Ñ”°€‰±½Í•ˆ¤ì)ô¤ì()Ñ•ÍÐ ‰Ý…±°µÑ¥µ”•áÁ¥Éä…¹•±ÌÉ…Ñ¡•ÈÑ¡…¸É•ÑÕÉ¹¥¹œ„±…Ñ”Ù…±Õ”ˆ°…Íå¹Œ€ ¤€ôøì(€±•ÐÑ¥µ•Èì(€±•ÐÉ•©•Ñ•ì(€½¹ÍÐÍÑ…Ñ”€ô¡…É¹•ÍÌ ¤ì(€ÍÑ…Ñ”¹½¹Ñ•áÑ…Ñ½Éä€ô€¡½ÁÑ¥½¹Ì¤€ôø€¡ì(€€€½ÁÑ¥½¹Ì°(€€€É•…‘äèAÉ½µ¥Í”¹É•Í½±Ù” ¤°(€€€±½Í” ¤íô°(€€€…±° ¤ì(€€€€€½¹ÍÐÁÉ½µ¥Í”€ô¹•ÜAÉ½µ¥Í” ¡}É•Í½±Ù”°É•©•Ð¤€ôøì(€€€€€€€É•©•Ñ•€ôÉ•©•Ðì(€€€€€ô¤ì(€€€€€ÁÉ½µ¥Í”¹…¹•°€ô€ ¤€ôøì(€€€€€€€É•©•Ñ•¡¹•ÜÉÉ½È ‰…¹•±±•ˆ¤¤ì(€€€€€€€É•ÑÕÉ¸ÑÉÕ”ì(€€€€€ôì(€€€€€É•ÑÕÉ¸ÁÉ½µ¥Í”ì(€€€ô°(€ô¤ì(€½¹ÍÐ¥¹ÍÑ…¹”€ôÍ…¹‘‰½à¡ì(€€€Ý½É­•É…Ñ½ÉäèÍÑ…Ñ”¹Ý½É­•É…Ñ½Éä°(€€€½¹Ñ•áÑ…Ñ½ÉäèÍÑ…Ñ”¹½¹Ñ•áÑ…Ñ½Éä°(€€€Í•ÑQ¥µ•È¡…±±‰…¬¤ì(€€€€€Ñ¥µ•È€ô…±±‰…¬ì(€€€€€É•ÑÕÉ¸€Äì(€€€ô°(€€€±•…ÉQ¥µ•È ¤íô°(€ô¤ì(€½¹ÍÐÁ•¹‘¥¹œ€ô¥¹ÍÑ…¹”¹ÉÕ¸¡ì(€€€½Á•É…Ñ¥½¸è€‰Í…¹‘‰½à¹•Ù…°ˆ°(€€€Í½ÕÉ”è€ˆ¡±½½Àmt¤ˆ°(€€€±¥µ¥ÑÌèìÝ…±±5Ìè€Ä°½ÕÑÁÕÑ	åÑ•Ìè€ÄÀÈÐô°(€ô¤ì(€…Ý…¥Ð¹•ÜAÉ½µ¥Í” ¡É•Í½±Ù”¤€ôøÍ•Ñ%µµ•‘¥…Ñ”¡É•Í½±Ù”¤¤ì(€Ñ¥µ•È ¤ì(€…Ý…¥Ð…ÍÍ•ÉÐ¹É•©•ÑÌ¡Á•¹‘¥¹œ°ì½‘”è€‰Í…¹‘‰½à½Ñ¥µ•µ½ÕÐˆô¤ì)ô¤ì()Ñ•ÍÐ ‰ÁÉ½©•ÑÌÑÉ…¹Í™•ÈµÍ…™”!QÙ…±Õ•Ì…¹É•©•ÑÌ±¥Ù”¡…¹‘±•Ìˆ°€ ¤€ôøì(€½¹ÍÐÙ…±Õ”€ô¹•Ü5…À¡l(€€€m¹•Ü!Ñ…-•åÝ½É ‰…¹ÍÝ•Èˆ¤°€ÐÉt°(€€€l‰¥Ñ•µÌˆ°mÑÉÕ”°¹Õ±°°€‰½¬‰ut°(€t¤ì(€…ÍÍ•ÉÐ¹‘••ÁÅÕ…°¡ÁÉ½©•Ñ	É½ÝÍ•ÉM…¹‘‰½áY…±Õ”¡Ù…±Õ”°€ÄÀÈÐ¤°ì(€€€Ñ•áÐè€ìˆé…¹ÍÝ•ÈˆèÐÈ°‰¥Ñ•µÌˆémÑÉÕ”±¹Õ±°°‰½¬‰uôœ°(€€€©Í½¸èì€ˆé…¹ÍÝ•Èˆè€ÐÈ°¥Ñ•µÌèmÑÉÕ”°¹Õ±°°€‰½¬‰tô°(€ô¤ì(€…ÍÍ•ÉÐ¹Ñ¡É½ÝÌ (€€€€ ¤€ôøÁÉ½©•Ñ	É½ÝÍ•ÉM…¹‘‰½áY…±Õ”¡¹•Ü!Ñ…!…¹‘±” ‰ÉÕ¹Ñ¥µ”ˆ°€‰½Á…ÅÕ”ˆ°€Å¸¤°€ÄÀÈÐ¤°(€€€ì½‘”è€‰Í…¹‘‰½à½É•ÍÕ±Ðµ¹½¸µÑÉ…¹Í™•É…‰±”ˆô°(€€¤ì(€…ÍÍ•ÉÐ¹Ñ¡É½ÝÌ  ¤€ôøÁÉ½©•Ñ	É½ÝÍ•ÉM…¹‘‰½áY…±Õ” ‰…‰ˆ°€Ì¤°ì½‘”è€‰Í…¹‘‰½à½½ÕÑÁÕÐµ±¥µ¥Ðˆô¤ì)ô¤ì()Ñ•ÍÐ ‰Í½ÕÉ”½¹Ñ…¥¹Ì¹¼½É‘¥¹…ÉäÍ•ÍÍ¥½¸½È‰É½ÝÍ•È…ÕÑ¡½É¥Ñä™…±±‰…¬ˆ°…Íå¹Œ€ ¤€ôøì(€½¹ÍÐÍ½ÕÉ”€ô…Ý…¥ÐÉ•…‘¥±”¡¹•ÜUI0 ˆ¸½Á…­…•Ì½¡Ñ„½Í…¹‘‰½à¹©Ìˆ°¥µÁ½ÉÐ¹µ•Ñ„¹ÕÉ°¤°€‰ÕÑ˜àˆ¤ì(€™½È€¡½¹ÍÐ™½É‰¥‘‘•¸½˜l(€€€€œ‰•ÍÍ¥½¸½•Ù…°ˆœ°(€€€€œ‰•Ù…°ˆœ°(€€€€‰I==Pˆ°(€€€€‰¡½ÍÑ…±±Ìèˆ°(€€€€‰™¥±•ÍåÍÑ•´½É•…Ñ”ˆ°(€€€€‰¥¹‘•á•‘‘ˆˆ°(€€€€‰¡É½µ”¸ˆ°(€€€€‰‰É½ÝÍ•È¹‘½´ˆ°(€€€€‰‰É½­•Èˆ°(€t¤ì(€€€¥˜€¡™½É‰¥‘‘•¸€ôôô€‰¡½ÍÑ…±±Ìèˆ¤½¹Ñ¥¹Õ”ì(€€€…ÍÍ•ÉÐ¹•ÅÕ…°¡Í½ÕÉ”¹Ñ½1½Ý•É…Í” ¤¹¥¹±Õ‘•Ì¡™½É‰¥‘‘•¸¹Ñ½1½Ý•É…Í” ¤¤°™…±Í”°™½É‰¥‘‘•¸¤ì(€ô(€…ÍÍ•ÉÐ¹µ…Ñ ¡Í½ÕÉ”°€½¡½ÍÑ…±±Ìè=‰©•Ñp¹™É••é•p¡qíqõp¤½Ô¤ì(€…ÍÍ•ÉÐ¹µ…Ñ ¡Í½ÕÉ”°€½™¥±•ÍåÍÑ•µ!½ÍÐè¹Õ±°½Ô¤ì(€…ÍÍ•ÉÐ¹µ…Ñ ¡Í½ÕÉ”°€½M9	=a}Y1}QIP½Ô¤ì)ô¤ì(