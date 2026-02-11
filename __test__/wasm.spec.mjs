/**
 * WASM integration tests.
 * Run: npx ava __test__/wasm.spec.mjs
 * Requires: make build-wasm
 */

import test from "ava";
import { readFileSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const FIXTURES_DIR = join(__dirname, "fixtures");

import { createRequire } from "module";
const require = createRequire(import.meta.url);

let wasm;
try {
  wasm = require("../pkg/node");
} catch {
  wasm = null;
}

if (wasm) {
  test("parse returns valid JSON", (t) => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const json = wasm.parse(text);
    const parsed = JSON.parse(json);
    t.is(parsed.archiveVersion, 1);
    t.truthy(parsed.objects);
    t.truthy(parsed.rootObject);
  });

  test("build round-trips", (t) => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const json = wasm.parse(text);
    const output = wasm.build(json);
    t.is(output, text);
  });

  test("parseAndBuild round-trips", (t) => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const output = wasm.parseAndBuild(text);
    t.is(output, text);
  });

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

  for (const fixture of roundTripFixtures) {
    test(`round-trip: ${fixture}`, (t) => {
      const text = readFileSync(join(FIXTURES_DIR, fixture), "utf8");
      const output = wasm.parseAndBuild(text);
      t.is(output, text);
    });
  }

  test("XcodeProject constructor works", (t) => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = new wasm.XcodeProject(text);
    t.is(Number(project.archiveVersion), 1);
    t.truthy(project.mainGroupUuid);
  });

  test("XcodeProject targets and build settings", (t) => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = new wasm.XcodeProject(text);

    const targets = project.getNativeTargets();
    t.true(targets.length > 0);

    const target = project.findMainAppTarget("ios");
    t.truthy(target);

    const name = project.getTargetName(target);
    t.truthy(name);

    project.setBuildSetting(target, "TEST_KEY", "TEST_VALUE");
    const output = project.toBuild();
    t.true(output.includes("TEST_KEY"));
    t.true(output.includes("TEST_VALUE"));
  });

  test("XcodeProject addFile + addBuildFile", (t) => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = new wasm.XcodeProject(text);

    const mainGroup = project.mainGroupUuid;
    const fileUuid = project.addFile(mainGroup, "Sources/New.swift");
    t.truthy(fileUuid);

    const target = project.findMainAppTarget("ios");
    const phase = project.ensureBuildPhase(target, "PBXSourcesBuildPhase");
    const buildFile = project.addBuildFile(phase, fileUuid);
    t.truthy(buildFile);

    const output = project.toBuild();
    t.true(output.includes("New.swift"));
    t.true(output.includes("New.swift in Sources"));
  });

  test("XcodeProject createNativeTarget + embedExtension", (t) => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = new wasm.XcodeProject(text);

    const host = project.findMainAppTarget("ios");
    const ext = project.createNativeTarget("Widget", "com.apple.product-type.app-extension", "com.example.widget");
    t.truthy(ext);

    project.addDependency(host, ext);
    const phase = project.embedExtension(host, ext);
    t.truthy(phase);

    const output = project.toBuild();
    t.true(output.includes("Widget"));
    t.true(output.includes("PBXCopyFilesBuildPhase"));
    t.true(output.includes("Embed Foundation Extensions"));
  });

  test("XcodeProject renameTarget cascades", (t) => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = new wasm.XcodeProject(text);

    const target = project.findMainAppTarget("ios");
    const oldName = project.getTargetName(target);

    project.renameTarget(target, oldName, "RenamedApp");
    t.is(project.getTargetName(target), "RenamedApp");

    const output = project.toBuild();
    t.true(output.includes("RenamedApp.app"));
    t.true(output.includes("RenamedApp"));
  });

  test("XcodeProject generic property access", (t) => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = new wasm.XcodeProject(text);

    const target = project.findMainAppTarget("ios");
    t.is(project.getObjectProperty(target, "isa"), "PBXNativeTarget");

    project.setObjectProperty(target, "productName", "Custom");
    t.is(project.getObjectProperty(target, "productName"), "Custom");
  });

  test("XcodeProject findObjectsByIsa", (t) => {
    const text = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = new wasm.XcodeProject(text);

    const groups = project.findObjectsByIsa("PBXGroup");
    t.true(groups.length > 0);

    const fileRefs = project.findObjectsByIsa("PBXFileReference");
    t.true(fileRefs.length > 0);
  });
} else {
  test("skipped — WASM not built", (t) => {
    t.pass("Run 'make build-wasm' first");
  });
}
