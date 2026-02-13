/**
 * Benchmark: WASM /node wrapper vs raw WASM vs napi.
 *
 * Run: node benches/benchmark-wasm-node.mjs
 * Requires: make build-wasm && make build
 */

import { readFileSync } from "fs";
import { createRequire } from "module";
import { dirname, join } from "path";
import { performance } from "perf_hooks";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

const FIXTURES_DIR = join(__dirname, "..", "tests", "fixtures");
const WARMUP = 10;
const ITERATIONS = 300;

let napi, wasmRaw, wasmNode;

try {
  napi = await import("../index.js");
} catch {}
try {
  wasmRaw = require("../pkg/node");
} catch {}
try {
  wasmNode = require("../pkg/node/node-wrapper");
} catch {}

if (!wasmRaw && !wasmNode) {
  console.error("WASM not built. Run: make build-wasm");
  process.exit(1);
}

function median(arr) {
  const sorted = [...arr].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)];
}

function bench(fn) {
  for (let i = 0; i < WARMUP; i++) fn();
  const times = [];
  for (let i = 0; i < ITERATIONS; i++) {
    const start = performance.now();
    fn();
    times.push(performance.now() - start);
  }
  return median(times);
}

function fmt(ms) {
  return ms !== null ? ms.toFixed(3).padStart(8) + " ms" : "         -  ";
}

const fixtures = [
  ["swift-protobuf.pbxproj", "257 KB"],
  ["AFNetworking.pbxproj", "99 KB"],
  ["project.pbxproj", "19 KB"],
];

console.log("=".repeat(78));
console.log(" WASM /node wrapper overhead");
console.log("=".repeat(78));
console.log(`Warmup: ${WARMUP}  Iterations: ${ITERATIONS}`);
console.log();

// ── parse() ────────────────────────────────────────────────────────

console.log("parse()");
console.log("─".repeat(78));
console.log(
  "  Fixture".padEnd(32) +
    "WASM raw".padStart(12) +
    "WASM /node".padStart(12) +
    (napi ? "napi".padStart(12) : "") +
    "  overhead",
);

for (const [fixture, size] of fixtures) {
  const text = readFileSync(join(FIXTURES_DIR, fixture), "utf8");
  const r = wasmRaw ? bench(() => wasmRaw.parse(text)) : null;
  const n = wasmNode ? bench(() => wasmNode.parse(text)) : null;
  const p = napi ? bench(() => napi.parse(text)) : null;
  const overhead = r && n ? `+${((n / r - 1) * 100).toFixed(0)}%` : "";
  console.log(
    `  ${fixture} (${size})`.padEnd(32) + fmt(r) + fmt(n) + (p !== null ? fmt(p) : "") + overhead.padStart(10),
  );
}

// ── build() ────────────────────────────────────────────────────────

console.log();
console.log("build(object)");
console.log("─".repeat(78));
console.log(
  "  Fixture".padEnd(32) +
    "WASM raw".padStart(12) +
    "WASM /node".padStart(12) +
    (napi ? "napi".padStart(12) : "") +
    "  overhead",
);

for (const [fixture, size] of fixtures) {
  const text = readFileSync(join(FIXTURES_DIR, fixture), "utf8");

  // raw WASM: build takes JSON string
  const jsonStr = wasmRaw ? wasmRaw.parse(text) : null;
  const r = wasmRaw ? bench(() => wasmRaw.build(jsonStr)) : null;

  // /node wrapper: build takes object
  const obj = wasmNode ? wasmNode.parse(text) : null;
  const n = wasmNode ? bench(() => wasmNode.build(obj)) : null;

  // napi: build takes object
  const napiObj = napi ? napi.parse(text) : null;
  const p = napi ? bench(() => napi.build(napiObj)) : null;

  const overhead = r && n ? `+${((n / r - 1) * 100).toFixed(0)}%` : "";
  console.log(
    `  ${fixture} (${size})`.padEnd(32) + fmt(r) + fmt(n) + (p !== null ? fmt(p) : "") + overhead.padStart(10),
  );
}

// ── toJSON() ───────────────────────────────────────────────────────

console.log();
console.log("XcodeProject: toJSON()");
console.log("─".repeat(78));
console.log(
  "  Fixture".padEnd(32) +
    "WASM raw".padStart(12) +
    "WASM /node".padStart(12) +
    (napi ? "napi".padStart(12) : "") +
    "  overhead",
);

for (const [fixture, size] of fixtures) {
  const text = readFileSync(join(FIXTURES_DIR, fixture), "utf8");

  const rProj = wasmRaw ? new wasmRaw.XcodeProject(text) : null;
  const r = rProj ? bench(() => rProj.toJSON()) : null;

  const nProj = wasmNode ? wasmNode.XcodeProject.fromString(text) : null;
  const n = nProj ? bench(() => nProj.toJSON()) : null;

  const pProj = napi ? napi.XcodeProject.fromString(text) : null;
  const p = pProj ? bench(() => pProj.toJSON()) : null;

  const overhead = r && n ? `+${((n / r - 1) * 100).toFixed(0)}%` : "";
  console.log(
    `  ${fixture} (${size})`.padEnd(32) + fmt(r) + fmt(n) + (p !== null ? fmt(p) : "") + overhead.padStart(10),
  );
}

// ── XcodeProject lifecycle ─────────────────────────────────────────

console.log();
console.log("XcodeProject: fromString + setBuildSetting + toBuild");
console.log("─".repeat(78));
console.log(
  "  Fixture".padEnd(32) +
    "WASM raw".padStart(12) +
    "WASM /node".padStart(12) +
    (napi ? "napi".padStart(12) : "") +
    "  overhead",
);

for (const [fixture, size] of fixtures) {
  const text = readFileSync(join(FIXTURES_DIR, fixture), "utf8");

  const r = wasmRaw
    ? bench(() => {
        const p = new wasmRaw.XcodeProject(text);
        const t = p.findMainAppTarget("ios");
        if (t) p.setBuildSetting(t, "SWIFT_VERSION", "6.0");
        p.toBuild();
      })
    : null;

  const n = wasmNode
    ? bench(() => {
        const p = wasmNode.XcodeProject.fromString(text);
        const t = p.findMainAppTarget("ios");
        if (t) p.setBuildSetting(t, "SWIFT_VERSION", "6.0");
        p.toBuild();
      })
    : null;

  const p = napi
    ? bench(() => {
        const proj = napi.XcodeProject.fromString(text);
        const t = proj.findMainAppTarget("ios");
        if (t) proj.setBuildSetting(t, "SWIFT_VERSION", "6.0");
        proj.toBuild();
      })
    : null;

  const overhead = r && n ? `+${((n / r - 1) * 100).toFixed(0)}%` : "";
  console.log(
    `  ${fixture} (${size})`.padEnd(32) + fmt(r) + fmt(n) + (p !== null ? fmt(p) : "") + overhead.padStart(10),
  );
}

console.log();
console.log("=".repeat(78));
