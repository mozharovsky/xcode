/**
 * Type declarations for @xcodekit/xcode-wasm/node.
 * Extends the base WASM XcodeProject with filesystem methods.
 */

export { build, parse, parseAndBuild } from "./pkg/node/xcode";

import { XcodeProject as BaseXcodeProject } from "./pkg/node/xcode";

export declare class XcodeProject extends BaseXcodeProject {
  /** Open and parse a .pbxproj file from disk. */
  static open(filePath: string): XcodeProject;

  /** Parse a .pbxproj string (no file on disk needed). */
  static fromString(content: string): XcodeProject;

  /** The file path this project was opened from, or null if fromString/constructor. */
  get filePath(): string | null;

  /** Write the project back to its original file. */
  save(): void;
}
