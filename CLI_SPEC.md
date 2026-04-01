# XcodeKit CLI Spec

## Goal

Position `xcodekit` as a native **Xcode CLI for AI agents**, not primarily as a JavaScript parser competitor to [`@bacons/xcode`](https://github.com/EvanBacon/xcode).

The core product should be:

- A native Rust CLI
- Deterministic and machine-readable
- Safe for agents and CI
- Able to inspect, mutate, validate, and rewrite Xcode projects and related files

JavaScript / TypeScript support should be a **client layer** over the CLI, not the product center.

## Product Positioning

### What we should be

`xcodekit` should be framed as:

- Xcode automation engine for AI agents
- CI-safe Xcode project editor
- Native command-line toolkit for `.pbxproj`, `.xcscheme`, `.xcworkspace`, `.xcconfig`, and related files

### What we should not optimize the story for

We should not optimize the product story around:

- "slightly faster JS `parse()` than Bacon"
- "drop-in object parser for Node apps"
- "Rust for the sake of Rust"

Those comparisons are noisy and force us into the least defensible part of the stack: Rust -> JS object bridging.

### Why CLI is the right center

CLI-first aligns with our actual strengths:

- Native speed for whole-file workflows
- No JS heap/object bridge for core operations
- Language-agnostic automation
- Great fit for agents, shell scripts, CI, editor extensions, and future MCP integration

## Current Coverage

## Current low-level APIs

The current codebase already supports:

- `parse(text)` -> JS object
- `parseToJSON(text)` -> JSON string
- `build(project)` -> `.pbxproj` text
- `buildFromJSON(json)` -> `.pbxproj` text
- `parseAndBuild(text)` -> round-trip in native memory
- `parsePlist(content)` / `buildPlist(obj)` for plist XML/binary workflows

These are useful compatibility and transport primitives, but they should not define the primary product.

## Current high-level `XcodeProject` APIs

Today the Rust core already supports most of a strong **CLI v1** for `.pbxproj` work:

- Open/parse from file or string
- Serialize back to `.pbxproj`
- Save back to disk
- List all objects / find by ISA
- List native targets
- Find main app target
- Get/set/remove build settings
- Add file references
- Add groups
- Add build files
- Ensure build phases
- Add frameworks
- Add target dependencies
- Create native targets
- Rename targets
- Embed extensions / app clips / watch content
- Add file-system-sync groups (Xcode 16+)
- Inspect sync group paths
- Get/set generic object properties
- Find orphaned references
- Deterministic UUID generation

This is already enough to build a compelling CLI for project inspection and mutation.

## Gap Analysis vs Evan Bacon's Surface

Based on the current feature set documented in [`EvanBacon/xcode`](https://github.com/EvanBacon/xcode), we are currently missing a number of **high-level ecosystems around `.pbxproj`**.

## We already cover well

- Fast `.pbxproj` parse / build
- Mutable project graph operations
- Target creation and rename flows
- Build settings
- Build phases
- File/group manipulation
- Dependency wiring
- Extension embedding
- Xcode 16 file-system-sync groups

## Major missing surfaces

### 1. XCScheme support

Missing:

- parse/build `.xcscheme`
- high-level `XCScheme` API
- create scheme for target
- save scheme
- launch args / env vars / build targets / test configuration support

### 2. Scheme management support

Missing:

- `xcschememanagement.plist`
- scheme ordering
- scheme visibility / hidden / shown state

### 3. XCWorkspace support

Missing:

- parse/build workspace XML
- add/remove projects from workspace
- group structure inside workspace
- workspace create/open/save

### 4. IDEWorkspaceChecks

Missing:

- `IDEWorkspaceChecks.plist`
- workspace warning/computed-state helpers

### 5. XCConfig support

Missing:

- parse/build `.xcconfig`
- `#include` and `#include?`
- flatten build settings
- conditional settings (`sdk`, `arch`, `config`)
- variable expansion / inherited handling

### 6. XCSharedData support

Missing:

- shared data container abstraction
- scheme discovery and save
- shared breakpoints
- workspace settings

### 7. XCUserData support

Missing:

- per-user Xcode data directories
- user schemes
- per-user breakpoints
- scheme management state

### 8. Breakpoints and workspace settings

Missing:

- `Breakpoints_v2.xcbkptlist`
- `WorkspaceSettings.xcsettings`

### 9. Swift Package Manager high-level APIs

Missing:

- add remote Swift package
- add local Swift package
- add package product to target
- remove package product
- query package refs / package products

### 10. Project creation from scratch

Missing:

- create full project scaffolding from scratch
- create project structure beyond single target creation

## Strategic takeaway

We do **not** need to match Bacon feature-for-feature before shipping a CLI.

For CLI-first positioning, the right path is:

- ship a strong `.pbxproj` automation CLI first
- then expand into scheme/workspace/config/shared-data families

Those missing surfaces should be roadmap items, not blockers for CLI v1.

## CLI Design Principles

The CLI should be:

- **Machine-readable first**
- **Dry-run by default** for mutations
- **Deterministic**
- **Composable**
- **Safe for agents**
- **Explicit about writes**

Every command should support:

- `--json`
- `--quiet`
- `--stdin` / `--stdout` where sensible
- `--write` for mutating commands
- `--in-place` alias if desired
- `--backup` optional
- stable exit codes

## CLI Command Model

Recommended top-level structure:

```bash
xcodekit <resource> <action> [options]
```

Examples:

```bash
xcodekit project inspect ios/App.xcodeproj --json
xcodekit target list ios/App.xcodeproj --json
xcodekit build setting set ios/App.xcodeproj --target App SWIFT_VERSION 6.0 --write
xcodekit file add ios/App.xcodeproj --group Sources --path Sources/Foo.swift --write
xcodekit spm add remote ios/App.xcodeproj --url https://github.com/apple/swift-collections --version 1.0.0 --write
xcodekit doctor orphans ios/App.xcodeproj --json
xcodekit plist parse ios/App/Info.plist --json
xcodekit batch ios/App.xcodeproj --write < operations.json
cat project.pbxproj | xcodekit project inspect --json
xcodekit version --json
```

## CLI v1: Must Support

These commands map directly to existing Rust capabilities and should define the first CLI release.

### Project inspection

- `project inspect <pbxproj>`
- `project targets <pbxproj>`
- `project health <pbxproj>`

### Target operations

- `target list <pbxproj>`
- `target show <pbxproj> --target <name|uuid>`
- `target rename <pbxproj> --target <name|uuid> --new-name <name>`
- `target create native <pbxproj> --name <name> --product-type <uti> --bundle-id <id>`
- `target list embedded <pbxproj> --target <name|uuid>`

### Build settings

- `build setting get <pbxproj> --target <name|uuid> --key <KEY>`
- `build setting set <pbxproj> --target <name|uuid> --key <KEY> --value <VALUE>`
- `build setting remove <pbxproj> --target <name|uuid> --key <KEY>`

### Files and groups

- `group list children <pbxproj> --group <name|uuid>`
- `group add <pbxproj> --parent <name|uuid> --name <name>`
- `file add <pbxproj> --group <name|uuid> --path <path>`

### Build phases and frameworks

- `build phase ensure <pbxproj> --target <name|uuid> --type <Sources|Frameworks|Resources|...>`
- `build phase add file <pbxproj> --phase <uuid> --file-ref <uuid>`
- `framework add <pbxproj> --target <name|uuid> --name <Framework>`

### Dependencies / embedding

- `dependency add <pbxproj> --target <name|uuid> --depends-on <name|uuid>`
- `extension embed <pbxproj> --host <name|uuid> --extension <name|uuid>`

### Validation / diagnostics

- `doctor orphans <pbxproj>`
- `doctor summary <pbxproj>`

### Xcode 16 file system sync groups

- `sync group add <pbxproj> --target <name|uuid> --path <path>`
- `sync group list <pbxproj> --target <name|uuid>`

### Swift Package Manager

SPM commands are v1 because `swift-install-tool` is a core agent workflow
and is blocked without them. These require implementing SPM object creation
in the Rust `XcodeProject` API.

- `spm add remote <pbxproj> --url <url> --version <semver>`
- `spm add local <pbxproj> --path <relative-path>`
- `spm add product <pbxproj> --target <name|uuid> --product <name> --package <name|uuid>`
- `spm list <pbxproj>`
- `spm remove product <pbxproj> --target <name|uuid> --product <name>`

### Plist operations

Used by `configure-project.ts` for entitlements and Info.plist editing.
The Rust `plist_xml` module already supports this.

- `plist parse <file> --json`
- `plist build --input <json-file> --output <plist-file>`

### Batch mode

Agent workflows like `swift-add-target-tool` perform 20+ sequential mutations.
Spawning 20 processes with 20 file re-parses is unacceptable. Batch mode accepts
a JSON array of operations and executes them all in a single parse/save cycle.

- `batch <pbxproj> --json < commands.json --write`

Input shape:

```json
[
  { "command": "build setting set", "target": "App", "key": "SWIFT_VERSION", "value": "6.0" },
  { "command": "dependency add", "target": "App", "depends-on": "Widget" },
  { "command": "extension embed", "host": "App", "extension": "Widget" }
]
```

Output: `{ "changed": true, "operationsExecuted": 3 }`

This is critical for agent performance. Without it, complex mutations require
N process spawns and N file re-parses.

### Stdin support

All read commands should accept pbxproj content from stdin when no file path
is given, or when `--stdin` is passed. This supports environments where the
pbxproj comes from a buffer (e.g., piped from cloud storage).

```bash
cat project.pbxproj | xcodekit project inspect --json
curl -s https://storage/project.pbxproj | xcodekit target list --json
```

### Advanced / power user (not core v1 UX)

These are useful for debugging, development, or power users, but should not be
the primary CLI story:

- `project dump <pbxproj>`
- `object get <pbxproj> --uuid <uuid>`
- `object get property <pbxproj> --uuid <uuid> --key <key>`
- `object set property <pbxproj> --uuid <uuid> --key <key> --value <value>`
- `object list by isa <pbxproj> --isa <PBX...>`

## CLI v2: Xcode metadata ecosystem

### XCScheme

- `scheme list <xcodeproj>`
- `scheme show <xcodeproj> --scheme <name>`
- `scheme create <xcodeproj> --name <name> --target <name|uuid>`
- `scheme set env <xcodeproj> --scheme <name> --key <KEY> --value <VALUE>`
- `scheme add arg <xcodeproj> --scheme <name> --arg <arg>`
- `scheme add build target <xcodeproj> --scheme <name> --target <name|uuid>`

### XCWorkspace

- `workspace inspect <xcworkspace>`
- `workspace list projects <xcworkspace>`
- `workspace add project <xcworkspace> --path <project-path>`
- `workspace remove project <xcworkspace> --path <project-path>`
- `workspace create <path>`

## CLI v3: Full Xcode metadata ecosystem (future)

- `xcconfig parse <file>`
- `xcconfig flatten <file>`
- `workspace checks get <xcworkspace>`
- `workspace checks set <xcworkspace> --key <key> --value <value>`
- `breakpoints list <xcodeproj>`
- `breakpoints add <xcodeproj> --file <path> --line <n>`
- `breakpoints remove <xcodeproj> --uuid <uuid>`
- `shared data inspect <xcodeproj>`
- `user data inspect <xcodeproj> --user <username>`
- `workspace settings get <xcworkspace>`
- `workspace settings set <xcworkspace> --key <key> --value <value>`

## Path Resolution

Commands accept either `.xcodeproj` directories or raw `project.pbxproj` paths.
The CLI normalizes internally:

- `ios/App.xcodeproj` resolves to `ios/App.xcodeproj/project.pbxproj`
- `ios/App.xcodeproj/project.pbxproj` used as-is

Agents and humans think in terms of `.xcodeproj`. The CLI should not force
them to append `/project.pbxproj` every time.

## Version and Capabilities

- `xcodekit version` -- print version string
- `xcodekit version --json` -- `{ "version": "0.7.0" }`

This helps agents adapt behavior without hardcoding assumptions about which
commands are available.

## Name and UUID Resolution

The CLI should be name-friendly, not UUID-first.

For any command that accepts a target/group/file/object reference:

- if input is a 24-char hex string, treat as UUID
- otherwise resolve by name
- if multiple matches exist, return an ambiguity error with candidates

Example ambiguity output:

```json
{
  "ok": false,
  "error": {
    "code": "AMBIGUOUS_REFERENCE",
    "message": "Multiple targets matched 'App'",
    "candidates": [
      { "uuid": "AAA...", "name": "App" },
      { "uuid": "BBB...", "name": "AppTests" }
    ]
  }
}
```

## Output Contract

### Principle

- On success: return the payload directly. No wrapper envelope.
- On failure: return `{ "error": ... }` and exit non-zero.
- Use **exit codes** for success/failure, not JSON booleans.

### Human mode (default, no `--json`)

Commands print concise human-readable text to stdout.
Errors go to stderr.

```
$ xcodekit target list ios/App.xcodeproj
App
AppTests
Widget

$ xcodekit build setting set ios/App.xcodeproj --target App SWIFT_VERSION 6.0 --write
Set SWIFT_VERSION = 6.0 on target App (2 configurations updated)

$ xcodekit version
xcodekit 0.7.0
```

### JSON mode (`--json`)

#### Inspection commands

Return the data directly:

```json
{
  "targets": [
    {
      "uuid": "13B07F961A680F5B00A75B9A",
      "name": "App",
      "productType": "com.apple.product-type.application"
    }
  ]
}
```

#### Mutation commands

Return a short confirmation:

```json
{
  "changed": true
}
```

`changed` tells the agent whether the project was actually modified. If the
value was already set to what was requested, `changed` is `false` and no
file write occurs even with `--write`. The agent infers write status from
`changed` + whether it passed `--write`.

#### Errors

Return a structured error object and exit non-zero:

```json
{
  "error": {
    "code": "TARGET_NOT_FOUND",
    "message": "Target 'Widget' was not found"
  }
}
```

## Write Modes

All mutating commands should support the same write model:

### Default

Dry-run:

- perform the mutation
- return result as JSON
- do not write to disk

### `--write`

- write back to the original file
- atomic temp-file replace if possible

### `--stdout`

- print rewritten `.pbxproj` to stdout
- useful for pipes / review / custom tooling

### `--backup`

- write a `.bak` copy before replacement

## Error Taxonomy

Recommended stable error codes:

- `FILE_NOT_FOUND`
- `PARSE_ERROR`
- `INVALID_ARGUMENT`
- `UNSUPPORTED_OPERATION`
- `TARGET_NOT_FOUND`
- `GROUP_NOT_FOUND`
- `OBJECT_NOT_FOUND`
- `AMBIGUOUS_REFERENCE`
- `WRITE_FAILED`
- `VALIDATION_FAILED`

## Practical Semantics

The CLI should optimize for the questions and actions an agent actually has.

### `project inspect`

This is a summary command, not a raw dump. It should return:

- project path
- archive/object version
- root object UUID
- main/product groups
- target list with UUID, name, product type
- object counts
- orphan count
- feature flags (SPM refs, sync groups, extensions, etc.)

### `project targets`

This is a convenience alias focused just on target discovery. It should return:

- UUID
- name
- product type
- bundle identifier if cheaply available
- whether it looks like the main app target

### `project health`

This is a high-signal validation command. It should return:

- orphaned reference count
- malformed or suspicious structures
- missing root references
- optionally warnings for weird but parseable states

### `project dump`

This is the escape hatch for full structural dumps. It is intentionally
positioned as a lower-level command, not a primary project workflow.

### Why `roundtrip` was removed

`roundtrip` is a maintainer/debugging primitive, not a practical agent command.
If we want this functionality at all, it belongs under a hidden `dev` namespace
or inside `doctor`, not in the main CLI UX.

### Why raw `object find-by-isa` was demoted

It is too implementation-shaped. Agents usually want semantic actions like:

- list targets
- show build settings
- add file
- embed extension
- inspect health

Raw object queries can remain as advanced/debug affordances, but they should not
define the product surface.

## Migration Strategy

Based on real production usage analysis across 10 files importing `@xcodekit/xcode-wasm`.

### Phase 1: CLI replaces read-only sandbox tools

These tools only inspect targets and read build settings. Direct CLI replacement
with `execa` + JSON parsing.

| Tool                  | Difficulty | Blocker? |
| --------------------- | ---------- | -------- |
| `sign-archive.ts`     | Easy       | No       |
| `image-gen/tools.ts`  | Easy       | No       |
| `rork-apple-agent.ts` | Easy       | No       |

### Phase 2: CLI replaces mutation tools

These tools set build settings, create targets, embed extensions. Requires
`--write` and ideally batch mode to avoid N sequential process spawns.

| Tool                       | Difficulty | Blocker?                                       |
| -------------------------- | ---------- | ---------------------------------------------- |
| `configure-project.ts`     | Moderate   | Batch mode helps perf                          |
| `swift-add-target-tool.ts` | Hard       | Needs low-level object commands for edge cases |
| `swift-install-tool.ts`    | Hard       | Blocked on SPM commands (now v1)               |

### Phase 3: Resolve API server constraint

`projects.ts` and `appstore-v2.ts` run on Cloudflare Workers where you cannot
spawn child processes. Options:

- Move the logic into E2B sandboxes where the CLI runs (preferred)
- Keep a minimal read-only WASM binding for CF Workers only
- Add an HTTP API mode to the CLI (overkill for now)

### Runtime constraints

| Environment          | CLI works? | Strategy                                   |
| -------------------- | ---------- | ------------------------------------------ |
| E2B sandbox          | Yes        | Primary target                             |
| CI / shell scripts   | Yes        | Primary target                             |
| Node.js / Bun server | Yes        | Spawn via execa                            |
| Cloudflare Workers   | No         | Move logic to sandbox or keep minimal WASM |
| Browser              | No         | Not a target                               |

### TS SDK strategy

If a TS SDK is needed, it should be a thin typed wrapper over CLI invocation:

```typescript
async function xcodekit(args: string[]): Promise<unknown> {
  const { stdout } = await execa("xcodekit", [...args, "--json"]);
  return JSON.parse(stdout);
}
```

The SDK should not re-implement project logic. It is a transport client.

For batch operations, the SDK can pipe a JSON array to `xcodekit batch`:

```typescript
async function xcodekit_batch(pbxproj: string, ops: object[]): Promise<void> {
  await execa("xcodekit", ["batch", pbxproj, "--write", "--json"], {
    input: JSON.stringify(ops),
  });
}
```

## Summary

CLI v1 must include:

- All `.pbxproj` inspection and mutation commands (targets, build settings,
  files, groups, build phases, frameworks, dependencies, extensions, sync groups)
- SPM commands (add remote, add local, add product, list, remove)
- Plist parse/build
- Batch mode for multi-operation workflows
- Stdin support for buffer-based inputs
- `--json` on every command
- `--write` / dry-run model for all mutations

This is enough to replace `@xcodekit/xcode-wasm` in all E2B sandbox tools
and most server-side tools. CF Workers read-only usage is the only remaining
constraint, addressable by moving logic into sandboxes.

The winning strategy is:

- CLI as the core product
- TS SDK as a thin CLI wrapper (separate repo)
- WASM kept only if CF Workers constraint persists
