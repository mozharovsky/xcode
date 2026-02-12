/**
 * Node.js wrapper for @xcodekit/xcode-wasm/node.
 *
 * Usage:
 *   const { XcodeProject, parse, build } = require("@xcodekit/xcode-wasm/node");
 */

const { readFileSync, writeFileSync } = require("fs");
const wasm = require("./xcode");

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

module.exports = { ...wasm, XcodeProject };
