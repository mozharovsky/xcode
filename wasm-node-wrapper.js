/**
 * CJS wrapper for @xcodekit/xcode-wasm/node.
 *
 * Usage:
 *   const { XcodeProject, parse, build } = require("@xcodekit/xcode-wasm/node");
 */

const { readFileSync, writeFileSync } = require("fs");
const wasm = require("./xcode");

class XcodeProject extends wasm.XcodeProject {
  /** @type {string | null} */
  #filePath = null;

  /**
   * Open and parse a .pbxproj file from disk.
   * @param {string} filePath
   * @returns {XcodeProject}
   */
  static open(filePath) {
    const content = readFileSync(filePath, "utf8");
    const project = new XcodeProject(content);
    project.#filePath = filePath;
    return project;
  }

  /**
   * Parse a .pbxproj string (no file on disk needed).
   * @param {string} content
   * @returns {XcodeProject}
   */
  static fromString(content) {
    return new XcodeProject(content);
  }

  /** The file path this project was opened from, or null if fromString/constructor. */
  get filePath() {
    return this.#filePath;
  }

  /** Write the project back to its original file. Throws if no file path is set. */
  save() {
    if (!this.#filePath) throw new Error("No file path set — use open() or save(path)");
    writeFileSync(this.#filePath, this.toBuild());
  }
}

module.exports = { ...wasm, XcodeProject };
