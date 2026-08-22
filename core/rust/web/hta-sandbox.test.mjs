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
        calls.push([target, arguments_]);
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
    instance.run({ operation: "sandbox.eval", source: "(+ 40 2)" }),
    { code: "sandbox/not-reusable" },
  );
});
