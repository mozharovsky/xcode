/**
 * Benchmark: WASM vs napi vs TypeScript.
 *
 * Run: node benches/benchmark-wasm.mjs
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
const ITERATIONS = 200;

// Load all three implementations
let napi, wasm, ts;

try {
  napi = await import("../index.js");
} catch {
  console.warn("napi not built — skipping. Run: make build");
}

try {
  wasm = require("../pkg/node");
} catch {
  console.warn("WASM not built — skipping. Run: make build-wasm");
}

try {
  ts = require("@bacons/xcode/json");
} catch {
  console.warn("@bacons/xcode not installed — skipping TS comparison");
}

if (!napi && !wasm) {
  console.error("Nothing to benchmark. Build at least one target first.");
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
  return ms.toFixed(3).padStart(8);
}

function speedup(base, cmp) {
  if (!base || !cmp) return "".padStart(14);
  const x = base / cmp;
  return (x >= 1 ? `${x.toFixed(1)}x faster` : `${(1 / x).toFixed(1)}x slower`).padStart(14);
}

const fixtures = [
  ["swift-protobuf.pbxproj", "257 KB"],
  ["Cocoa-Application.pbxproj", "166 KB"],
  ["AFNetworking.pbxproj", "99 KB"],
  ["watch.pbxproj", "48 KB"],
  ["project.pbxproj", "19 KB"],
];

const header =
  "  Fixture".padEnd(38) +
  (wasm ? "WASM".padStart(10) : "") +
  (napi ? "napi".padStart(10) : "") +
  (ts ? "TS".padStart(10) : "") +
  (wasm && ts ? "  WASM vs TS".padStart(14) : "");

// ── PARSE ──────────────────────────────────────────────────────────

console.log("=".repeat(78));
console.log(" WASM vs napi vs TypeScript");
console.log("=".repeat(78));
console.log(`Warmup: ${WARMUP}  Iterations: ${ITERATIONS}`);
console.log();

console.log("PARSE");
console.log("─".repeat(78));
console.log(header);

for (const [fixture, size] of fixtures) {
  const text = readFileSync(join(FIXTURES_DIR, fixture), "utf8");
  const w = wasm ? bench(() => wasm.parse(text)) : null;
  const n = napi ? bench(() => napi.parse(text)) : null;
  const t = ts ? bench(() => ts.parse(text)) : null;

  let line = `  ${fixture} (${size})`.padEnd(38);
  if (w !== null) line += fmt(w) + " ms";
  if (n !== null) line += fmt(n) + " ms";
  if (t !== null) line += fmt(t) + " ms";
  if (w !== null && t !== null) line += speedup(t, w);
  console.log(line);
}

// ── BUILD ──────────────────────────────────────────────────────────

console.log();
console.log("BUILD (from pre-parsed data)");
console.log("─".repeat(78));
console.log(header);

for (const [fixture, size] of fixtures) {
  const text = readFileSync(join(FIXTURES_DIR, fixture), "utf8");
  // WASM: build from JSON string (includes JSON.stringify)
  const wasmParsed = wasm ? wasm.parse(text) : null;
  // napi: build from JS object
  const napiParsed = napi ? napi.parse(text) : null;
  // TS: build from JS object
  const tsParsed = ts ? ts.parse(text) : null;

  const w = wasm ? bench(() => wasm.build(wasmParsed)) : null;
  const n = napi ? bench(() => napi.build(napiParsed)) : null;
  const t = ts ? bench(() => ts.build(tsParsed)) : null;

  let line = `  ${fixture} (${size})`.padEnd(38);
  if (w !== null) line += fmt(w) + " ms";
  if (n !== null) line += fmt(n) + " ms";
  if (t !== null) line += fmt(t) + " ms";
  if (w !== null && t !== null) line += speedup(t, w);
  console.log(line);
}

// ── PARSE AND BUILD ────────────────────────────────────────────────

console.log();
console.log("PARSE AND BUILD (zero-crossing round-trip)");
console.log("─".repeat(78));
console.log(header);

for (const [fixture, size] of fixtures) {
  const text = readFileSync(join(FIXTURES_DIR, fixture), "utf8");
  const w = wasm ? bench(() => wasm.parseAndBuild(text)) : null;
  const n = napi ? bench(() => napi.parseAndBuild(text)) : null;
  const t = ts ? bench(() => ts.build(ts.parse(text))) : null;

  let line = `  ${fixture} (${size})`.padEnd(38);
  if (w !== null) line += fmt(w) + " ms";
  if (n !== null) line += fmt(n) + " ms";
  if (t !== null) line += fmt(t) + " ms";
  if (w !== null && t !== null) line += speedup(t, w);
  console.log(line);
}

// ── XcodeProject lifecycle ─────────────────────────────────────────

console.log();
console.log("XCODE PROJECT (fromString + setBuildSetting + toBuild)");
console.log("─".repeat(78));

const header2 = "  Fixture".padEnd(38) + (wasm ? "WASM".padStart(10) : "") + (napi ? "napi".padStart(10) : "");
console.log(header2);

for (const [fixture, size] of fixtures) {
  const text = readFileSync(join(FIXTURES_DIR, fixture), "utf8");

  const w = wasm
    ? bench(() => {
        const p = new wasm.XcodeProject(text);
        const t = p.findMainAppTarget("ios");
        if (t) p.setBuildSetting(t, "SWIFT_VERSION", "6.0");
        p.toBuild();
      })
    : null;

  const n = napi
    ? bench(() => {
        const p = napi.XcodeProject.fromString(text);
        const t = p.findMainAppTarget("ios");
        if (t) p.setBuildSetting(t, "SWIFT_VERSION", "6.0");
        p.toBuild();
      })
    : null;

  let line = `  ${fixture} (${size})`.padEnd(38);
  if (w !== null) line += fmt(w) + " ms";
  if (n !== null) line += fmt(n) + " ms";
  if (w !== null && n !== null) line += speedup(n, w).padStart(14);
  console.log(line);
}

console.log();
console.log("=".repeat(78));
