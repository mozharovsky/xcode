---
name: autorelease
description: Bump version across all package files, run npm install, commit, tag, and push. Use when the user wants to release a new version, bump versions, or tag a release.
---

# Autorelease

Bump version, commit, tag, and push — in one command.

## CRITICAL SAFETY RULES

**FORBIDDEN COMMANDS — NEVER RUN THESE:**

- `git reset --hard` — destroys uncommitted changes
- `git push --force` or `git push -f` — rewrites remote history
- `git clean` — deletes untracked files
- Any command with `--hard`, `--force`, `-f` flags unless explicitly requested

## Instructions

### 1. Determine the target version

The user provides the version (e.g. `0.6.0`). If they say "bump patch/minor/major", read the current version from `Cargo.toml` and calculate the next one.

### 2. Check for clean working tree

```bash
git status
```

If there are uncommitted changes, **STOP** and warn the user. Do not proceed with dirty state.

### 3. Update version in all files

Replace **all occurrences** of the old version with the new version. The easiest approach:

```bash
# Find all files containing the old version
grep -rl '"OLD_VERSION"' Cargo.toml package.json npm/
```

Files that need updating (11 total):

| File                                                    | Fields                                              |
| ------------------------------------------------------- | --------------------------------------------------- |
| `Cargo.toml`                                            | `version`                                           |
| `package.json`                                          | `version`                                           |
| `npm/xcode/package.json`                                | `version` + `dependencies` + `optionalDependencies` |
| `npm/xcode-node/package.json`                           | `version` + `optionalDependencies`                  |
| `npm/xcode-wasm/package.json`                           | `version`                                           |
| `npm/xcode-node/platforms/darwin-arm64/package.json`    | `version`                                           |
| `npm/xcode-node/platforms/darwin-x64/package.json`      | `version`                                           |
| `npm/xcode-node/platforms/linux-x64-gnu/package.json`   | `version`                                           |
| `npm/xcode-node/platforms/linux-arm64-gnu/package.json` | `version`                                           |
| `npm/xcode-node/platforms/win32-x64-msvc/package.json`  | `version`                                           |

**Tip:** For `npm/xcode/package.json` and `npm/xcode-node/package.json`, use `replace_all` since the version appears in multiple fields (version + dependency versions).

### 4. Run npm install

```bash
npm install
```

This syncs `package-lock.json` with the new versions.

### 5. Show summary and ASK for confirmation

Display:

- The old → new version
- List of files changed
- The exact git commands that will run

**Wait for explicit user approval before proceeding.**

### 6. Stage, commit, tag, and push

```bash
git add Cargo.toml Cargo.lock package.json package-lock.json \
       npm/xcode/package.json \
       npm/xcode-node/package.json \
       npm/xcode-wasm/package.json \
       npm/xcode-node/platforms/darwin-arm64/package.json \
       npm/xcode-node/platforms/darwin-x64/package.json \
       npm/xcode-node/platforms/linux-x64-gnu/package.json \
       npm/xcode-node/platforms/linux-arm64-gnu/package.json \
       npm/xcode-node/platforms/win32-x64-msvc/package.json

git commit -m "chore: bump version to X.Y.Z"

git tag vX.Y.Z

git push origin main
git push origin vX.Y.Z
```

### 7. Confirm success

```bash
git status
git log --oneline -1
```

Report the commit hash, tag, and that the publish workflow should now be running.
