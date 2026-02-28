/**
 * @xcodekit/xcode — Node.js/Bun entry.
 *
 * Tries native (@xcodekit/xcode-node) first, falls back to WASM.
 */

import { createRequire } from "module";

const require = createRequire(import.meta.url);

let mod;
try {
  mod = require("@xcodekit/xcode-node");
} catch {
  mod = await import("@xcodekit/xcode-wasm");
}

export const { parse, build, parseAndBuild, parsePlist, buildPlist, XcodeProject } = mod;
