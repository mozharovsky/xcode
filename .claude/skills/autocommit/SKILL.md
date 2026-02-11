---
name: autocommit
description: Automatically commit staged changes using conventional commit format with safety checks. Use when you need a structured, conventional commit with scopes.
disable-model-invocation: true
allowed-tools: Bash(git diff *), Bash(git log *), Bash(git status *), Bash(git commit *)
---

Automatically commit staged changes using conventional commit format.

## CRITICAL SAFETY RULES - NEVER VIOLATE THESE

**FORBIDDEN COMMANDS - NEVER RUN THESE:**

- `git reset --hard` - destroys uncommitted changes
- `git checkout .` or `git checkout -- <file>` - discards changes
- `git clean` - deletes untracked files
- `git stash drop` - loses stashed changes
- `git push --force` or `git push -f` - rewrites remote history
- `git rebase` without explicit user request
- Any command with `--hard`, `--force`, `-f` flags unless explicitly requested
- `git add .` or `git add -A` - user controls what is staged

**ONLY ALLOWED GIT COMMANDS:**

- `git diff --cached` or `git diff --staged` - view staged changes
- `git log` - read commit history
- `git status` - check repository state
- `git commit -m "..."` - commit with message (ONLY after user approval)

## Instructions

1. **Read staged changes ONLY:**

   ```bash
   git diff --cached --stat
   git diff --cached
   ```

2. **Read recent commit history for style reference:**

   ```bash
   git log --oneline -20
   ```

3. **Generate a conventional commit message** following this format:

   ```
   <type>(<scope>): <short description>

   <optional body with more details>
   ```

   **Types:**

   - `feat` - new feature
   - `fix` - bug fix
   - `refactor` - code refactoring without changing functionality
   - `chore` - maintenance tasks, dependencies
   - `docs` - documentation changes
   - `style` - formatting, whitespace (no code change)
   - `test` - adding/updating tests
   - `perf` - performance improvements
   - `ci` - CI/CD changes

   **Scopes (project-specific):**

   - `parser` - lexer, parser, escape handling (`src/parser/`)
   - `writer` - serializer, comments, quoting (`src/writer/`)
   - `project` - XcodeProject API, build settings, paths, UUIDs (`src/project/`)
   - `types` - plist types, ISA definitions, constants (`src/types/`)
   - `objects` - pbxproj object model (`src/objects/`)
   - `napi` - Node.js N-API bindings (`src/lib.rs`)
   - `tests` - integration tests and JS specs (`tests/`, `__test__/`)
   - `bench` - benchmarks (`benches/`)
   - `ci` - GitHub Actions workflows (`.github/`)
   - `npm` - npm packaging and platform targets (`npm/`, `package.json`)

4. **Show the proposed commit message and ASK for confirmation:**

   - Display the exact command that will be run
   - Wait for explicit user approval before running `git commit`

5. **After successful commit:**
   - Show `git status` to confirm the commit
   - Do NOT push unless explicitly requested
