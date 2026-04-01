# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

`xcodekit` -- a native Rust CLI for Xcode project automation. Parses, manipulates, and serializes `.pbxproj` files. Designed for AI agents, CI, and developer tooling.

## Commands

```bash
# Run all tests
cargo test

# Build release binary
cargo build --release

# Benchmarks
cargo bench --bench parse_build

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt
```

## Architecture

### Pure Rust Library

- `src/parser/` -- single-pass lexer + recursive descent parser producing `PlistValue`
- `src/writer/` -- `PlistValue` to `.pbxproj` text serializer with comment generation
- `src/project/` -- `XcodeProject` high-level container with mutation API
- `src/types/` -- `PlistValue` enum (with `Cow<'a, str>` zero-copy strings), ISA enum, constants
- `src/objects/` -- `PbxObject` generic wrapper with ISA-based dispatch
- `src/plist_xml.rs` -- XML/binary plist parsing via the `plist` crate

### CLI Binary

- `cli/main.rs` -- clap app definition, top-level command dispatch
- `cli/commands/` -- one module per command group (project, target, build_setting, etc.)
- `cli/output.rs` -- JSON/human output helpers, error formatting
- `cli/resolve.rs` -- name-to-UUID resolution for targets and groups

### Key Design Decisions

**No JS/npm surface.** This is a pure Rust project. No napi, no wasm, no npm packages.

**Zero-copy parsing.** `PlistValue<'a>` uses `Cow<'a, str>` to borrow strings directly from input. Only escaped strings allocate. Object keys are also `Cow`.

**Vec-based objects.** `PlistValue::Object` stores `Vec<(Cow<str>, PlistValue)>` instead of a hash map. No hashing during parse. `PbxObject` converts to `IndexMap` (with ahash) when created for efficient key lookups.

**One generic `PbxObject` for all 29 ISA types.** Properties stored as `IndexMap<Cow<str>, PlistValue>` with ISA-based `reference_keys()` dispatch.

**`serde_json` with `preserve_order`.** Critical for round-trip fidelity.

**Comment generation uses a reverse index.** `create_reference_list` pre-builds a `HashMap<build_file_uuid, (phase_isa, phase_name)>`.

### Data Flow

```
CLI command
  → XcodeProject::open(path)
    → parser::parse(text) → PlistValue (zero-copy, Vec-based objects)
    → XcodeProject::from_plist_value() → IndexMap<String, PbxObject>
  → mutation (setBuildSetting, addFile, etc.)
  → XcodeProject::to_pbxproj()
    → to_plist() → PlistValue
    → writer::serializer::build() → String
  → write to file (if --write)
```

### Test Fixtures

20 `.pbxproj` files in `tests/fixtures/` from real-world projects. 13 of these are round-trip fixtures -- parse then build must produce byte-identical output.

Fixture files must have LF line endings (enforced by `.gitattributes`).
