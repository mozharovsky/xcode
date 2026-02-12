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

Replace the **old version** with the **new version** in these 7 files:

| File                               | Fields                                            |
| ---------------------------------- | ------------------------------------------------- |
| `Cargo.toml`                       | `version`                                         |
| `package.json`                     | `version` + all 5 `optionalDependencies` versions |
| `npm/darwin-arm64/package.json`    | `version`                                         |
| `npm/darwin-x64/package.json`      | `version`                                         |
| `npm/linux-x64-gnu/package.json`   | `version`                                         |
| `npm/linux-arm64-gnu/package.json` | `version`                                         |
| `npm/win32-x64-msvc/package.json`  | `version`                                         |

### 4. Run npm install

```bash
npm install
```

This syncs `package-lock.json` and `Cargo.lock` with the new versions.

### 5. Show summary and ASK for confirmation

Display:

- The old → new version
- List of files changed
- The exact git commands that will run

**Wait for explicit user approval before proceeding.**

### 6. Stage, commit, tag, and push

```bash
git add Cargo.toml Cargo.lock package.json package-lock.json \
       npm/darwin-arm64/package.json \
       npm/darwin-x64/package.json \
       npm/linux-x64-gnu/package.json \
       npm/linux-arm64-gnu/package.json \
       npm/win32-x64-msvc/package.json

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
