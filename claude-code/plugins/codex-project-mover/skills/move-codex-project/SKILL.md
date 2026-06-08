---
description: Move or relink a Codex Desktop project folder with codex-project-mover. Use when the user wants to move, rename, or relink a Codex project while preserving local Codex conversations.
---

# Move Codex Project

Use this skill from Claude Code, not from Codex. Codex should normally be fully
closed before mutation because `codex-project-mover apply`, `verify`, and
`rollback` edit local Codex state under `CODEX_HOME` and refuse to run when
Codex processes are detected.

## Workflow

1. Confirm the old absolute project path, new absolute project path, and whether
   the folder has already been moved manually.
2. Run a dry run:

   ```bash
   codex-project-mover --json plan --old "$OLD" --new "$NEW"
   ```

   If the user has a non-default Codex home, include
   `--codex-home "$CODEX_HOME"` on every `plan`, `apply`, and `verify`
   command in this workflow.

3. Read the JSON result. Summarize `old_reference_count`,
   `references`, `git_worktree`, `fsmonitor`, and `codex_processes`.
4. Before mutation, tell the user to close Codex Desktop, Codex CLI sessions,
   and Codex app-server processes that could touch the same `CODEX_HOME`.
5. If the old folder still exists, run:

   ```bash
   codex-project-mover --json apply --old "$OLD" --new "$NEW" --auto-rollback
   ```

   If the folder was already moved manually, run:

   ```bash
   codex-project-mover --json apply --old "$OLD" --new "$NEW" --relink-only --auto-rollback
   ```

6. Run verification:

   ```bash
   codex-project-mover --json verify --old "$OLD" --new "$NEW"
   ```

7. If apply or verify reports an error, inspect `exit_code`, `error_kind`,
   `message`, and any `rollback` object. If automatic rollback did not run or
   failed, use the reported backup manifest:

   ```bash
   codex-project-mover --json rollback --backup "$BACKUP_MANIFEST"
   ```

## Exit Codes

- `0`: success
- `2`: invalid arguments from CLI parsing
- `3`: Codex process guard
- `4`: path validation
- `5`: backup failure
- `6`: copy, move, trash, or Git worktree operation failure
- `7`: metadata update failure
- `8`: verification failure
- `9`: rollback failure

## Rules

- Prefer `--json` so decisions use structured output instead of human text.
- Do not pass `--allow-running-codex` unless the user explicitly accepts the
  race risk after reviewing detected processes.
- Do not edit Codex metadata manually. Use `plan`, `apply`, `verify`, and
  `rollback`.
- Preserve the backup path in the final response.
