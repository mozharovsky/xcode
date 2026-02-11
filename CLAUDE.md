# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

`@xcodekit/xcode` — a native Rust parser/serializer for Xcode `.pbxproj` files, exposed to Node.js via napi-rs. Published to npm as a drop-in replacement for `@bacons/xcode/json`.

## Commands

```bash
# Rust tests (fast, no Node required)
make test-rust          # or: cargo test --no-default-features

# JS integration tests (builds debug binary first)
make test-js            # or: npx napi build --platform && npx ava __test__/index.spec.mjs

# Build native .node binary
make build              # release: npx napi build --platform --release
make build-debug        # debug: npx napi build --platform

# Check compilation without codegen
make check              # checks both with and without napi feature

# Benchmarks
make bench-rust         # pure Rust: cargo bench --no-default-features --bench parse_build
make bench-js           # Rust vs TS: node benches/benchmark.mjs

# Other
make fmt                # cargo fmt
make lint               # cargo clippy --no-default-features -- -D warnings
make clean              # removes target/, *.node, index.js, index.d.ts
```

## Architecture

### Two-Layer Design

**Pure Rust library** (works without Node.js, tested with `--no-default-features`):

- `src/parser/` — lexer + recursive descent parser → `PlistValue`
- `src/writer/` — `PlistValue` → .pbxproj text
- `src/project/` — `XcodeProject` high-level container
- `src/types/` — `PlistValue` enum, ISA enum, constants
- `src/objects/` — `PbxObject` generic wrapper with ISA-based dispatch

**napi bindings** (feature-gated behind `napi` feature):

- `src/lib.rs` — `#[cfg(feature = "napi")] mod napi_bindings` wraps the pure Rust API

### Key Design Decisions

**napi is optional.** The `napi` feature is the default but can be disabled. All Rust tests run with `--no-default-features` to avoid needing Node.js for linking. The `[lib]` section has `crate-type = ["cdylib", "lib"]` — `cdylib` for the .node binary, `lib` for Rust tests.

**No object inflation.** Unlike the TypeScript original which inflates UUID strings into live object pointers (creating circular reference graphs), all references stay as UUID strings. Lookups go through `XcodeProject`'s flat `IndexMap<String, PbxObject>`. This avoids `Rc<RefCell<>>` complexity.

**One generic `PbxObject` for all 29 ISA types.** Instead of 29 separate structs, `PbxObject` stores props as `IndexMap<String, PlistValue>` and uses `reference_keys()` with ISA-based match to know which properties contain UUID references.

**`serde_json` with `preserve_order`.** Critical for round-trip fidelity — without it, JSON serialization reorders keys alphabetically, breaking pbxproj output. The `preserve_order` feature makes `serde_json::Value` use `IndexMap` internally.

**Comment generation uses a reverse index.** The `create_reference_list` function pre-builds a `HashMap<build_file_uuid, (phase_isa, phase_name)>` to avoid O(n\*m) scanning for each build file's containing phase.

### Data Flow

```
parse(text)        → Lexer → Parser → PlistValue → serde → JS object
build(obj)         → serde → PlistValue → Writer → string
parseAndBuild(text)→ Lexer → Parser → PlistValue → Writer → string  (no serde)
XcodeProject.open()→ file → parse → PbxObject map (stays in Rust)
```

The `XcodeProject` path is the fastest because data never crosses the JS/Rust FFI boundary. Methods like `setBuildSetting`, `addFile`, `addFramework` operate on the Rust-side `PbxObject` map, passing only primitive strings across napi.

### Performance-Critical Code

- **Lexer** (`src/parser/lexer.rs`): Direct byte scanning, no per-character function calls, no line/column tracking. This was the single biggest optimization (38x speedup).
- **Serializer** (`src/writer/serializer.rs`): Pre-computed indent cache, direct buffer writes via `write_ensure_quotes_to()`, `BTreeMap` for ISA sorting.
- **Comment O(1) lookup** (`src/writer/comments.rs`): Reverse index `file_to_phase` built in one pass.

### Test Fixtures

20 `.pbxproj` files in `__test__/fixtures/` from real-world projects. 13 of these are "round-trip fixtures" — parse → build must produce byte-identical output. The `malformed.pbxproj` fixture has an intentional orphaned reference (UUID `3E1C2299F05049539341855D`) for testing graceful error handling.

Fixture files must have LF line endings (enforced by `.gitattributes`) for round-trip tests to pass on Windows.

## Releasing

Pushing a version tag triggers the publish workflow (`.github/workflows/publish.yml`):

```bash
git tag v0.2.0
git push origin v0.2.0
```

This builds all 5 platform binaries, runs tests, publishes platform packages (`@xcodekit/xcode-darwin-arm64`, etc.) to npm, then publishes the main `@xcodekit/xcode` package, and creates a GitHub Release.

**Before tagging:** update `version` in `package.json` and all `npm/*/package.json` files to match the tag. The `optionalDependencies` versions in the root `package.json` must also match.

**Secrets required:** `NPM_TOKEN` (granular access token with "Bypass 2FA" enabled, read-write on packages and the `xcodekit` org).

**Cross-compilation:** The `aarch64-unknown-linux-gnu` target uses `--zig` flag for cross-compiling on an x86 Ubuntu runner.

### Generated Files (gitignored)

`index.js`, `index.d.ts`, and `*.node` are generated by `napi build` — not checked in. CI regenerates them.
