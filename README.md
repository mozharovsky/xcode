# xcodekit

Native Xcode project manipulation from the command line. Built in Rust. Single binary, no dependencies, no Xcode required. Parses at 1.6 GB/s.

Designed for **AI coding agents**, **CI pipelines**, and **shell scripts** that need to inspect and mutate `.pbxproj` files without touching Xcode.

```bash
brew install mozharovsky/tap/xcodekit
```

or `cargo install --git https://github.com/mozharovsky/xcode`

## Why

Xcode projects are notoriously hard to automate. The `.pbxproj` format is a proprietary plist dialect with no official tooling outside Xcode itself. Existing solutions are either slow (Ruby/Swift), tied to a runtime (Node.js), or require Docker.

**xcodekit** is a standalone binary that parses, mutates, and rewrites `.pbxproj` files at native speed. Every command outputs structured JSON, supports dry-run by default, and returns typed error codes. This makes it trivial for AI agents and scripts to operate on Xcode projects without surprises.

## Quick Start

```bash
# Inspect a project
xcodekit project inspect App.xcodeproj --json

# List targets
xcodekit target list App.xcodeproj

# Set a build setting (dry-run by default)
xcodekit build setting set App.xcodeproj --target MyApp --key SWIFT_VERSION --value 6.0

# Actually write to disk
xcodekit build setting set App.xcodeproj --target MyApp --key SWIFT_VERSION --value 6.0 --write

# Add a Swift package
xcodekit spm add-remote App.xcodeproj \
  --url https://github.com/apple/swift-collections --version 1.0.0 --write
```

## Performance

Pure Rust parser, benchmarked on Apple M5 Max (5,000 iterations, median):

| Metric     | swift-protobuf.pbxproj (257 KB) |
| ---------- | ------------------------------- |
| Parse      | 0.15 ms (1,658 MB/s)            |
| Build      | 0.42 ms (599 MB/s)              |
| Round-trip | 0.58 ms                         |

## Key Features

**Safe by default** — all mutations are dry-run unless you pass `--write`. Preview what would change before committing.

**Structured output** — every command supports `--json`. Errors go to stderr with typed codes:

```json
{ "error": { "code": "TARGET_NOT_FOUND", "message": "Target 'Widget' was not found" } }
```

**Batch mode** -- execute multiple operations in a single parse/save cycle. Pipe a JSON array to stdin:

```bash
echo '[
  {"command": "build setting set", "target": "App", "key": "SWIFT_VERSION", "value": "6.0"},
  {"command": "build setting set", "target": "App", "key": "IPHONEOS_DEPLOYMENT_TARGET", "value": "17.0"},
  {"command": "framework add", "target": "App", "name": "StoreKit"},
  {"command": "build phase add script", "target": "App", "name": "SwiftLint", "script": "swiftlint"}
]' | xcodekit batch App.xcodeproj --write --json
```

**Name resolution** — pass target and group names, not UUIDs. Ambiguous names return an error with candidates.

**Path normalization** — pass `App.xcodeproj` or `App.xcodeproj/project.pbxproj`, both work.

**Stdin support** — pipe `.pbxproj` content directly for inspection (use `-` as the path).

## Commands

### Inspection

```
project inspect  <pbxproj>                                          # Project summary
project targets  <pbxproj>                                          # List all targets
project health   <pbxproj>                                          # Validate integrity
project dump     <pbxproj>                                          # Full raw JSON dump
doctor orphans   <pbxproj>                                          # Find orphaned refs
doctor summary   <pbxproj>                                          # Health summary
```

### Targets

```
target list           <pbxproj>                                     # List native targets
target show           <pbxproj> --target <name>                     # Show target details
target rename         <pbxproj> --target <name> --new-name <n>      # Rename with cascade
target create-native  <pbxproj> --name <n> --product-type <uti> --bundle-id <id>
target duplicate      <pbxproj> --target <name> --new-name <n>      # Deep-clone target
target list-embedded  <pbxproj> --target <name>                     # List embedded exts
```

### Build Settings

```
build setting get     <pbxproj> --target <name> --key <KEY>
build setting set     <pbxproj> --target <name> --key <KEY> --value <VAL>
build setting remove  <pbxproj> --target <name> --key <KEY>
```

### Files and Groups

```
file add              <pbxproj> --group <name> --file-path <path>
file remove           <pbxproj> --file <name-or-uuid>
file add-folder       <pbxproj> --target <name> --folder <path> --group <name>
group add             <pbxproj> --parent <name> --name <name>
group remove          <pbxproj> --group <name>
group list-children   <pbxproj> --group <name>
```

### Build Phases

```
build phase ensure     <pbxproj> --target <name> --type <sources|frameworks|resources|headers>
build phase add-file   <pbxproj> --phase <uuid> --file-ref <uuid>
build phase add-script <pbxproj> --target <name> --name <n> --script <body> [--shell /bin/sh]
```

### Frameworks and Dependencies

```
framework add    <pbxproj> --target <name> --name <Framework>
dependency add   <pbxproj> --target <name> --depends-on <name>
extension embed  <pbxproj> --host <name> --extension <name>
```

### Swift Package Manager

```
spm add-remote      <pbxproj> --url <url> --version <ver>
spm add-local       <pbxproj> --package-path <path>
spm add-product     <pbxproj> --target <name> --product <name> --package <name>
spm remove-product  <pbxproj> --target <name> --product <name>
spm list            <pbxproj>
```

### Schemes (.xcscheme)

```
scheme list              <xcodeproj>                                          # List all schemes
scheme show              <xcodeproj> --scheme <name>                          # Dump scheme as JSON
scheme create            <xcodeproj> --name <name> --target <name>            # Create minimal scheme
scheme set-env           <xcodeproj> --scheme <name> --key <K> --value <V>    # Set env var
scheme add-arg           <xcodeproj> --scheme <name> --arg <arg>              # Add launch argument
scheme add-build-target  <xcodeproj> --scheme <name> --target <name> ...      # Add build target
```

### Workspaces (.xcworkspace)

```
workspace inspect         <xcworkspace>                                       # List contents
workspace list-projects   <xcworkspace>                                       # List project refs
workspace add-project     <xcworkspace> --project-path <path>                 # Add project
workspace remove-project  <xcworkspace> --project-path <path>                 # Remove project
workspace create          <path>                                              # Create empty workspace
```

### XCConfig (.xcconfig)

```
xcconfig parse    <file>                                                      # Parse to JSON
xcconfig flatten  <file>                                                      # Resolve to key-value pairs
```

### Breakpoints

```
breakpoint list    <xcodeproj-or-file>                                        # List all breakpoints
breakpoint add     <xcodeproj-or-file> --file <path> --line <n>               # Add file breakpoint
breakpoint remove  <xcodeproj-or-file> --uuid <uuid>                          # Remove breakpoint
```

### Plist

```
plist parse  <file>                                                           # XML/binary plist to JSON
plist build  --input <json> --output <file>                                   # JSON to XML plist
```

### Advanced

```
object get           <pbxproj> --uuid <uuid>                                  # Inspect any object
object get-property  <pbxproj> --uuid <uuid> --key <key>                      # Read a property
object set-property  <pbxproj> --uuid <uuid> --key <k> --value <v>            # Write a property
object list-by-isa   <pbxproj> --isa <ISA>                                    # List by type
sync group add       <pbxproj> --target <name> --sync-path <p>                # Xcode 16+ file sync
sync group list      <pbxproj> --target <name>
```

All mutation commands support `--write` (save to disk) and `--json` (structured output).

## For AI Agents

xcodekit is designed as a tool AI agents call via `execa`, `subprocess`, or shell. Key conventions:

- `--json` on every command for structured, parseable output
- Typed error codes in stderr (`TARGET_NOT_FOUND`, `PARSE_ERROR`, `FILE_NOT_FOUND`, etc.)
- Non-zero exit codes on failure
- `--write` is explicit -- agents can preview changes safely
- `batch` command for multi-step mutations in one process spawn
- Stdin support (`-`) for piping `.pbxproj` content without temp files

## Install

**Homebrew** (macOS / Linux):

```bash
brew install mozharovsky/tap/xcodekit
```

**Cargo** (build from source):

```bash
cargo install --git https://github.com/mozharovsky/xcode
```

Prebuilt binaries for macOS (x86_64, aarch64) and Linux (x86_64, aarch64) are available on the [Releases](https://github.com/mozharovsky/xcode/releases) page.

## Development

```bash
cargo test           # 181 tests
cargo bench          # parse/build benchmarks
cargo clippy         # lint
cargo fmt            # format
```

## License

MIT
