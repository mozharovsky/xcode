/**
 * Benchmark: @xcodekit/xcode (Rust) vs @bacons/xcode (TypeScript).
 *
 * Run: node benches/benchmark.mjs
 * Requires: npm run build (or make build) first
 * Requires: npm install @bacons/xcode
 */

import { readFileSync } from "fs";
import { createRequire } from "module";
import { dirname, join } from "path";
import { performance } from "perf_hooks";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

const rust = await import("../index.js");

let ts;
try {
  ts = require("@bacons/xcode/json");
} catch {
  console.warn("@bacons/xcode not installed — skipping TS comparison");
  console.warn("Run: npm install @bacons/xcode");
  ts = null;
}

const FIXTURES_DIR = join(__dirname, "..", "__test__", "fixtures");
const WARMUP = 10;
const ITERATIONS = 200;

// ── Helpers ────────────────────────────────────────────────────────

function median(arr) {
  const sorted = [...arr].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)];
}

function p95(arr) {
  const sorted = [...arr].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length * 0.95)];
}

function bench(fn) {
  for (let i = 0; i < WARMUP; i++) fn();
  const times = [];
  for (let i = 0; i < ITERATIONS; i++) {
    const start = performance.now();
    fn();
    times.push(performance.now() - start);
  }
  return { median: median(times), p95: p95(times) };
}

function fmt(ms) {
  return ms.toFixed(3).padStart(8);
}

function speedup(base, cmp) {
  const x = (base / cmp).toFixed(1);
  return cmp < base ? `${x}x faster` : `${(1 / (base / cmp)).toFixed(1)}x slower`;
}

// ── Fixtures ───────────────────────────────────────────────────────

const fixtures = [
  ["swift-protobuf.pbxproj", "257 KB"],
  ["Cocoa-Application.pbxproj", "166 KB"],
  ["AFNetworking.pbxproj", "99 KB"],
  ["watch.pbxproj", "48 KB"],
  ["project.pbxproj", "19 KB"],
];

// ── Run ────────────────────────────────────────────────────────────

console.log("=".repeat(78));
console.log(" @xcodekit/xcode (Rust) vs @bacons/xcode (TypeScript)");
console.log("=".repeat(78));
console.log(`Warmup: ${WARMUP}  Iterations: ${ITERATIONS}`);
console.log();

console.log("PARSE");
console.log("─".repeat(78));
console.log("  Fixture".padEnd(38) + "Rust".padStart(10) + (ts ? "TS".padStart(10) + "Speedup".padStart(18) : ""));
for (const [fixture, size] of fixtures) {
  const text = readFileSync(join(FIXTURES_DIR, fixture), "utf8");
  const r = bench(() => rust.parse(text));
  const t = ts ? bench(() => ts.parse(text)) : null;
  const label = `  ${fixture} (${size})`.padEnd(38);
  const rStr = fmt(r.median) + " ms";
  const tStr = t ? fmt(t.median) + " ms" : "";
  const sStr = t ? speedup(t.median, r.median).padStart(14) : "";
  console.log(label + rStr + (t ? tStr + sStr : ""));
}

console.log();
console.log("BUILD (from pre-parsed object)");
console.log("─".repeat(78));
console.log("  Fixture".padEnd(38) + "Rust".padStart(10) + (ts ? "TS".padStart(10) + "Speedup".padStart(18) : ""));
for (const [fixture, size] of fixtures) {
  const text = readFileSync(join(FIXTURES_DIR, fixture), "utf8");
  const rustParsed = rust.parse(text);
  const tsParsed = ts ? ts.parse(text) : null;
  const r = bench(() => rust.build(rustParsed));
  const t = ts ? bench(() => ts.build(tsParsed)) : null;
  const label = `  ${fixture} (${size})`.padEnd(38);
  const rStr = fmt(r.median) + " ms";
  const tStr = t ? fmt(t.median) + " ms" : "";
  const sStr = t ? speedup(t.median, r.median).padStart(14) : "";
  console.log(label + rStr + (t ? tStr + sStr : ""));
}

console.log();
console.log("BUILD FROM JSON (buildFromJSON — avoids napi object marshalling)");
console.log("─".repeat(78));
console.log("  Fixture".padEnd(38) + "Rust".padStart(10) + (ts ? "TS".padStart(10) + "Speedup".padStart(18) : ""));
for (const [fixture, size] of fixtures) {
  const text = readFileSync(join(FIXTURES_DIR, fixture), "utf8");
  const rustParsed = rust.parse(text);
  const tsParsed = ts ? ts.parse(text) : null;
  const jsonStr = JSON.stringify(rustParsed);
  const r = bench(() => rust.buildFromJSON(jsonStr));
  const t = ts ? bench(() => ts.build(tsParsed)) : null;
  const label = `  ${fixture} (${size})`.padEnd(38);
  const rStr = fmt(r.median) + " ms";
  const tStr = t ? fmt(t.median) + " ms" : "";
  const sStr = t ? speedup(t.median, r.median).padStart(14) : "";
  console.log(label + rStr + (t ? tStr + sStr : ""));
}

console.log();
console.log("ROUND-TRIP (parse + build)");
console.log("─".repeat(78));
console.log("  Fixture".padEnd(38) + "Rust".padStart(10) + (ts ? "TS".padStart(10) + "Speedup".padStart(18) : ""));
for (const [fixture, size] of fixtures) {
  const text = readFileSync(join(FIXTURES_DIR, fixture), "utf8");
  const r = bench(() => rust.build(rust.parse(text)));
  const t = ts ? bench(() => ts.build(ts.parse(text))) : null;
  const label = `  ${fixture} (${size})`.padEnd(38);
  const rStr = fmt(r.median) + " ms";
  const tStr = t ? fmt(t.median) + " ms" : "";
  const sStr = t ? speedup(t.median, r.median).padStart(14) : "";
  console.log(label + rStr + (t ? tStr + sStr : ""));
}

console.log();
console.log("PARSE AND BUILD (parseAndBuild — zero JS↔Rust marshalling)");
console.log("─".repeat(78));
console.log("  Fixture".padEnd(38) + "Rust".padStart(10) + (ts ? "TS".padStart(10) + "Speedup".padStart(18) : ""));
for (const [fixture, size] of fixtures) {
  const text = readFileSync(join(FIXTURES_DIR, fixture), "utf8");
  const r = bench(() => rust.parseAndBuild(text));
  const t = ts ? bench(() => ts.build(ts.parse(text))) : null;
  const label = `  ${fixture} (${size})`.padEnd(38);
  const rStr = fmt(r.median) + " ms";
  const tStr = t ? fmt(t.median) + " ms" : "";
  const sStr = t ? speedup(t.median, r.median).padStart(14) : "";
  console.log(label + rStr + (t ? tStr + sStr : ""));
}

console.log();
console.log("XCODE PROJECT (open + toJSON)");
console.log("─".repeat(78));
{
  const bigFile = join(FIXTURES_DIR, "swift-protobuf.pbxproj");
  const r = bench(() => {
    const p = rust.XcodeProject.open(bigFile);
    p.toJSON();
  });
  console.log(`  XcodeProject.open() + toJSON():  ${fmt(r.median)} ms (median)  ${fmt(r.p95)} ms (p95)`);
}

console.log();
console.log("=".repeat(78));
