# @mozharovsky/xcode

A fast, native Xcode `.pbxproj` parser and serializer for Node.js, written in Rust.

Drop-in replacement for the low-level API of [`@bacons/xcode`](https://github.com/EvanBacon/xcode) — same `parse()` and `build()` interface, **2-12x faster parsing**, byte-identical output, shipped as a single native binary per platform.

## Install

```bash
npm install @mozharovsky/xcode
```

Platform-specific binaries are installed automatically via `optionalDependencies`.

## Quick Start

```js
import { parse, build } from "@mozharovsky/xcode";
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

#### `build(project: object): string`

Serialize a JSON object back to `.pbxproj` format. Produces byte-identical output to `@bacons/xcode/json`'s `build()`.

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

#### `XcodeProject.open(filePath: string): XcodeProject`

Open and parse a `.pbxproj` file from disk.

```js
import { XcodeProject } from "@mozharovsky/xcode";

const project = XcodeProject.open("ios/MyApp.xcodeproj/project.pbxproj");

// Properties
project.archiveVersion; // 1
project.objectVersion;  // 46
project.filePath;       // the path it was opened from

// Targets
const targets = project.getNativeTargets(); // UUID[]
const mainApp = project.findMainAppTarget("ios"); // UUID | null

// Build settings
project.getBuildSetting(targetUuid, "PRODUCT_BUNDLE_IDENTIFIER");
project.setBuildSetting(targetUuid, "SWIFT_VERSION", "5.0");
project.removeBuildSetting(targetUuid, "CODE_SIGN_IDENTITY");

// Serialize
const pbxproj = project.toBuild(); // string
const json = project.toJSON();     // object

// Write back to disk
project.save();

// Deterministic UUID generation
const uuid = project.getUniqueId("my-seed-string"); // 24-char hex
```

## Performance

Benchmarked on Apple M4 Pro, Node.js v24. Median of 200 iterations.

### Parse

| Fixture | Rust | TypeScript | Speedup |
|---------|------|-----------|---------|
| swift-protobuf (257 KB) | 3.7 ms | 43.5 ms | **11.6x** |
| Cocoa-Application (166 KB) | 3.2 ms | 17.3 ms | **5.5x** |
| AFNetworking (99 KB) | 1.8 ms | 6.6 ms | **3.8x** |
| watch (48 KB) | 0.9 ms | 2.1 ms | **2.2x** |
| project (19 KB) | 0.4 ms | 0.8 ms | **2.2x** |

### Round-Trip (parse + build)

| Fixture | Rust | TypeScript | Speedup |
|---------|------|-----------|---------|
| swift-protobuf (257 KB) | 9.1 ms | 63.1 ms | **6.9x** |
| Cocoa-Application (166 KB) | 8.0 ms | 22.2 ms | **2.8x** |
| AFNetworking (99 KB) | 4.2 ms | 9.3 ms | **2.2x** |
| watch (48 KB) | 2.1 ms | 2.7 ms | **1.3x** |
| project (19 KB) | 0.8 ms | 1.0 ms | **1.2x** |

### parseAndBuild (zero marshalling)

| Fixture | Rust | TypeScript | Speedup |
|---------|------|-----------|---------|
| swift-protobuf (257 KB) | 4.6 ms | 62.9 ms | **13.6x** |
| Cocoa-Application (166 KB) | 3.8 ms | 22.0 ms | **5.8x** |
| AFNetworking (99 KB) | 1.9 ms | 9.1 ms | **4.9x** |
| watch (48 KB) | 0.9 ms | 2.6 ms | **2.8x** |
| project (19 KB) | 0.4 ms | 1.0 ms | **2.7x** |

### Package Size

| | @bacons/xcode | @mozharovsky/xcode |
|-|--------------|-------------------|
| Unpacked | 1.1 MB | 559 KB |
| Gzipped | ~400 KB | ~270 KB |

Run benchmarks yourself:

```bash
make bench        # both Rust + JS
make bench-rust   # pure Rust (no napi overhead)
make bench-js     # Rust vs TypeScript head-to-head
```

## Choosing the Right Function

| Use case | Function | Notes |
|----------|----------|-------|
| Parse only | `parse(text)` | 2-12x faster than TS |
| Build from JS object | `build(obj)` | Fastest on large files (>100 KB) |
| Build from JSON string | `buildFromJSON(json)` | Faster than `build()` on all sizes |
| Full round-trip | `parseAndBuild(text)` | Fastest path, zero JS/Rust overhead |
| Project manipulation | `XcodeProject.open()` | Stays in Rust, use `.toBuild()` to serialize |

## Compatibility

- Full feature parity with `@bacons/xcode/json` (parse/build)
- 13/13 round-trip fixtures produce **byte-identical** output
- All escape sequences: standard (`\n`, `\t`, etc.), Unicode (`\Uxxxx`), octal, NeXTSTEP (128 entries)
- Xcode 16 file system synchronized groups
- Swift Package Manager references
- 106 tests (84 Rust + 22 JS)

## Supported Platforms

| Platform | Architecture |
|----------|-------------|
| macOS | arm64 (Apple Silicon), x64 (Intel) |
| Linux | x64 (glibc), arm64 (glibc) |
| Windows | x64 (MSVC) |

## Development

```bash
# Prerequisites
# - Rust toolchain (cargo)
# - Node.js >= 18

# Install dependencies
npm install

# Run tests
make test           # Rust + JS tests
make test-rust      # Rust tests only (fast, no Node needed)
make test-js        # JS tests (builds debug binary first)

# Build
make build          # Release build
make build-debug    # Debug build (faster compilation)

# Other
make check          # Type-check without building
make fmt            # cargo fmt
make lint           # cargo clippy
make clean          # Remove all artifacts
```

### Project Structure

```
src/
  lib.rs                  # napi exports: parse, build, XcodeProject
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
