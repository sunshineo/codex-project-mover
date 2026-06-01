# Codex Project Mover — Agent Instructions

This tool moves a Codex Desktop project folder and updates local Codex metadata so existing conversations continue to work under the new path.

## Prerequisites

Codex should normally be fully closed before running any command. The tool checks for processes that can plausibly read or write the same local `CODEX_HOME` state being edited: the main Codex Desktop process, `codex app-server`, and standalone `codex` CLI/`codex exec` processes. It reports those processes and exits without trying to stop them.

If the user has reviewed the reported process list and knows the process is unrelated to the project being moved, they can pass `--allow-running-codex` to proceed. Do not add this flag automatically; it is a user risk decision because concurrent Codex writes can race with the metadata update.

## Build

```bash
cargo build --release
./target/release/codex-project-mover --help
./target/release/codex-project-mover --version
```

## Recommended workflow

Run these steps in order:

### 1. Plan (dry run)

```bash
codex-project-mover plan --old /old/project --new /new/project
```

Scans all supported Codex metadata surfaces and prints every reference that would be updated. Makes no changes. Review the output to confirm the right files are affected before proceeding.

### 2. Apply

```bash
codex-project-mover apply --old /old/project --new /new/project
```

- Backs up all affected metadata files to `~/.codex/codex-project-mover-backups/<id>/`
- Copies the project folder from old to new
- Verifies the copy
- Updates all supported metadata
- Verifies old-path references are gone and new-path references are present
- Moves the old folder to macOS Trash

On success, prints the backup directory path and a summary of updated references.

### 3. Verify (optional post-check)

```bash
codex-project-mover verify --old /old/project --new /new/project
```

Re-scans all surfaces. Fails if any old-path references remain or no new-path references are found. Safe to run at any time after apply.

## Relink-only mode

Use this when the project folder has already been moved manually and only metadata needs updating:

```bash
codex-project-mover apply --old /old/project --new /new/project --relink-only
```

Requires: old path does not exist, new path exists as a directory. Does not move or copy any files.

## Rollback

```bash
codex-project-mover rollback --backup ~/.codex/codex-project-mover-backups/<id>/manifest.json
codex-project-mover rollback --backup ~/.codex/codex-project-mover-backups/<id>
```

`--backup` accepts either the backup directory printed by `apply` or the `manifest.json` file inside it. Rollback restores all backed-up metadata files from the manifest. If the tool created the new project folder during a normal apply, it moves that folder to Trash.

Rollback does not restore the old project folder from Trash — that must be done manually if needed.

## Supported metadata surfaces

- `~/.codex/sessions/**/*.jsonl` and `~/.codex/archived_sessions/**/*.jsonl` — structured `cwd` fields only
- `~/.codex/state_*.sqlite` — `threads.cwd` column, exact matches only
- `~/.codex/.codex-global-state.json` — exact string values and exact object keys equal to the old path
- `~/.codex/config.toml` — exact string values, `[projects."/path"]` table keys, and per-path open-target preference keys
- `~/.codex/sqlite/codex-dev.db` — automation `cwds` columns, when present

The tool never edits arbitrary message text in session history.

## What to do if apply fails mid-way

Apply prints the backup path before making any changes. If verification fails after metadata is written, the output includes the rollback command to run. Run it, confirm Codex metadata is restored, then investigate before retrying.
