# xcodekit

Native Xcode project automation CLI for AI agents, CI, and developer tooling.

Built in Rust. Single binary, no runtime dependencies. Parses at 1.2 GB/s.

## Install

```bash
cargo install xcodekit
```

Or build from source:

```bash
git clone https://github.com/mozharovsky/xcodekit
cd xcodekit
cargo build --release
# Binary at target/release/xcodekit
```

## Quick Start

```bash
# Inspect a project
xcodekit project inspect ios/App.xcodeproj/project.pbxproj --json

# List targets
xcodekit target list ios/App.xcodeproj/project.pbxproj

# Set a build setting
xcodekit build setting set ios/App.xcodeproj/project.pbxproj \
  --target App --key SWIFT_VERSION --value 6.0 --write

# Add a file to a group
xcodekit file add ios/App.xcodeproj/project.pbxproj \
  --group Sources --file-path Sources/NewFile.swift --write

# Check project health
xcodekit doctor orphans ios/App.xcodeproj/project.pbxproj --json
```

## Commands

### Project

| Command                     | Description                                     |
| --------------------------- | ----------------------------------------------- |
| `project inspect <pbxproj>` | Project summary: targets, object counts, health |
| `project targets <pbxproj>` | List all targets                                |
| `project health <pbxproj>`  | Validate project integrity                      |
| `project dump <pbxproj>`    | Full raw JSON dump                              |

### Targets

| Command                                                                           | Description           |
| --------------------------------------------------------------------------------- | --------------------- |
| `target list <pbxproj>`                                                           | List native targets   |
| `target show <pbxproj> --target <name>`                                           | Show target details   |
| `target rename <pbxproj> --target <name> --new-name <name>`                       | Rename with cascade   |
| `target create-native <pbxproj> --name <n> --product-type <uti> --bundle-id <id>` | Create target         |
| `target list-embedded <pbxproj> --target <name>`                                  | List embedded targets |

### Build Settings

| Command                                                                   | Description            |
| ------------------------------------------------------------------------- | ---------------------- |
| `build setting get <pbxproj> --target <name> --key <KEY>`                 | Get a build setting    |
| `build setting set <pbxproj> --target <name> --key <KEY> --value <VALUE>` | Set a build setting    |
| `build setting remove <pbxproj> --target <name> --key <KEY>`              | Remove a build setting |

### Files and Groups

| Command                                                | Description          |
| ------------------------------------------------------ | -------------------- |
| `file add <pbxproj> --group <name> --file-path <path>` | Add a file reference |
| `group add <pbxproj> --parent <name> --name <name>`    | Create a group       |
| `group list-children <pbxproj> --group <name>`         | List group children  |

### Build Phases and Frameworks

| Command                                                                                | Description            |
| -------------------------------------------------------------------------------------- | ---------------------- |
| `build phase ensure <pbxproj> --target <name> --type <Sources\|Frameworks\|Resources>` | Ensure phase exists    |
| `build phase add-file <pbxproj> --phase <uuid> --file-ref <uuid>`                      | Add file to phase      |
| `framework add <pbxproj> --target <name> --name <Framework>`                           | Add a system framework |

### Dependencies and Extensions

| Command                                                        | Description             |
| -------------------------------------------------------------- | ----------------------- |
| `dependency add <pbxproj> --target <name> --depends-on <name>` | Add target dependency   |
| `extension embed <pbxproj> --host <name> --extension <name>`   | Embed extension in host |

### Validation

| Command                    | Description              |
| -------------------------- | ------------------------ |
| `doctor orphans <pbxproj>` | Find orphaned references |
| `doctor summary <pbxproj>` | Health summary           |

### Sync Groups (Xcode 16+)

| Command                                                       | Description           |
| ------------------------------------------------------------- | --------------------- |
| `sync group add <pbxproj> --target <name> --sync-path <path>` | Add sync group        |
| `sync group list <pbxproj> --target <name>`                   | List sync group paths |

## Output

All commands support `--json` for machine-readable output.

**Default (human-readable):**

```
$ xcodekit target list project.pbxproj
App
AppTests
```

**JSON mode:**

```json
$ xcodekit target list project.pbxproj --json
{
  "targets": [
    { "uuid": "13B07F961A680F5B00A75B9A", "name": "App", "productType": "com.apple.product-type.application" }
  ]
}
```

**Errors** are returned as JSON to stderr with a non-zero exit code:

```json
{ "error": { "code": "TARGET_NOT_FOUND", "message": "Target 'Widget' was not found" } }
```

## Write Modes

Mutating commands are **dry-run by default**. Use `--write` to save changes to disk.

```bash
# Preview (no changes written)
xcodekit build setting set project.pbxproj --target App --key SWIFT_VERSION --value 6.0

# Actually write
xcodekit build setting set project.pbxproj --target App --key SWIFT_VERSION --value 6.0 --write
```

## Name Resolution

Commands accept target and group names, not just UUIDs. If a name is ambiguous, the CLI returns an error with candidates.

## Performance

Pure Rust, benchmarked on Apple M3 Max (`cargo bench`):

| Metric     | swift-protobuf (257 KB) |
| ---------- | ----------------------- |
| Parse      | 0.20 ms (1,250 MB/s)    |
| Build      | 0.58 ms (430 MB/s)      |
| Round-trip | 0.80 ms                 |

## Development

```bash
cargo test          # run all tests
cargo build         # debug build
cargo build --release  # release build
cargo bench         # benchmarks
cargo clippy        # lint
cargo fmt           # format
```

## License

MIT
