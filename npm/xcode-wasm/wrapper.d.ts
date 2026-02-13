/**
 * Type declarations for @xcodekit/xcode-wasm/node.
 */

import { XcodeProject as BaseXcodeProject } from "./xcode";

/** Parse a .pbxproj string into a JSON-compatible object. */
export declare function parse(text: string): any;

/** Serialize a JSON object back to .pbxproj format. */
export declare function build(project: object): string;

/** Parse and immediately re-serialize. Stays in WASM, zero marshalling. */
export declare function parseAndBuild(text: string): string;

export declare class XcodeProject extends BaseXcodeProject {
  /** Open and parse a .pbxproj file from disk. */
  static open(filePath: string): XcodeProject;

  /** Parse a .pbxproj string (no file on disk needed). */
  static fromString(content: string): XcodeProject;

  /** The file path this project was opened from, or null if fromString/constructor. */
  get filePath(): string | null;

  /** Write the project back to its original file. */
  save(): void;

  /** Convert the project to a JSON-compatible object. */
  toJSON(): any;
}
