# @xcodekit/xcode

Super fast Xcode `.pbxproj` parser and serializer written in Rust.

Drop-in replacement for the low-level API of [`@bacons/xcode`](https://github.com/EvanBacon/xcode) — same `parse()` and `build()` interface, **3-15x faster parsing**, byte-identical output.

## Install

```bash
# Recommended — auto-selects native or WASM
npm install @xcodekit/xcode

# Native only (Node.js, fastest)
npm install @xcodekit/xcode-node

# WASM only (universal — Bun, CF Workers, any bundler)
npm install @xcodekit/xcode-wasm
```

`@xcodekit/xcode` is a thin auto-selector — in Node.js/Bun it tries the native addon first, everywhere else it uses WASM. You get optimal performance without choosing.

## Quick Start

```js
import { parse, build } from "@xcodekit/xcode";
import { readFileSync, writeFileSync } from "fs";

// Parse a .pbxproj file
const text = readFileSync("project.pbxproj", "utf8");
const project = parse(text);

// Modify it
project.objects["YOUR_UUID"] = {
  isa: "PBXFileReference",
  path: "NewFile.swift",
  sourceTree: "<group>",
};

// Write it back
const output = build(project);
writeFileSync("project.pbxproj", output);
```

## API

### Low-Level (JSON)

#### `parse(text: string): object`

Parse a `.pbxproj` string into a JSON-compatible object. Matches the output of `@bacons/xcode/json`'s `parse()`.

```js
const project = parse(readFileSync("project.pbxproj", "utf8"));
```

#### `build(project: object): string`

Serialize a JSON object back to `.pbxproj` format. Produces byte-identical output to `@bacons/xcode/json`'s `build()`.

```js
writeFileSync("project.pbxproj", build(project));
```

#### `parseAndBuild(text: string): string`

Parse and immediately re-serialize. Stays entirely in Rust/WASM with zero JS marshalling — the fastest possible round-trip path.

```js
const output = parseAndBuild(readFileSync("project.pbxproj", "utf8"));
```

### High-Level (XcodeProject)

#### `XcodeProject.open(filePath)` / `XcodeProject.fromString(content)`

Open from disk or parse from a string.

```js
import { XcodeProject } from "@xcodekit/xcode";

// From disk
const project = XcodeProject.open("ios/MyApp.xcodeproj/project.pbxproj");

// From string (no file on disk needed)
const project = XcodeProject.fromString(pbxprojContent);

// Properties
project.archiveVersion; // 1
project.objectVersion; // 46
project.filePath; // path if opened from disk, null if fromString
project.mainGroupUuid; // root group UUID

// Targets
const targets = project.getNativeTargets(); // UUID[]
const mainApp = project.findMainAppTarget("ios"); // UUID | null
project.getTargetName(mainApp); // "MyApp"
project.setTargetName(mainApp, "NewName");
project.renameTarget(mainApp, "OldName", "NewName"); // cascades to groups, product refs, proxies

// Build settings
project.getBuildSetting(targetUuid, "PRODUCT_BUNDLE_IDENTIFIER");
project.setBuildSetting(targetUuid, "SWIFT_VERSION", "5.0");
project.removeBuildSetting(targetUuid, "CODE_SIGN_IDENTITY");

// Files & groups
const fileUuid = project.addFile(project.mainGroupUuid, "Sources/App.swift");
const groupUuid = project.addGroup(project.mainGroupUuid, "Features");
const children = project.getGroupChildren(groupUuid);

// Build phases
const phase = project.ensureBuildPhase(targetUuid, "PBXSourcesBuildPhase");
project.addBuildFile(phase, fileUuid);

// Frameworks
project.addFramework(targetUuid, "SwiftUI");

// Create targets
const widgetTarget = project.createNativeTarget(
  "MyWidget",
  "com.apple.product-type.app-extension",
  "com.example.mywidget",
);

// Embed extension into host app
project.addDependency(mainApp, widgetTarget);
project.embedExtension(mainApp, widgetTarget);

// Xcode 16+ file system sync groups
project.addFileSystemSyncGroup(widgetTarget, "MyWidget");

// Generic object access
project.getObjectProperty(uuid, "path");
project.setObjectProperty(uuid, "path", "new/path");
const proxies = project.findObjectsByIsa("PBXContainerItemProxy");

// Validation
const orphans = project.findOrphanedReferences();
// [{ referrerUuid, referrerIsa, property, orphanUuid }]

// Serialize
const pbxproj = project.toBuild(); // string
const json = project.toJSON(); // object

// Write back to disk
project.save();

// Deterministic UUID generation
const uuid = project.getUniqueId("my-seed-string"); // 24-char hex
```

All `XcodeProject` methods operate in Rust/WASM — only primitive strings cross the boundary. This is the fastest path for project manipulation.

## Packages

| Package                | What                                             | Size    | Runtime                            |
| ---------------------- | ------------------------------------------------ | ------- | ---------------------------------- |
| `@xcodekit/xcode`      | Auto-selector (tries native, falls back to WASM) | ~4 KB   | Node.js, Bun, CF Workers, browsers |
| `@xcodekit/xcode-node` | Native napi-rs addon                             | ~600 KB | Node.js only                       |
| `@xcodekit/xcode-wasm` | WASM with inlined base64                         | ~370 KB | Everywhere                         |

`@xcodekit/xcode` uses [exports conditions](https://nodejs.org/api/packages.html#conditional-exports): the `node` condition loads native, `default` loads WASM. No try/catch at runtime for most environments.

### When to use which

| Environment               | Recommended                                          |
| ------------------------- | ---------------------------------------------------- |
| Node.js / Bun             | `@xcodekit/xcode` (auto-selects native)              |
| Cloudflare Workers / edge | `@xcodekit/xcode` or `@xcodekit/xcode-wasm`          |
| Bundled app (single file) | `@xcodekit/xcode-wasm` (inlined, no file resolution) |
| Need max performance      | `@xcodekit/xcode-node` (direct, no facade overhead)  |

## Performance

Benchmarked on Apple M4 Pro, Node.js v24. Median of 200 iterations.

- **xcode-wasm** — `@xcodekit/xcode-wasm`
- **xcode-node** — `@xcodekit/xcode-node`
- **@bacons/xcode** — `@bacons/xcode/json`

### Parse

| Fixture                    | xcode-wasm | xcode-node | @bacons/xcode | wasm speedup | node speedup |
| -------------------------- | ---------- | ---------- | ------------- | ------------ | ------------ |
| swift-protobuf (257 KB)    | 2.9 ms     | 3.7 ms     | 43.9 ms       | **15.2x**    | **11.8x**    |
| Cocoa-Application (166 KB) | 2.4 ms     | 3.2 ms     | 17.2 ms       | **7.3x**     | **5.4x**     |
| AFNetworking (99 KB)       | 1.3 ms     | 1.7 ms     | 6.6 ms        | **5.1x**     | **3.9x**     |
| watch (48 KB)              | 0.7 ms     | 0.9 ms     | 2.1 ms        | **3.0x**     | **2.3x**     |
| project (19 KB)            | 0.3 ms     | 0.4 ms     | 0.8 ms        | **2.9x**     | **2.2x**     |

### Build

| Fixture                    | xcode-wasm | xcode-node | @bacons/xcode | wasm speedup | node speedup |
| -------------------------- | ---------- | ---------- | ------------- | ------------ | ------------ |
| swift-protobuf (257 KB)    | 4.1 ms     | 5.2 ms     | 12.0 ms       | **2.9x**     | **2.3x**     |
| Cocoa-Application (166 KB) | 3.3 ms     | 4.5 ms     | 2.7 ms        | 1.2x slower  | 1.7x slower  |
| AFNetworking (99 KB)       | 1.6 ms     | 2.3 ms     | 1.8 ms        | **1.1x**     | 1.3x slower  |
| watch (48 KB)              | 0.8 ms     | 1.1 ms     | 0.4 ms        | 1.9x slower  | 2.7x slower  |
| project (19 KB)            | 0.3 ms     | 0.4 ms     | 0.2 ms        | 1.9x slower  | 2.7x slower  |

> [!NOTE]
> TypeScript wins `build()` on smaller files because it operates directly on native JS objects with zero serialization cost. On large files (>100 KB) where actual serialization work dominates, Rust wins.

### Round-Trip (parse + build)

| Fixture                    | xcode-wasm | xcode-node | @bacons/xcode | wasm speedup | node speedup |
| -------------------------- | ---------- | ---------- | ------------- | ------------ | ------------ |
| swift-protobuf (257 KB)    | 7.0 ms     | 9.0 ms     | 55.9 ms       | **8.0x**     | **6.2x**     |
| Cocoa-Application (166 KB) | 5.7 ms     | 7.7 ms     | 19.9 ms       | **3.5x**     | **2.6x**     |
| AFNetworking (99 KB)       | 2.9 ms     | 4.0 ms     | 8.4 ms        | **2.9x**     | **2.1x**     |
| watch (48 KB)              | 1.5 ms     | 2.0 ms     | 2.5 ms        | **1.7x**     | **1.3x**     |
| project (19 KB)            | 0.6 ms     | 0.8 ms     | 1.0 ms        | **1.7x**     | **1.3x**     |

### Round-Trip (parseAndBuild — zero marshalling)

| Fixture                    | xcode-wasm | xcode-node | @bacons/xcode | wasm speedup | node speedup |
| -------------------------- | ---------- | ---------- | ------------- | ------------ | ------------ |
| swift-protobuf (257 KB)    | 4.8 ms     | 4.4 ms     | 62.7 ms       | **13.1x**    | **14.2x**    |
| Cocoa-Application (166 KB) | 3.7 ms     | 3.7 ms     | 22.4 ms       | **6.0x**     | **6.1x**     |
| AFNetworking (99 KB)       | 1.9 ms     | 1.8 ms     | 9.2 ms        | **4.7x**     | **5.1x**     |
| watch (48 KB)              | 0.9 ms     | 0.9 ms     | 2.8 ms        | **3.0x**     | **3.1x**     |
| project (19 KB)            | 0.4 ms     | 0.3 ms     | 1.0 ms        | **2.8x**     | **2.9x**     |

### Package Size

|              | xcode-wasm | xcode-node | @bacons/xcode |
| ------------ | ---------- | ---------- | ------------- |
| Uncompressed | ~370 KB    | 559 KB     | 1.1 MB        |
| Gzipped      | ~96 KB     | 270 KB     | ~400 KB       |

## Compatibility

- Full feature parity with `@bacons/xcode/json` (parse/build)
- 13/13 round-trip fixtures produce **byte-identical** output
- All escape sequences: standard (`\n`, `\t`, etc.), Unicode (`\Uxxxx`), octal, NeXTSTEP (128 entries)
- Xcode 16+ file system synchronized groups
- Swift Package Manager references

## Supported Platforms

### Native (`@xcodekit/xcode-node`)

| Platform | Architecture                       |
| -------- | ---------------------------------- |
| macOS    | arm64 (Apple Silicon), x64 (Intel) |
| Linux    | x64 (glibc), arm64 (glibc)         |
| Windows  | x64 (MSVC)                         |

### WASM (`@xcodekit/xcode-wasm`)

Any environment that supports WebAssembly — Node.js, Bun, Deno, Cloudflare Workers, browsers, etc. WASM binary is inlined as base64 — no file resolution needed, works in single-file bundles.

## Development

```bash
# Prerequisites: Rust toolchain, Node.js >= 18, wasm-pack, binaryen

npm install

# Tests
make test           # all (Rust + native JS + WASM JS)
make test-rust      # Rust unit + integration tests
make test-js        # native napi tests (vitest)
make test-wasm      # WASM tests (vitest)

# Build
make build          # native napi release
make build-debug    # native napi debug (faster compilation)
make build-wasm     # WASM → pkg/xcode-wasm/
make build-node     # native → pkg/xcode-node/
make build-all      # all packages

# Other
make check          # type-check all targets
make fmt            # cargo fmt + prettier
make lint           # cargo clippy
make bench          # all benchmarks
make clean          # remove all artifacts
```

### Project Structure

```
src/                          # Rust source (parser, writer, project API)
tests/
  integration_tests.rs        # Rust integration tests
  node.test.mjs               # Native napi JS tests (vitest)
  wasm.test.mjs               # WASM JS tests (vitest)
  fixtures/                   # 20 real-world .pbxproj files
npm/                          # Publishable package metadata
  xcode/                      # @xcodekit/xcode (auto-selector facade)
  xcode-node/                 # @xcodekit/xcode-node (native napi)
    platforms/                # Platform binary packages (darwin-arm64, etc.)
  xcode-wasm/                 # @xcodekit/xcode-wasm (WASM wrapper + types)
scripts/                      # Build scripts
  build-node-pkg.sh           # Assembles pkg/xcode-node/
  build-wasm-pkg.sh           # Assembles pkg/xcode-wasm/ (wasm-opt, base64 inline)
pkg/                          # Generated build output (gitignored)
```

## License

MIT
