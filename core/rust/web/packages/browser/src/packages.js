import { parseEDNString } from "edn-data";
import { unzipSync } from "fflate";

const ednOptions = {
  mapAs: "object",
  setAs: "array",
  listAs: "array",
  keywordAs: "string",
  charAs: "string",
  objectKeysAs: "string"
};

const textDecoder = new TextDecoder();

function parseEdn(source) {
  return parseEDNString(String(source), ednOptions);
}

function hex(bytes) {
  return [...bytes]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

async function sha256(bytes) {
  const digest = await globalThis.crypto.subtle.digest("SHA-256", bytes);
  return `sha256:${hex(new Uint8Array(digest))}`;
}

const defaultPackagesOrigin = "https://packages.hara-lang.org";

function safeArchivePath(path) {
  return path
    && !path.startsWith("/")
    && !path.includes("\\")
    && path.split("/").every((part) => part && part !== "." && part !== "..");
}

/**
 * Downloads and verifies every HARP archive through the commit-pinned Packages
 * registry. Nothing is registered until all packages verify.
 */
export async function loadLockedPackageResources(
  lockSource,
  request = (...args) => globalThis.fetch(...args),
  origin = defaultPackagesOrigin
) {
  const lock = parseEdn(lockSource);
  if (lock["lock/format"] !== "0.0.0-alpha") {
    throw new Error("project.lock.edn requires :lock/format \"0.0.0-alpha\"");
  }

  const staged = {};
  for (const [coordinate, entry] of Object.entries(lock.packages ?? {})) {
    const registryCommit = entry["registry-commit"];
    const identityRevision = entry["identity-revision"];
    const digest = entry["archive-sha256"];
    const version = entry.version;
    if (!/^[0-9a-f]{40}$/.test(registryCommit ?? "")
        || !/^[0-9a-f]{40}$/.test(identityRevision ?? "")
        || !/^sha256:[0-9a-f]{64}$/.test(digest ?? "")
        || typeof version !== "string") {
      throw new Error(`Locked package ${coordinate} has an incomplete exact descriptor`);
    }
    const base = String(origin).replace(/\/$/, "");
    const registryResponse = await request(`${base}/v1/registry?ref=${registryCommit}`);
    if (!registryResponse.ok) {
      throw new Error(`Locked package ${coordinate} registry failed: ${registryResponse.status}`);
    }
    const registry = parseEdn(await registryResponse.text());
    const release = registry["registry/packages"]?.[coordinate]?.[version];
    if (release?.["archive-sha256"] !== digest
        || release?.["identity-revision"] !== identityRevision) {
      throw new Error(`Locked package ${coordinate} registry mismatch`);
    }
    const response = await request(`${base}/objects/sha256/${digest.slice(7)}`);
    if (!response.ok) {
      throw new Error(`Locked package ${coordinate} failed: ${response.status}`);
    }
    const archive = new Uint8Array(await response.arrayBuffer());
    if (entry.size !== undefined && archive.byteLength !== entry.size) {
      throw new Error(`Locked package ${coordinate} size mismatch`);
    }
    if (await sha256(archive) !== digest) {
      throw new Error(`Locked package ${coordinate} digest mismatch`);
    }

    const files = unzipSync(archive);
    if (!files["package.edn"]) {
      throw new Error(`Locked package ${coordinate} has no package.edn`);
    }
    for (const path of Object.keys(files)) {
      if (!safeArchivePath(path)) {
        throw new Error(`Locked package ${coordinate} contains an unsafe path`);
      }
    }

    const manifest = parseEdn(textDecoder.decode(files["package.edn"]));
    for (const [path, file] of Object.entries(manifest.files ?? {})) {
      const bytes = files[path];
      if (!bytes) {
        throw new Error(`Locked package ${coordinate} is missing ${path}`);
      }
      if (file.size !== bytes.byteLength || await sha256(bytes) !== file.sha256) {
        throw new Error(`Locked package ${coordinate} failed file verification: ${path}`);
      }
    }
    for (const [namespace, path] of Object.entries(manifest.resources ?? {})) {
      if (Object.hasOwn(staged, namespace)) {
        throw new Error(`Duplicate locked HAL namespace: ${namespace}`);
      }
      const bytes = files[path];
      if (!bytes) {
        throw new Error(`Locked package ${coordinate} is missing resource ${path}`);
      }
      staged[namespace] = textDecoder.decode(bytes);
    }
  }
  return staged;
}

/** Verifies a lock completely, then atomically exposes its HAL resources. */
export async function installLockedPackages(runtime, lockSource, options = {}) {
  runtime.raw?.registerPackageLock?.(lockSource);
  const resources = await loadLockedPackageResources(
    lockSource,
    options.fetch,
    options.origin ?? defaultPackagesOrigin
  );
  for (const [namespace, source] of Object.entries(resources)) {
    runtime.registerResource(namespace, source);
  }
  return Object.keys(resources);
}
