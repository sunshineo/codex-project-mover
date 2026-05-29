# Git Worktree Auto-Repair Design

Date: 2026-05-29

## Context

`codex-project-mover` currently moves a project by copying the old project folder to the new path, verifying the copied tree, updating Codex metadata, and moving the old folder to Trash. That is safe for ordinary folders, but Git worktrees contain path-sensitive metadata.

Git uses "worktree" to mean a checkout directory with project files. The Codex project path should always point at a checkout root, never at Git's internal `.git/worktrees/...` metadata directory.

There are two relevant Git cases:

- Main worktree: the original checkout root, usually with `.git` as a directory.
- Linked worktree: a secondary checkout root, usually with `.git` as a text file pointing at metadata under the main worktree's `.git/worktrees/...` directory.

Moving either path with generic filesystem operations can leave Git metadata pointing at old absolute paths. The mover should detect these cases and repair them automatically.

## Goal

Make Git worktree repair automatic during `plan`, `apply`, `relink-only`, and `verify` while preserving the current safety model:

- `--old` and `--new` remain project checkout roots.
- Non-Git projects keep the current behavior.
- Git repair runs by default when a Git worktree layout is detected.
- The tool stops before moving the old folder to Trash if Git repair or verification fails.
- Output shows the detected Git worktree layout and any repair commands that were run or should be run manually.

## Non-Goals

- Do not point Codex metadata at `.git/worktrees/...`.
- Do not implement a general Git repository migration tool.
- Do not silently enable `worktree.useRelativePaths` or `git worktree repair --relative-paths` by default.
- Do not rewrite arbitrary Git internals manually when Git's own commands can do the repair.

## Proposed Architecture

Add a `git_worktree` module with these responsibilities:

- Detect whether a project path is inside a Git checkout by running Git commands from that path.
- Inspect the checkout's `.git` filesystem entry directly before repair: a directory usually means a main worktree, and a `gitdir: ...` file usually means a linked worktree.
- Classify the project path as `NotGit`, `MainWorktree`, or `LinkedWorktree`.
- Read `git worktree list --porcelain -z` into structured entries containing path, HEAD, branch, detached state, bare state, locked state, and prunable state when present.
- Map worktree paths under the old project root to their expected new paths.
- Build a repair plan that can be displayed by `plan`, executed by `apply`, and validated by `verify`.

The module should use `std::process::Command` and fail with clear command, cwd, exit status, stderr, and stdout context when Git commands fail.

## Command Behavior

### `plan`

`plan` should keep reporting Codex metadata matches, then add a Git section when Git is detected:

- Classification of `--old`: main worktree or linked worktree.
- Worktrees known to Git.
- Which worktree paths are expected to change.
- Whether `apply` will use generic copy plus `git worktree repair`, or `git worktree move`.

If Git is not detected, `plan` should say that no Git worktree repair is needed.

### `apply`

When `--old` is not a Git checkout, keep the current copy, verify, metadata update, and Trash flow.

When `--old` is the main worktree:

1. Copy the old project folder to the new path.
2. Verify the copied tree.
3. Run `git -C <new> worktree repair` with mapped linked-worktree paths when there are affected linked worktrees.
4. Verify the Git worktree layout from the new path.
5. Update Codex metadata.
6. Verify Codex metadata.
7. Move the old folder to Trash.

When `--old` is itself a linked worktree:

1. Resolve a stable Git cwd from the linked worktree's `.git` pointer.
2. Run `git worktree move <old> <new>` from that Git cwd.
3. Verify the moved linked worktree with `git -C <new> status --short`.
4. Update Codex metadata.
5. Verify Codex metadata.

For linked worktree moves, do not generic-copy the directory first. Git's native move operation updates the main repository's worktree metadata.

If Git repair or verification fails, `apply` should stop before moving the old path to Trash and print the manual command the user can try.

### `relink-only`

`relink-only` means the user already moved the project folder manually. It should:

1. Validate the old path is missing and the new path exists, as today.
2. Inspect `new/.git` directly before running Git, because `git -C <new>` may fail when a manually moved linked worktree still points at old metadata.
3. Run `git -C <new> worktree repair` when the new path is a main worktree.
4. Run `git worktree repair <new>` from the main repo when the new path is a linked worktree and the main repo can be resolved from the `.git` pointer.
5. Continue with Codex metadata updates and verification.

If the main repo cannot be found for a moved linked worktree, the tool should fail before Codex metadata updates and print the manual repair command. This keeps the default auto-repair promise honest instead of silently accepting broken Git state.

### `verify`

`verify` should keep its current Codex metadata checks and add Git validation when Git is detected:

- `git -C <new> worktree list --porcelain -z` succeeds.
- No relevant worktree entry points to the old path.
- Relevant moved worktrees are not marked prunable.
- `git -C <new> status --short` succeeds.
- For mapped linked worktrees that still exist, `git -C <linked-path> status --short` succeeds.

Git verification failures should make `verify` fail with a clear message.

## Error Handling

The Git worktree flow should distinguish:

- Git not installed: fail only when the path looks like a Git checkout and Git commands are required.
- Not a Git checkout: continue with existing non-Git behavior.
- Dirty worktree: do not block solely because files are dirty; moving a project folder should preserve dirty files.
- Locked worktree: report it in `plan`; allow repair if Git allows it, otherwise fail with Git's error.
- Prunable worktree: fail verification and tell the user to inspect or prune manually.
- Submodules: treat them as ordinary content for this feature unless the project root itself is the submodule checkout.

## Testing

Add focused tests around the new `git_worktree` module:

- Detect non-Git paths.
- Detect a main worktree.
- Detect a linked worktree.
- Parse `git worktree list --porcelain -z`.
- Map linked worktree paths under an old root to a new root.

Add CLI integration tests that create temporary Git repositories:

- Moving a main worktree with no linked worktrees preserves Git status.
- Moving a main worktree with a linked worktree under the moved tree repairs the linked worktree path.
- Moving a linked worktree uses `git worktree move` and preserves Git status.
- `relink-only` repairs a manually moved main worktree.
- `verify` fails when Git still reports an old worktree path.

Tests should skip with a clear reason if `git` is unavailable.

## Open Design Decisions

No product decisions remain open. The default behavior is automatic Git worktree repair.

An optional `--git-relative-worktrees` flag can be considered later, but it is intentionally outside this design because it changes Git extension behavior and may affect compatibility with older Git clients.
