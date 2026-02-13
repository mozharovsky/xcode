/**
 * WASM integration tests.
 * Run: npx vitest tests/wasm.test.mjs
 * Requires: make build-wasm
 */

import { cpSync, mkdtempSync, readFileSync } from "fs";
import { tmpdir } from "os";
import { dirname, join } from "path";
import { fileURLToPath } from "url";
import { describe, expect, test } from "vitest";

const __dirname = dirname(fileURLToPath(import.meta.url));
const FIXTURES_DIR = join(__dirname, "fixtures");

let wasm;
try {
  wasm = await import("../pkg/xcode-wasm/index.mjs");
} catch {
  wasm = null;
}

const roundTripFixtures = [
  "006-spm.pbxproj",
  "007-xcode16.pbxproj",
  "AFNetworking.pbxproj",
  "project.pbxproj",
  "project-rn74.pbxproj",
  "project-multitarget.pbxproj",
  "project-rni.pbxproj",
  "project-swift.pbxproj",
  "project-with-entitlements.pbxproj",
  "watch.pbxproj",
];

describe.skipIf(!wasm)("wasm", () => {
  test("parse returns a JS object", () => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const parsed = wasm.parse(text);
    expect(typeof parsed).toBe("object");
    expect(parsed.archiveVersion).toBe(1);
    expect(parsed.objects).toBeTruthy();
    expect(parsed.rootObject).toBeTruthy();
  });

  test("build round-trips", () => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const json = wasm.parse(text);
    const output = wasm.build(json);
    expect(output).toBe(text);
  });

  test("parseAndBuild round-trips", () => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const output = wasm.parseAndBuild(text);
    expect(output).toBe(text);
  });

  for (const fixture of roundTripFixtures) {
    test(`round-trip: ${fixture}`, () => {
      const text = readFileSync(join(FIXTURES_DIR, fixture), "utf8");
      const output = wasm.parseAndBuild(text);
      expect(output).toBe(text);
    });
  }

  test("XcodeProject.open() reads from disk", () => {
    const project = wasm.XcodeProject.open(join(FIXTURES_DIR, "project.pbxproj"));
    expect(project.filePath).toBeTruthy();
    expect(project.getNativeTargets().length > 0).toBe(true);
  });

  test("XcodeProject.fromString() has null filePath", () => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = wasm.XcodeProject.fromString(text);
    expect(project.filePath).toBe(null);
    expect(project.getNativeTargets().length > 0).toBe(true);
  });

  test("XcodeProject.open() round-trips", () => {
    const original = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = wasm.XcodeProject.open(join(FIXTURES_DIR, "project.pbxproj"));
    expect(project.toBuild()).toBe(original);
  });

  test("XcodeProject.save() persists changes", () => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-wasm-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = wasm.XcodeProject.open(pbxpath);
    const target = project.findMainAppTarget("ios");
    project.setBuildSetting(target, "WASM_SAVE_TEST", "works");
    project.save();

    const saved = readFileSync(pbxpath, "utf8");
    expect(saved.includes("WASM_SAVE_TEST")).toBe(true);
    expect(saved.includes("works")).toBe(true);
    expect(saved.startsWith("// !$*UTF8*$!")).toBe(true);
  });

  test("targets and build settings", () => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = wasm.XcodeProject.fromString(text);

    const targets = project.getNativeTargets();
    expect(targets.length > 0).toBe(true);

    const target = project.findMainAppTarget("ios");
    expect(target).toBeTruthy();
    expect(project.getTargetName(target)).toBeTruthy();

    project.setBuildSetting(target, "TEST_KEY", "TEST_VALUE");
    const output = project.toBuild();
    expect(output.includes("TEST_KEY")).toBe(true);
    expect(output.includes("TEST_VALUE")).toBe(true);
  });

  test("addFile + addBuildFile", () => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = wasm.XcodeProject.fromString(text);

    const mainGroup = project.mainGroupUuid;
    const fileUuid = project.addFile(mainGroup, "Sources/New.swift");
    expect(fileUuid).toBeTruthy();

    const target = project.findMainAppTarget("ios");
    const phase = project.ensureBuildPhase(target, "PBXSourcesBuildPhase");
    const buildFile = project.addBuildFile(phase, fileUuid);
    expect(buildFile).toBeTruthy();

    const output = project.toBuild();
    expect(output.includes("New.swift")).toBe(true);
    expect(output.includes("New.swift in Sources")).toBe(true);
  });

  test("createNativeTarget + embedExtension", () => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = wasm.XcodeProject.fromString(text);

    const host = project.findMainAppTarget("ios");
    const ext = project.createNativeTarget("Widget", "com.apple.product-type.app-extension", "com.example.widget");
    expect(ext).toBeTruthy();

    project.addDependency(host, ext);
    const phase = project.embedExtension(host, ext);
    expect(phase).toBeTruthy();

    const output = project.toBuild();
    expect(output.includes("Widget")).toBe(true);
    expect(output.includes("PBXCopyFilesBuildPhase")).toBe(true);
    expect(output.includes("Embed Foundation Extensions")).toBe(true);
  });

  test("renameTarget cascades", () => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = wasm.XcodeProject.fromString(text);

    const target = project.findMainAppTarget("ios");
    const oldName = project.getTargetName(target);

    project.renameTarget(target, oldName, "RenamedApp");
    expect(project.getTargetName(target)).toBe("RenamedApp");

    const output = project.toBuild();
    expect(output.includes("RenamedApp.app")).toBe(true);
    expect(output.includes("RenamedApp")).toBe(true);
  });

  test("generic property access", () => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = wasm.XcodeProject.fromString(text);

    const target = project.findMainAppTarget("ios");
    expect(project.getObjectProperty(target, "isa")).toBe("PBXNativeTarget");

    project.setObjectProperty(target, "productName", "Custom");
    expect(project.getObjectProperty(target, "productName")).toBe("Custom");
  });

  test("findObjectsByIsa", () => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = wasm.XcodeProject.fromString(text);

    const groups = project.findObjectsByIsa("PBXGroup");
    expect(groups.length > 0).toBe(true);

    const fileRefs = project.findObjectsByIsa("PBXFileReference");
    expect(fileRefs.length > 0).toBe(true);
  });

  test("re-exports parse/build/parseAndBuild", () => {
    expect(typeof wasm.parse).toBe("function");
    expect(typeof wasm.build).toBe("function");
    expect(typeof wasm.parseAndBuild).toBe("function");
  });
});
