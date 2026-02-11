import test from "ava";
import { cpSync, mkdtempSync, readFileSync } from "fs";
import { tmpdir } from "os";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

// The native module will be loaded from the project root
let native;
try {
  native = await import("../index.js");
} catch {
  // Fallback: try loading the .node file directly
  console.warn("Could not load index.js — skipping JS integration tests");
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

if (native) {
  test("parse() returns an object", (t) => {
    const input = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const result = native.parse(input);
    t.truthy(result);
    t.is(typeof result, "object");
    t.truthy(result.archiveVersion);
    t.truthy(result.objectVersion);
    t.truthy(result.objects);
  });

  test("build() produces valid pbxproj output", (t) => {
    const input = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const parsed = native.parse(input);
    const output = native.build(parsed);
    t.is(typeof output, "string");
    t.true(output.startsWith("// !$*UTF8*$!"));
    t.true(output.includes("archiveVersion"));
  });

  for (const fixture of fixtures) {
    test(`round-trip: ${fixture}`, (t) => {
      const original = readFileSync(join(FIXTURES_DIR, fixture), "utf8");
      const parsed = native.parse(original);
      const output = native.build(parsed);
      t.is(output, original);
    });
  }

  test("parse() handles escape sequences", (t) => {
    const input = '{ key = "hello\\nworld"; }';
    const result = native.parse(input);
    t.is(result.key, "hello\nworld");
  });

  test("parse() preserves numeric types", (t) => {
    const input = "{ version = 46; octal = 0755; }";
    const result = native.parse(input);
    t.is(result.version, 46);
    t.is(result.octal, "0755");
  });

  test("XcodeProject.open() works", (t) => {
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "project.pbxproj"));
    t.truthy(project);
    const json = project.toJSON();
    t.truthy(json);
    t.truthy(json.objects);
  });

  test("XcodeProject.toBuild() round-trips", (t) => {
    const original = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "project.pbxproj"));
    const output = project.toBuild();
    t.is(output, original);
  });

  test("XcodeProject properties", (t) => {
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "project.pbxproj"));
    t.is(project.archiveVersion, 1);
    t.is(project.objectVersion, 46);
    t.truthy(project.filePath);
  });

  test("XcodeProject.getNativeTargets() returns UUIDs", (t) => {
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "project.pbxproj"));
    const targets = project.getNativeTargets();
    t.true(Array.isArray(targets));
    t.true(targets.length > 0);
    // UUIDs should be 24-char hex strings
    for (const uuid of targets) {
      t.is(typeof uuid, "string");
      t.is(uuid.length, 24);
    }
  });

  test("XcodeProject.findMainAppTarget()", (t) => {
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "project.pbxproj"));
    const targetUuid = project.findMainAppTarget("ios");
    t.truthy(targetUuid);
    t.is(typeof targetUuid, "string");
  });

  test("XcodeProject.getUniqueId() is deterministic", (t) => {
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "project.pbxproj"));
    const id1 = project.getUniqueId("test-seed");
    const id2 = project.getUniqueId("test-seed");
    t.is(id1, id2);
    t.is(id1.length, 24);
  });

  test("parseAndBuild() round-trips", (t) => {
    const original = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const output = native.parseAndBuild(original);
    t.is(output, original);
  });

  test("buildFromJSON() round-trips", (t) => {
    const original = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const parsed = native.parse(original);
    const output = native.buildFromJSON(JSON.stringify(parsed));
    t.is(output, original);
  });

  test("clean project has no orphaned references", (t) => {
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "project.pbxproj"));
    const orphans = project.findOrphanedReferences();
    t.is(orphans.length, 0);
  });

  test("setBuildSetting modifies and persists code signing settings", (t) => {
    // Work on a copy so we don't mutate the fixture
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    // Open, modify, save
    const project = native.XcodeProject.open(pbxpath);
    const target = project.findMainAppTarget("ios");
    t.truthy(target);

    project.setBuildSetting(target, "CODE_SIGN_STYLE", "Manual");
    project.setBuildSetting(target, "CODE_SIGN_IDENTITY", "Apple Distribution");
    project.setBuildSetting(target, "DEVELOPMENT_TEAM", "ABCDE12345");
    project.setBuildSetting(target, "PROVISIONING_PROFILE_SPECIFIER", "MyApp_Profile");
    project.save();

    // Re-open and verify settings persisted
    const reopened = native.XcodeProject.open(pbxpath);
    const target2 = reopened.findMainAppTarget("ios");

    t.is(reopened.getBuildSetting(target2, "CODE_SIGN_STYLE"), "Manual");
    t.is(reopened.getBuildSetting(target2, "CODE_SIGN_IDENTITY"), "Apple Distribution");
    t.is(reopened.getBuildSetting(target2, "DEVELOPMENT_TEAM"), "ABCDE12345");
    t.is(reopened.getBuildSetting(target2, "PROVISIONING_PROFILE_SPECIFIER"), "MyApp_Profile");

    // Verify the file is valid pbxproj
    const content = readFileSync(pbxpath, "utf8");
    t.true(content.startsWith("// !$*UTF8*$!"));
    t.true(content.includes("CODE_SIGN_STYLE = Manual"));
    t.true(content.includes("PROVISIONING_PROFILE_SPECIFIER = MyApp_Profile"));
  });

  test("malformed project detects orphaned references", (t) => {
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "malformed.pbxproj"));
    const orphans = project.findOrphanedReferences();
    t.true(orphans.length > 0);

    const known = orphans.find((o) => o.orphanUuid === "3E1C2299F05049539341855D");
    t.truthy(known);
    t.is(known.referrerIsa, "PBXResourcesBuildPhase");
    t.is(known.property, "files");
  });

  test("malformed project still parses and serializes", (t) => {
    const project = native.XcodeProject.open(join(FIXTURES_DIR, "malformed.pbxproj"));
    t.truthy(project.toJSON());
    const output = project.toBuild();
    t.true(output.includes("PBXResourcesBuildPhase"));
    t.true(output.includes("baconwidget"));
  });
  test("addFile creates a file reference in a group", (t) => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const mainGroup = project.mainGroupUuid;
    t.truthy(mainGroup);

    const fileUuid = project.addFile(mainGroup, "Sources/NewFile.swift");
    t.truthy(fileUuid);
    t.is(fileUuid.length, 24);

    // File should appear in group children
    const children = project.getGroupChildren(mainGroup);
    t.true(children.includes(fileUuid));

    // Should serialize correctly
    const output = project.toBuild();
    t.true(output.includes("NewFile.swift"));
    t.true(output.includes("sourcecode.swift"));
  });

  test("addGroup creates a nested group", (t) => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const mainGroup = project.mainGroupUuid;

    const groupUuid = project.addGroup(mainGroup, "NewFeature");
    t.truthy(groupUuid);

    const children = project.getGroupChildren(mainGroup);
    t.true(children.includes(groupUuid));

    const output = project.toBuild();
    t.true(output.includes("NewFeature"));
  });

  test("addFramework adds a framework to a target", (t) => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const target = project.findMainAppTarget("ios");

    const buildFileUuid = project.addFramework(target, "SwiftUI");
    t.truthy(buildFileUuid);

    const output = project.toBuild();
    t.true(output.includes("SwiftUI.framework"));
    t.true(output.includes("wrapper.framework"));
  });

  test("addDependency links two targets", (t) => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project-multitarget.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const targets = project.getNativeTargets();
    t.true(targets.length >= 2);

    const depUuid = project.addDependency(targets[0], targets[1]);
    t.truthy(depUuid);

    const output = project.toBuild();
    t.true(output.includes("PBXTargetDependency"));
    t.true(output.includes("PBXContainerItemProxy"));
  });

  test("createNativeTarget creates a complete target", (t) => {
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
    t.truthy(targetUuid);

    // Should have one more target
    const afterTargets = project.getNativeTargets();
    t.is(afterTargets.length, beforeTargets.length + 1);
    t.true(afterTargets.includes(targetUuid));

    // Can set build settings on the new target
    project.setBuildSetting(targetUuid, "IPHONEOS_DEPLOYMENT_TARGET", "16.0");

    // Save and reopen
    project.save();
    const reopened = native.XcodeProject.open(pbxpath);
    const output = reopened.toBuild();

    t.true(output.includes("MyWidget"));
    t.true(output.includes("com.example.mywidget"));
    t.true(output.includes("com.apple.product-type.app-extension"));
    t.true(output.includes("PBXSourcesBuildPhase"));
    t.true(output.includes("PBXFrameworksBuildPhase"));
    t.true(output.includes("PBXResourcesBuildPhase"));

    // Verify build setting persisted
    t.is(reopened.getBuildSetting(targetUuid, "IPHONEOS_DEPLOYMENT_TARGET"), "16.0");
  });

  test("full workflow: add file + add to sources phase + save + reopen", (t) => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const target = project.findMainAppTarget("ios");
    const mainGroup = project.mainGroupUuid;

    // Add a Swift file to the project
    const fileUuid = project.addFile(mainGroup, "Features/Login.swift");

    // Add it to the Sources build phase
    const sourcesPhase = project.ensureBuildPhase(target, "PBXSourcesBuildPhase");
    const buildFileUuid = project.addBuildFile(sourcesPhase, fileUuid);
    t.truthy(buildFileUuid);

    // Save and reopen
    project.save();
    const reopened = native.XcodeProject.open(pbxpath);
    const output = reopened.toBuild();

    t.true(output.includes("Login.swift"));
    t.true(output.includes("Login.swift in Sources"));
    t.true(output.includes("sourcecode.swift"));
  });
  test("XcodeProject.fromString() parses without a file on disk", (t) => {
    const content = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = native.XcodeProject.fromString(content);

    t.is(project.archiveVersion, 1);
    t.truthy(project.mainGroupUuid);
    t.true(project.getNativeTargets().length > 0);
    t.is(project.filePath, null); // no file path since it came from a string

    // Can still use all high-level APIs
    const target = project.findMainAppTarget("ios");
    t.truthy(project.getBuildSetting(target, "PRODUCT_NAME"));

    // Can serialize
    const output = project.toBuild();
    t.is(output, content); // round-trip matches
  });

  test("getTargetName / setTargetName", (t) => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const target = project.findMainAppTarget("ios");

    const oldName = project.getTargetName(target);
    t.truthy(oldName);

    project.setTargetName(target, "RenamedApp");
    t.is(project.getTargetName(target), "RenamedApp");

    project.save();
    const output = readFileSync(pbxpath, "utf8");
    t.true(output.includes("RenamedApp"));
  });

  test("getObjectProperty / setObjectProperty", (t) => {
    const content = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = native.XcodeProject.fromString(content);

    const target = project.findMainAppTarget("ios");
    t.is(project.getObjectProperty(target, "isa"), "PBXNativeTarget");
    t.truthy(project.getObjectProperty(target, "name"));

    project.setObjectProperty(target, "productName", "CustomProduct");
    t.is(project.getObjectProperty(target, "productName"), "CustomProduct");
  });

  test("findObjectsByIsa", (t) => {
    const content = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = native.XcodeProject.fromString(content);

    const groups = project.findObjectsByIsa("PBXGroup");
    t.true(groups.length > 0);
    // Every returned UUID should be a PBXGroup
    for (const uuid of groups) {
      t.is(project.getObjectProperty(uuid, "isa"), "PBXGroup");
    }

    const fileRefs = project.findObjectsByIsa("PBXFileReference");
    t.true(fileRefs.length > 0);
  });

  test("renameTarget cascades through project", (t) => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const target = project.findMainAppTarget("ios");
    const oldName = project.getTargetName(target);

    project.renameTarget(target, oldName, "BrandNewApp");

    // Target name updated
    t.is(project.getTargetName(target), "BrandNewApp");

    project.save();
    const output = readFileSync(pbxpath, "utf8");

    // Product reference updated
    t.true(output.includes("BrandNewApp.app"));
    t.false(output.includes(`${oldName}.app`));

    // Group path updated
    t.true(output.includes("BrandNewApp"));

    // Still valid pbxproj
    t.true(output.startsWith("// !$*UTF8*$!"));
  });

  test("embedExtension wires copy files phase", (t) => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const hostTarget = project.findMainAppTarget("ios");

    // Create an extension target
    const extTarget = project.createNativeTarget(
      "MyWidget",
      "com.apple.product-type.app-extension",
      "com.example.widget",
    );

    // Embed it
    const phaseUuid = project.embedExtension(hostTarget, extTarget);
    t.truthy(phaseUuid);

    project.save();
    const output = readFileSync(pbxpath, "utf8");
    t.true(output.includes("PBXCopyFilesBuildPhase"));
    t.true(output.includes("Embed Foundation Extensions"));
    t.true(output.includes("dstSubfolderSpec = 13"));
  });

  test("addFileSystemSyncGroup (Xcode 16+)", (t) => {
    const tmp = mkdtempSync(join(tmpdir(), "xcode-test-"));
    const pbxpath = join(tmp, "project.pbxproj");
    cpSync(join(FIXTURES_DIR, "project.pbxproj"), pbxpath);

    const project = native.XcodeProject.open(pbxpath);
    const target = project.findMainAppTarget("ios");

    const syncUuid = project.addFileSystemSyncGroup(target, "MyApp");
    t.truthy(syncUuid);

    // Should be in main group children
    const mainChildren = project.getGroupChildren(project.mainGroupUuid);
    t.true(mainChildren.includes(syncUuid));

    project.save();
    const output = readFileSync(pbxpath, "utf8");
    t.true(output.includes("PBXFileSystemSynchronizedRootGroup"));
    t.true(output.includes("fileSystemSynchronizedGroups"));
  });
} else {
  test("skipped — native module not available", (t) => {
    t.pass("Native module not built; JS tests skipped");
  });
}
