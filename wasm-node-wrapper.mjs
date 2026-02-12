/**
 * ESM wrapper for @xcodekit/xcode-wasm/node.
 * Adds open() and save() methods using the filesystem.
 *
 * Usage:
 *   import { XcodeProject } from "@xcodekit/xcode-wasm/node";
 */

import { readFileSync, writeFileSync } from "fs";
import { createRequire } from "module";

const require = createRequire(import.meta.url);
const wasm = require("./xcode.js");

class XcodeProject extends wasm.XcodeProject {
  #filePath = null;

  static open(filePath) {
    const content = readFileSync(filePath, "utf8");
    const project = new XcodeProject(content);
    project.#filePath = filePath;
    return project;
  }

  static fromString(content) {
    return new XcodeProject(content);
  }

  get filePath() {
    return this.#filePath;
  }

  save() {
    if (!this.#filePath) throw new Error("No file path set — use open() or save(path)");
    writeFileSync(this.#filePath, this.toBuild());
  }
}

export { XcodeProject };
export const { parse, build, parseAndBuild } = wasm;
