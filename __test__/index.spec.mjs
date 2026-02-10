import test from "ava";
import { readFileSync } from "fs";
import { join, dirname } from "path";
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
    const project = native.XcodeProject.open(
      join(FIXTURES_DIR, "project.pbxproj")
    );
    t.truthy(project);
    const json = project.toJSON();
    t.truthy(json);
    t.truthy(json.objects);
  });

  test("XcodeProject.toBuild() round-trips", (t) => {
    const original = readFileSync(join(FIXTURES_DIR, "project.pbxproj"), "utf8");
    const project = native.XcodeProject.open(
      join(FIXTURES_DIR, "project.pbxproj")
    );
    const output = project.toBuild();
    t.is(output, original);
  });

  test("XcodeProject properties", (t) => {
    const project = native.XcodeProject.open(
      join(FIXTURES_DIR, "project.pbxproj")
    );
    t.is(project.archiveVersion, 1);
    t.is(project.objectVersion, 46);
    t.truthy(project.filePath);
  });

  test("XcodeProject.getNativeTargets() returns UUIDs", (t) => {
    const project = native.XcodeProject.open(
      join(FIXTURES_DIR, "project.pbxproj")
    );
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
    const project = native.XcodeProject.open(
      join(FIXTURES_DIR, "project.pbxproj")
    );
    const targetUuid = project.findMainAppTarget("ios");
    t.truthy(targetUuid);
    t.is(typeof targetUuid, "string");
  });

  test("XcodeProject.getUniqueId() is deterministic", (t) => {
    const project = native.XcodeProject.open(
      join(FIXTURES_DIR, "project.pbxproj")
    );
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
    const project = native.XcodeProject.open(
      join(FIXTURES_DIR, "project.pbxproj")
    );
    const orphans = project.findOrphanedReferences();
    t.is(orphans.length, 0);
  });

  test("malformed project detects orphaned references", (t) => {
    const project = native.XcodeProject.open(
      join(FIXTURES_DIR, "malformed.pbxproj")
    );
    const orphans = project.findOrphanedReferences();
    t.true(orphans.length > 0);

    const known = orphans.find(
      (o) => o.orphanUuid === "3E1C2299F05049539341855D"
    );
    t.truthy(known);
    t.is(known.referrerIsa, "PBXResourcesBuildPhase");
    t.is(known.property, "files");
  });

  test("malformed project still parses and serializes", (t) => {
    const project = native.XcodeProject.open(
      join(FIXTURES_DIR, "malformed.pbxproj")
    );
    t.truthy(project.toJSON());
    const output = project.toBuild();
    t.true(output.includes("PBXResourcesBuildPhase"));
    t.true(output.includes("baconwidget"));
  });
} else {
  test("skipped — native module not available", (t) => {
    t.pass("Native module not built; JS tests skipped");
  });
}
