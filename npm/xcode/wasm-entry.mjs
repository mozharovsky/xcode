/**
 * @xcodekit/xcode — universal entry (browsers, CF Workers, bundlers).
 *
 * Uses the WASM build directly. No native addon, no platform binaries.
 */

export { XcodeProject, build, parse, parseAndBuild } from "@xcodekit/xcode-wasm";
