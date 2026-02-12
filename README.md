# @xcodekit/xcode

Super fast Xcode `.pbxproj` parser and serializer written in Rust.

Drop-in replacement for the low-level API of [`@bacons/xcode`](https://github.com/EvanBacon/xcode) — same `parse()` and `build()` interface, **3-15x faster parsing**, byte-identical output. Available as a native binary (napi) or universal WASM.

## Install

```bash
# Native (fastest, Node.js only)
npm install @xcodekit/xcode

# WASM (universal — browsers, Deno, Cloudflare Workers, etc.)
npm install @xcodekit/xcode-wasm
```

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

#### `buildFromJSON(json: string): string`

Same as `build()` but accepts `JSON.stringify(project)` directly. Faster because it avoids napi's recursive JS-to-Rust object marshalling.

```js
const output = buildFromJSON(JSON.stringify(project));
```

#### `parseAndBuild(text: string): string`

Parse and immediately re-serialize. Stays entirely in Rust with zero JS/Rust marshalling — the fastest possible round-trip path.

```js
const output = parseAndBuild(readFileSync("project.pbxproj", "utf8"));
```

### High-Level (XcodeProject)

#### `XcodeProject.open(filePath)` / `XcodeProject.fromString(content)`

Open from disk or parse from a string (e.g. content fetched over the network).

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

### WASM

The WASM build (`@xcodekit/xcode-wasm`) has the same API with minor differences:

- `parse()` / `build()` work with JSON **strings** (not JS objects) — call `JSON.parse()` / `JSON.stringify()` on your side
- `XcodeProject` is created with `new XcodeProject(content)` instead of factory methods

```js
// Browser / Deno / Cloudflare Workers
import { parse, build, XcodeProject } from "@xcodekit/xcode-wasm";

// Low-level
const json = parse(text); // returns JSON string
const project = JSON.parse(json);
const output = build(json); // accepts JSON string

// High-level (same API as napi)
const xcode = new XcodeProject(text);
xcode.setBuildSetting(target, "SWIFT_VERSION", "6.0");
const pbxproj = xcode.toBuild();
```

### WASM on Node.js

Use the `/node` subpath to get `open()` and `save()`:

```js
// ESM
import { XcodeProject } from "@xcodekit/xcode-wasm/node";

// CJS
const { XcodeProject } = require("@xcodekit/xcode-wasm/node");

const project = XcodeProject.open("project.pbxproj");
project.setBuildSetting(target, "SWIFT_VERSION", "6.0");
project.save();
```

### Shared Types

Both packages ship `types.d.ts` with rich TypeScript types for the parsed JSON structure:

```ts
import type { ParsedProject, PBXNativeTarget, BuildSettings, ISA } from "@xcodekit/xcode-wasm/types";
// or
import type { ParsedProject, PBXNativeTarget, BuildSettings, ISA } from "@xcodekit/xcode/types";
```

## Performance

Benchmarked on Apple M4 Pro, Node.js v24. Median of 200 iterations.

- **WASM** — `@xcodekit/xcode-wasm`, the WebAssembly build. Runs everywhere without native compilation, including edge runtimes and browsers.
- **napi** — `@xcodekit/xcode`, the native Node.js addon via napi-rs. Fastest on supported platforms (macOS, Linux, Windows).
- **TS** — `@bacons/xcode/json`, the original TypeScript implementation using Chevrotain.

### Parse

| Fixture                    | WASM   | napi   | TS      | WASM vs TS | napi vs TS |
| -------------------------- | ------ | ------ | ------- | ---------- | ---------- |
| swift-protobuf (257 KB)    | 2.9 ms | 3.7 ms | 43.9 ms | **15.2x**  | **11.8x**  |
| Cocoa-Application (166 KB) | 2.4 ms | 3.2 ms | 17.2 ms | **7.3x**   | **5.4x**   |
| AFNetworking (99 KB)       | 1.3 ms | 1.7 ms | 6.6 ms  | **5.1x**   | **3.9x**   |
| watch (48 KB)              | 0.7 ms | 0.9 ms | 2.1 ms  | **3.0x**   | **2.3x**   |
| project (19 KB)            | 0.3 ms | 0.4 ms | 0.8 ms  | **2.9x**   | **2.2x**   |

### Build

| Fixture                    | WASM   | napi   | TS      | WASM vs TS  | napi vs TS  |
| -------------------------- | ------ | ------ | ------- | ----------- | ----------- |
| swift-protobuf (257 KB)    | 4.1 ms | 5.2 ms | 12.0 ms | **2.9x**    | **2.3x**    |
| Cocoa-Application (166 KB) | 3.3 ms | 4.5 ms | 2.7 ms  | 1.2x slower | 1.7x slower |
| AFNetworking (99 KB)       | 1.6 ms | 2.3 ms | 1.8 ms  | **1.1x**    | 1.3x slower |
| watch (48 KB)              | 0.8 ms | 1.1 ms | 0.4 ms  | 1.9x slower | 2.7x slower |
| project (19 KB)            | 0.3 ms | 0.4 ms | 0.2 ms  | 1.9x slower | 2.7x slower |

> [!NOTE]
> TypeScript wins `build()` on smaller files because it operates directly on native JS objects with zero serialization cost. Rust pays a fixed overhead for JSON deserialization (~0.1 ms) which dominates on small inputs. On large files (>100 KB) where actual serialization work dominates, Rust wins.
>
> In practice this doesn't matter much — the parse speedup more than compensates, as the round-trip tables below show.

### Round-Trip (parse + build)

| Fixture                    | WASM   | napi   | TS      | WASM vs TS | napi vs TS |
| -------------------------- | ------ | ------ | ------- | ---------- | ---------- |
| swift-protobuf (257 KB)    | 7.0 ms | 9.0 ms | 55.9 ms | **8.0x**   | **6.2x**   |
| Cocoa-Application (166 KB) | 5.7 ms | 7.7 ms | 19.9 ms | **3.5x**   | **2.6x**   |
| AFNetworking (99 KB)       | 2.9 ms | 4.0 ms | 8.4 ms  | **2.9x**   | **2.1x**   |
| watch (48 KB)              | 1.5 ms | 2.0 ms | 2.5 ms  | **1.7x**   | **1.3x**   |
| project (19 KB)            | 0.6 ms | 0.8 ms | 1.0 ms  | **1.7x**   | **1.3x**   |

### Round-Trip (parseAndBuild — zero marshalling)

| Fixture                    | WASM   | napi   | TS      | WASM vs TS | napi vs TS |
| -------------------------- | ------ | ------ | ------- | ---------- | ---------- |
| swift-protobuf (257 KB)    | 4.8 ms | 4.4 ms | 62.7 ms | **13.1x**  | **14.2x**  |
| Cocoa-Application (166 KB) | 3.7 ms | 3.7 ms | 22.4 ms | **6.0x**   | **6.1x**   |
| AFNetworking (99 KB)       | 1.9 ms | 1.8 ms | 9.2 ms  | **4.7x**   | **5.1x**   |
| watch (48 KB)              | 0.9 ms | 0.9 ms | 2.8 ms  | **3.0x**   | **3.1x**   |
| project (19 KB)            | 0.4 ms | 0.3 ms | 1.0 ms  | **2.8x**   | **2.9x**   |

### Package Size

|              | WASM   | napi   | TS      |
| ------------ | ------ | ------ | ------- |
| Uncompressed | 245 KB | 559 KB | 1.1 MB  |
| Gzipped      | 96 KB  | 270 KB | ~400 KB |

Run benchmarks yourself:

```bash
make bench          # all benchmarks
make bench-rust     # pure Rust (no JS overhead)
make bench-js       # napi vs TypeScript
make bench-wasm     # WASM vs napi vs TypeScript
```

## Choosing the Right Package

| Environment                        | Package                | Notes                                    |
| ---------------------------------- | ---------------------- | ---------------------------------------- |
| Node.js                            | `@xcodekit/xcode`      | Fastest. Native binary per platform.     |
| Browser / Deno / Workers           | `@xcodekit/xcode-wasm` | Universal. 96 KB gzipped.                |
| Node.js without native compilation | `@xcodekit/xcode-wasm` | Works everywhere, no build tools needed. |

## Compatibility

- Full feature parity with `@bacons/xcode/json` (parse/build)
- 13/13 round-trip fixtures produce **byte-identical** output
- All escape sequences: standard (`\n`, `\t`, etc.), Unicode (`\Uxxxx`), octal, NeXTSTEP (128 entries)
- Xcode 16+ file system synchronized groups
- Swift Package Manager references
- 119 tests (62 Rust + 38 napi JS + 19 WASM JS)

## Supported Platforms

### napi (`@xcodekit/xcode`)

| Platform | Architecture                       |
| -------- | ---------------------------------- |
| macOS    | arm64 (Apple Silicon), x64 (Intel) |
| Linux    | x64 (glibc), arm64 (glibc)         |
| Windows  | x64 (MSVC)                         |

### WASM (`@xcodekit/xcode-wasm`)

Any environment that supports WebAssembly — browsers, Node.js, Deno, Bun, Cloudflare Workers, etc.

## Development

```bash
# Prerequisites
# - Rust toolchain (cargo)
# - Node.js >= 18
# - wasm-pack (for WASM builds)

# Install dependencies
npm install

# Run tests
make test           # all tests (Rust + napi JS + WASM JS)
make test-rust      # Rust tests only (fast, no Node needed)
make test-js        # napi JS tests (builds debug binary first)
make test-wasm      # WASM JS tests (requires: make build-wasm)

# Build
make build          # napi release build
make build-debug    # napi debug build (faster compilation)
make build-wasm     # WASM build (web + node + bundler targets)

# Other
make check          # Type-check all targets without building
make fmt            # cargo fmt
make lint           # cargo clippy
make clean          # Remove all artifacts
```

### Project Structure

```
src/
  lib.rs                  # napi + wasm exports
  parser/
    lexer.rs              # Fast byte-scanning tokenizer
    parser.rs             # Recursive descent parser → PlistValue
    escape.rs             # String unescape (standard, Unicode, octal, NeXTSTEP)
  writer/
    serializer.rs         # PlistValue → .pbxproj (section sorting, inline formatting)
    comments.rs           # UUID → inline comment generation
    quotes.rs             # String quoting/escaping
  types/
    plist.rs              # PlistValue enum (String, Integer, Float, Data, Object, Array)
    isa.rs                # ISA enum (29 variants)
    constants.rs          # File type mappings, SDK versions, default build settings
  project/
    xcode_project.rs      # High-level project container
    uuid.rs               # Deterministic MD5-based UUID generation
    paths.rs              # sourceTree path resolution
    build_settings.rs     # $(VARIABLE:transform) resolver
  objects/
    mod.rs                # PbxObject + PbxObjectExt trait
```

## License

MIT
