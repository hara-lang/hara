import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { formatGitHubOutput, normalizeReleaseCut } from "./release-cut.mjs";

const valid = Object.freeze({
  schema: "hara-release-cut/v1",
  tag: "v0.1.2",
  version: "0.1.2",
  commit: "08322990fd1a26e3a004e7bf3459f45a25158311",
  workflow: "release.yml"
});

test("a reviewed release cut is normalized without changing its target", () => {
  assert.deepEqual(normalizeReleaseCut(valid), valid);
});

test("tag and version must identify the same immutable release", () => {
  assert.throws(
    () => normalizeReleaseCut({ ...valid, tag: "v0.1.3" }),
    /does not match version 0\.1\.2/
  );
  assert.throws(
    () => normalizeReleaseCut({ ...valid, version: "latest" }),
    /version is invalid/
  );
});

test("release cuts require a full commit rather than a movable branch", () => {
  for (const commit of ["main", "08322990", "g".repeat(40), ""]) {
    assert.throws(
      () => normalizeReleaseCut({ ...valid, commit }),
      /full 40-character SHA-1/
    );
  }
});

test("the workflow must be a top-level YAML file", () => {
  for (const workflow of ["../release.yml", ".github/workflows/release.yml", "release", "release.json"]) {
    assert.throws(
      () => normalizeReleaseCut({ ...valid, workflow }),
      /workflow is invalid/
    );
  }
});

test("unknown manifest fields are rejected", () => {
  assert.throws(
    () => normalizeReleaseCut({ ...valid, force: true }),
    /unknown fields: force/
  );
});

test("GitHub output contains only validated scalar fields", () => {
  assert.equal(formatGitHubOutput(valid), [
    "schema=hara-release-cut/v1",
    "tag=v0.1.2",
    "version=0.1.2",
    "commit=08322990fd1a26e3a004e7bf3459f45a25158311",
    "workflow=release.yml",
    ""
  ].join("\n"));
});

test("Truffle release tooling resolves paths from the repository root", async () => {
  const script = await readFile(new URL("./build-truffle-native", import.meta.url), "utf8");
  assert.match(
    script,
    /HARA_REPOSITORY_ROOT:-\$\(cd "\$\(dirname "\$\{BASH_SOURCE\[0\]\}"\)\/\.\.\/\.\."/,
  );
  assert.match(script, /core\/java\/pom\.xml/);
  assert.doesNotMatch(
    script,
    /ROOT="\$\(cd "\$\(dirname "\$\{BASH_SOURCE\[0\]\}"\)\/\.\." && pwd\)"/,
  );
});

test("the immutable v0.1.5 rerun repairs only release tooling and isolates tap credentials", async () => {
  const workflow = await readFile(
    new URL("../../.github/workflows/release.yml", import.meta.url),
    "utf8",
  );
  assert.match(workflow, /Repair immutable v0\.1\.5 release tooling/);
  assert.match(workflow, /needs\.guard\.outputs\.tag == 'v0\.1\.5'/);
  assert.match(workflow, /publish-homebrew:\n[\s\S]*?continue-on-error: true/);
  assert.match(workflow, /publish-source-formula:\n[\s\S]*?continue-on-error: true/);
  assert.match(workflow, /ref: \$\{\{ needs\.guard\.outputs\.tag \}\}/);
});
