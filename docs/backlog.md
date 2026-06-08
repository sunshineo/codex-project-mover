# Backlog

This backlog captures the next larger project directions after `v1.0.0`.
Items are ordered by a mix of implementation difficulty and dependency order.

## Recommended Order

1. GitHub Actions release automation
2. Detailed exit-code taxonomy
3. `--json` or other machine-readable output
4. Automatic rollback after failed verification
5. Claude Code skill and plugin as the primary interface
6. GUI
7. Windows/Linux support

## Dependency Chain

```text
CI/release automation
  -> typed errors / exit-code taxonomy
  -> machine-readable output
  -> automatic rollback
  -> Claude Code skill/plugin and GUI
  -> Windows/Linux support
```

## Items

### 1. GitHub Actions Release Automation

Status: Implemented.

Difficulty: Easy to medium.

Add GitHub Actions workflows for routine validation and release artifacts. Start
with `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test`, and locked release builds. Then add tag-triggered artifact and
checksum generation for the currently supported macOS release target.

This should happen first because it reduces risk for every later feature.
Windows/Linux support can later expand this into a platform matrix.

### 2. Detailed Exit-Code Taxonomy

Status: Implemented.

Difficulty: Easy to medium.

The CLI currently exits with `1` for every runtime error. Add stable exit codes
for major failure categories, such as invalid arguments, Codex process guard
failure, path validation failure, backup failure, copy or move failure,
metadata update failure, verification failure, and rollback failure.

This should happen before machine-readable output, the Claude Code
skill/plugin, and the GUI so callers can make reliable decisions without
parsing human text.

### 3. Machine-Readable Output

Status: Implemented.

Difficulty: Medium.

Add `--json` or a similar output mode for commands that currently print
human-oriented text. Prefer explicit serializable response structs over scraping
stdout. Include enough data for automation: command status, old and new paths,
matched metadata references, backup paths, changed reference counts, process
guard findings, Git worktree plan details, and verification results.

This should build on the exit-code taxonomy so success and failure behavior are
stable for scripts and future interfaces.

### 4. Automatic Rollback After Failed Verification

Status: Implemented.

Difficulty: Medium to hard.

When `apply --auto-rollback` updates metadata and the post-update verification
fails, rollback runs automatically using the backup manifest that was just created.
This should not trigger for failures before metadata changes, such as invalid
paths, failed copy verification, or failed Git worktree repair.

The implementation needs careful phase tracking so the tool can report whether
rollback was not needed, attempted and succeeded, or attempted and failed.
Machine-readable output should expose those states.

### 5. Claude Code Skill And Plugin As Primary Interface

Status: Implemented.

Difficulty: Medium.

Create an installable Claude Code skill and companion plugin that guide users
through project moves using the CLI. Codex should normally be fully closed
during a move, so Codex itself is not a reliable primary interface for this
workflow. The skill/plugin should explain when to use `plan`, `apply`,
`verify`, and `rollback`; warn users to close Codex before mutation; and use
the stable machine-readable CLI contract when automation is appropriate.

This should come after exit codes, JSON output, and rollback behavior are
settled so the skill/plugin does not encode a fragile or soon-to-change
workflow.

### 6. GUI

Difficulty: Hard.

Build a graphical interface for selecting the old project path, selecting the
new project path, previewing the plan, running apply, showing verification
results, and offering rollback when needed. The GUI should wrap stable library
or CLI behavior instead of duplicating move/update logic.

This depends on the machine-readable contract and exit-code taxonomy if it
drives the CLI as a subprocess. It also benefits from automatic rollback
because rollback state can be presented clearly to the user.

### 7. Windows/Linux Support

Difficulty: Hardest.

Extend the tool beyond the current Mac-first contract. This requires validating
Codex state locations, path formats, process detection, Trash/recycle-bin
behavior, Git worktree behavior, packaging, and release artifacts on each
supported platform.

Do this last because it touches the most assumptions. The earlier GitHub
Actions work can grow into the cross-platform test and release matrix needed
for this effort.
