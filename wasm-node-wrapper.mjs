/**
 * ESM wrapper for @xcodekit/xcode-wasm/node.
 *
 * Usage:
 *   import { XcodeProject, parse, build } from "@xcodekit/xcode-wasm/node";
 */

import { readFileSync, writeFileSync } from "fs";
import * as wasm from "./xcode.js";

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

export { XcodeProject };

/**
 * Parse a .pbxproj string into a JSON-compatible object.
 * @type {(text: string) => any}
 */
export const parse = wasm.parse;

/**
 * Serialize a JSON object back to .pbxproj format.
 * @type {(project: object) => string}
 */
export const build = wasm.build;

/**
 * Parse and immediately re-serialize. Stays in WASM, zero marshalling.
 * @type {(text: string) => string}
 */
export const parseAndBuild = wasm.parseAndBuild;
