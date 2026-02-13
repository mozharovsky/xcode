/**
 * Native napi integration tests.
 * Run: npx vitest tests/node.test.mjs
 * Requires: npx napi build --platform
 */

import { cpSync, mkdtempSync, readFileSync } from "fs";
import { tmpdir } from "os";
import { dirname, join } from "path";
import { fileURLToPath } from "url";
import { describe, expect, test } from "vitest";

const __dirname = dirname(fileURLToPath(import.meta.url));

let native;
try {
  native = await import("../index.js");
} catch {
  native = null;
}

const FIXTURES_DIR = join(__dirname, "fixtures");

const fixtures = [
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

describe.skipIf(!native)("native napi", () => {
  test("parse() returns an object", () => {
    const input = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const result = native.parse(input);
    expect(result).toBeTruthy();
    expect(typeof result).toBe("object");
    expect(result.archiveVersion).toBeTruthy();
    expect(result.objectVersion).toBeTruthy();
    expect(result.objects).toBeTruthy();
  });

  test("build() produces valid pbxproj output", () => {
    const input = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const parsed = native.parse(input);
    const output = native.build(parsed);
    expect(typeof output).toBe("string");
    expect(output.startsWith("// !$*UTF8*$!")).toBe(true);
    expect(output.includes("archiveVersion")).toBe(true);
  });

  for (const fixture of fixtures) {
    test(`round-trip: ${fixture}`, () => {
      const original = readFileSync(join(FIXTURES_DIR, fixture), "utf8");
      const parsed = native.parse(original);
      const output = native.build(parsed);
      expect(output).toBe(original);
    });
  }

  test("parse() handles escape sequences", () => {
    const input = '{ key = "hello\\nworld"; }';
    const result = native.parse(input);
    expect(result.key).toBe("hello\nworld");
  });

  test("parse() preserves numeric types", () => {
    const input = "{ version = 46; octal = 0755; }";
    const result = native.parse(input);
    expect(result.version).toBe(46);
    expect(result.octal).toBe("0755");
  });

  test("XcodeProject.open() works", () => {
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "project.pbxproj"));
    expect(project).toBeTruthy();
    const json = project.toJSON();
    expect(json).toBeTruthy();
    expect(json.objects).toBeTruthy();
  });

  test("XcodeProject.toBuild() round-trips", () => {
    const original = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "project.pbxproj"));
    const output = project.toBuild();
    expect(output).toBe(original);
  });

  test("XcodeProject properties", () => {
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "project.pbxproj"));
    expect(project.archiveVersion).toBe(1);
    expect(project.objectVersion).toBe(46);
    expect(project.filePath).toBeTruthy();
  });

  test("XcodeProject.getNativeTargets() returns UUIDs", () => {
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "project.pbxproj"));
    const targets = project.getNativeTargets();
    expect(Array.isArray(targets)).toBe(true);
    expect(targets.length > 0).toBe(true);
    for (const uuid of targets) {
      expect(typeof uuid).toBe("string");
      expect(uuid.length).toBe(24);
    }
  });

  test("XcodeProject.findMainAppTarget()", () => {
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "project.pbxproj"));
    const targetUuid = project.findMainAppTarget("ios");
    expect(targetUuid).toBeTruthy();
    expect(typeof targetUuid).toBe("string");
  });

  test("XcodeProject.getUniqueId() is deterministic", () => {
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "project.pbxproj"));
    const id1 = project.getUniqueId("test-seed");
    const id2 = project.getUniqueId("test-seed");
    expect(id1).toBe(id2);
    expect(id1.length).toBe(24);
  });

  test("parseAndBuild() round-trips", () => {
    const original = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const output = native.parseAndBuild(original);
    expect(output).toBe(original);
  });

  test("buildFromJSON() round-trips", () => {
    const original = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const parsed = native.parse(original);
    const output = native.buildFromJSON(JSON.stringify(parsed));
    expect(output).toBe(original);
  });

  test("clean project has no orphaned references", () => {
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "project.pbxproj"));
    const orphans = project.findOrphanedReferences();
    expect(orphans.length).toBe(0);
  });

  test("setBuildSetting modifies and persists code signing settings", () => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const target = project.findMainAppTarget("ios");
    expect(target).toBeTruthy();

    project.setBuildSetting(target, "CODE_SIGN_STYLE", "Manual");
    project.setBuildSetting(target, "CODE_SIGN_IDENTITY", "Apple Distribution");
    project.setBuildSetting(target, "DEVELOPMENT_TEAM", "ABCDE12345");
    project.setBuildSetting(target, "PROVISIONING_PROFILE_SPECIFIER", "MyApp_Profile");
    project.save();

    const reopened = native.XcodeProject.open(pbxpath);
    const target2 = reopened.findMainAppTarget("ios");

    expect(reopened.getBuildSetting(target2, "CODE_SIGN_STYLE")).toBe("Manual");
    expect(reopened.getBuildSetting(target2, "CODE_SIGN_IDENTITY")).toBe("Apple Distribution");
    expect(reopened.getBuildSetting(target2, "DEVELOPMENT_TEAM")).toBe("ABCDE12345");
    expect(reopened.getBuildSetting(target2, "PROVISIONING_PROFILE_SPECIFIER")).toBe("MyApp_Profile");

    const content = readFileSync(pbxpath, "utf8");
    expect(content.startsWith("// !$*UTF8*$!")).toBe(true);
    expect(content.includes("CODE_SIGN_STYLE = Manual")).toBe(true);
    expect(content.includes("PROVISIONING_PROFILE_SPECIFIER = MyApp_Profile")).toBe(true);
  });

  test("malformed project detects orphaned references", () => {
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "malformed.pbxproj"));
    const orphans = project.findOrphanedReferences();
    expect(orphans.length > 0).toBe(true);

    const known = orphans.find((o) => o.orphanUuid === "3E1C2299F05049539341855D");
    expect(known).toBeTruthy();
    expect(known.referrerIsa).toBe("PBXResourcesBuildPhase");
    expect(known.property).toBe("files");
  });

  test("malformed project still parses and serializes", () => {
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "malformed.pbxproj"));
    expect(project.toJSON()).toBeTruthy();
    const output = project.toBuild();
    expect(output.includes("PBXResourcesBuildPhase")).toBe(true);
    expect(output.includes("baconwidget")).toBe(true);
  });

  test("addFile creates a file reference in a group", () => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const mainGroup = project.mainGroupUuid;
    expect(mainGroup).toBeTruthy();

    const fileUuid = project.addFile(mainGroup, "Sources/NewFile.swift");
    expect(fileUuid).toBeTruthy();
    expect(fileUuid.length).toBe(24);

    const children = project.getGroupChildren(mainGroup);
    expect(children.includes(fileUuid)).toBe(true);

    const output = project.toBuild();
    expect(output.includes("NewFile.swift")).toBe(true);
    expect(output.includes("sourcecode.swift")).toBe(true);
  });

  test("addGroup creates a nested group", () => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const mainGroup = project.mainGroupUuid;

    const groupUuid = project.addGroup(mainGroup, "NewFeature");
    expect(groupUuid).toBeTruthy();

    const children = project.getGroupChildren(mainGroup);
    expect(children.includes(groupUuid)).toBe(true);

    const output = project.toBuild();
    expect(output.includes("NewFeature")).toBe(true);
  });

  test("addFramework adds a framework to a target", () => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const target = project.findMainAppTarget("ios");

    const buildFileUuid = project.addFramework(target, "SwiftUI");
    expect(buildFileUuid).toBeTruthy();

    const output = project.toBuild();
    expect(output.includes("SwiftUI.framework")).toBe(true);
    expect(output.includes("wrapper.framework")).toBe(true);
  });

  test("addDependency links two targets", () => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project-multitarget.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const targets = project.getNativeTargets();
    expect(targets.length >= 2).toBe(true);

    const depUuid = project.addDependency(targets[0], targets[1]);
    expect(depUuid).toBeTruthy();

    const output = project.toBuild();
    expect(output.includes("PBXTargetDependency")).toBe(true);
    expect(output.includes("PBXContainerItemProxy")).toBe(true);
  });

  test("createNativeTarget creates a complete target", () => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const beforeTargets = project.getNativeTargets();

    const targetUuid = project.createNativeTarget(
      "MyWidget",
      "com.apple.product-type.app-extension",
      "com.example.mywidget",
    );
    expect(targetUuid).toBeTruthy();

    const afterTargets = project.getNativeTargets();
    expect(afterTargets.length).toBe(beforeTargets.length + 1);
    expect(afterTargets.includes(targetUuid)).toBe(true);

    project.setBuildSetting(targetUuid, "IPHONEOS_DEPLOYMENT_TARGET", "16.0");
    project.save();

    const reopened = native.XcodeProject.open(pbxpath);
    const output = reopened.toBuild();
    expect(output.includes("MyWidget")).toBe(true);
    expect(output.includes("com.example.mywidget")).toBe(true);
    expect(output.includes("com.apple.product-type.app-extension")).toBe(true);
    expect(output.includes("PBXSourcesBuildPhase")).toBe(true);
    expect(output.includes("PBXFrameworksBuildPhase")).toBe(true);
    expect(output.includes("PBXResourcesBuildPhase")).toBe(true);
    expect(reopened.getBuildSetting(targetUuid, "IPHONEOS_DEPLOYMENT_TARGET")).toBe("16.0");
  });

  test("full workflow: add file + add to sources phase + save + reopen", () => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const target = project.findMainAppTarget("ios");
    const mainGroup = project.mainGroupUuid;

    const fileUuid = project.addFile(mainGroup, "Features/Login.swift");
    const sourcesPhase = project.ensureBuildPhase(target, "PBXSourcesBuildPhase");
    const buildFileUuid = project.addBuildFile(sourcesPhase, fileUuid);
    expect(buildFileUuid).toBeTruthy();

    project.save();
    const reopened = native.XcodeProject.open(pbxpath);
    const output = reopened.toBuild();
    expect(output.includes("Login.swift")).toBe(true);
    expect(output.includes("Login.swift in Sources")).toBe(true);
    expect(output.includes("sourcecode.swift")).toBe(true);
  });

  test("XcodeProject.fromString() parses without a file on disk", () => {
    const content = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = native.XcodeProject.fromString(content);

    expect(project.archiveVersion).toBe(1);
    expect(project.mainGroupUuid).toBeTruthy();
    expect(project.getNativeTargets().length > 0).toBe(true);
    expect(project.filePath).toBe(null);

    const target = project.findMainAppTarget("ios");
    expect(project.getBuildSetting(target, "PRODUCT_NAME")).toBeTruthy();

    const output = project.toBuild();
    expect(output).toBe(content);
  });

  test("getTargetName / setTargetName", () => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const target = project.findMainAppTarget("ios");

    const oldName = project.getTargetName(target);
    expect(oldName).toBeTruthy();

    project.setTargetName(target, "RenamedApp");
    expect(project.getTargetName(target)).toBe("RenamedApp");

    project.save();
    const output = readFileSync(pbxpath, "utf8");
    expect(output.includes("RenamedApp")).toBe(true);
  });

  test("getObjectProperty / setObjectProperty", () => {
    const content = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = native.XcodeProject.fromString(content);

    const target = project.findMainAppTarget("ios");
    expect(project.getObjectProperty(target, "isa")).toBe("PBXNativeTarget");
    expect(project.getObjectProperty(target, "name")).toBeTruthy();

    project.setObjectProperty(target, "productName", "CustomProduct");
    expect(project.getObjectProperty(target, "productName")).toBe("CustomProduct");
  });

  test("findObjectsByIsa", () => {
    const content = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = native.XcodeProject.fromString(content);

    const groups = project.findObjectsByIsa("PBXGroup");
    expect(groups.length > 0).toBe(true);
    for (const uuid of groups) {
      expect(project.getObjectProperty(uuid, "isa")).toBe("PBXGroup");
    }

    const fileRefs = project.findObjectsByIsa("PBXFileReference");
    expect(fileRefs.length > 0).toBe(true);
  });

  test("renameTarget cascades through project", () => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const target = project.findMainAppTarget("ios");
    const oldName = project.getTargetName(target);

    project.renameTarget(target, oldName, "BrandNewApp");
    expect(project.getTargetName(target)).toBe("BrandNewApp");

    project.save();
    const output = readFileSync(pbxpath, "utf8");
    expect(output.includes("BrandNewApp.app")).toBe(true);
    expect(output.includes(`${oldName}.app`)).toBe(false);
    expect(output.includes("BrandNewApp")).toBe(true);
    expect(output.startsWith("// !$*UTF8*$!")).toBe(true);
  });

  test("embedExtension wires copy files phase", () => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const hostTarget = project.findMainAppTarget("ios");

    const extTarget = project.createNativeTarget(
      "MyWidget",
      "com.apple.product-type.app-extension",
      "com.example.widget",
    );
    const phaseUuid = project.embedExtension(hostTarget, extTarget);
    expect(phaseUuid).toBeTruthy();

    project.save();
    const output = readFileSync(pbxpath, "utf8");
    expect(output.includes("PBXCopyFilesBuildPhase")).toBe(true);
    expect(output.includes("Embed Foundation Extensions")).toBe(true);
    expect(output.includes("dstSubfolderSpec = 13")).toBe(true);
  });

  test("addFileSystemSyncGroup (Xcode 16+)", () => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const target = project.findMainAppTarget("ios");

    const syncUuid = project.addFileSystemSyncGroup(target, "MyApp");
    expect(syncUuid).toBeTruthy();

    const mainChildren = project.getGroupChildren(project.mainGroupUuid);
    expect(mainChildren.includes(syncUuid)).toBe(true);

    project.save();
    const output = readFileSync(pbxpath, "utf8");
    expect(output.includes("PBXFileSystemSynchronizedRootGroup")).toBe(true);
    expect(output.includes("fileSystemSynchronizedGroups")).toBe(true);
  });
});
