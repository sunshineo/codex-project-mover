---
description: Move or relink a Codex Desktop project folder with codex-project-mover. Use when the user wants to move, rename, or relink a Codex project while preserving local Codex conversations.
---

# Move Codex Project

Use this skill from Claude Code, not from Codex. Codex should normally be fully
closed before mutation because `codex-project-mover apply`, `verify`, and
`rollback` edit local Codex state under `CODEX_HOME` and refuse to run when
Codex processes are detected.

## Invocation

- **Binary:** these commands assume `codex-project-mover` is on `PATH` (e.g.
  installed with `cargo install --path .`). If the project was only built with
  `cargo build --release`, the binary is not on `PATH` — invoke it as
  `./target/release/codex-project-mover` instead.
- **Capture both streams:** in `--json` mode the tool emits exactly one JSON
  object — to stdout on success, but to **stderr** on error. Always capture both
  streams when reading the result (the commands below append `2>&1`) so error
  fields (`error_kind`, `message`, and any `rollback` object) are never lost.
- **Custom Codex home:** if the user has a non-default Codex home, add
  `--codex-home "$CODEX_HOME"` to every `plan`, `apply`, and `verify` command
  (`rollback` takes no `--codex-home`).

## Workflow

1. Confirm the old absolute project path, new absolute project path, and whether
   the folder has already been moved manually.
2. Run a dry run:

   ```bash
   codex-project-mover --json plan --old "$OLD" --new "$NEW" 2>&1
   ```

3. Read the JSON result. Summarize `old_reference_count`,
   `references`, `git_worktree`, `fsmonitor`, and `codex_processes`.
4. Ensure Codex is fully closed before mutating. `plan` reports running
   processes but does not block; `apply` and `verify` refuse to run and exit
   with code `3` while any are detected. If `codex_processes.count` is greater
   than 0, tell the user to close Codex Desktop, Codex CLI sessions, and Codex
   app-server processes that could touch the same `CODEX_HOME`, and do not
   proceed to apply/verify until a fresh `plan` reports 0 (unless the user
   explicitly accepts the race risk via `--allow-running-codex`).
5. If the old folder still exists, run:

   ```bash
   codex-project-mover --json apply --old "$OLD" --new "$NEW" --auto-rollback 2>&1
   ```

   If the folder was already moved manually, run:

   ```bash
   codex-project-mover --json apply --old "$OLD" --new "$NEW" --relink-only --auto-rollback 2>&1
   ```

6. Run verification:

   ```bash
   codex-project-mover --json verify --old "$OLD" --new "$NEW" 2>&1
   ```

7. On success, preserve the backup path from the `backup_dir` / `backup_manifest`
   fields. If apply or verify reports an error, inspect `exit_code`,
   `error_kind`, and `message`. A `rollback` object is present only when
   `apply --auto-rollback` hits a post-update verification failure (exit `8`);
   other failures (e.g. exit `6` or `7`) may not include a backup path. If
   automatic rollback did not run or failed and you have a backup manifest,
   restore with:

   ```bash
   codex-project-mover --json rollback --backup "$BACKUP_MANIFEST" 2>&1
   ```

## Exit Codes

- `0`: success
- `1`: unexpected error
- `2`: invalid arguments from CLI parsing
- `3`: Codex process guard
- `4`: path validation
- `5`: backup failure
- `6`: copy, move, trash, or Git worktree operation failure
- `7`: metadata update failure
- `8`: verification failure
- `9`: rollback failure

## Rules

- Prefer `--json` so decisions use structured output instead of human text, and
  capture both stdout and stderr (errors print their JSON to stderr).
- Do not pass `--allow-running-codex` unless the user explicitly accepts the
  race risk after reviewing detected processes.
- Do not edit Codex metadata manually. Use `plan`, `apply`, `verify`, and
  `rollback`.
- Preserve the backup path in the final response.
