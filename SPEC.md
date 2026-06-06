# Codex Project Mover Spec

Date: 2026-05-28

## Context

Codex Desktop projects are tied to local workspace paths. Moving or renaming a project folder can leave existing conversations grouped under the old path, fail with missing working directory errors, or require starting a new project in the app.

This project should build a small Mac-first utility that safely moves a Codex project folder and updates local Codex metadata so existing conversations continue to work under the new path.

## Research Summary

Local Codex state on this Mac stores absolute workspace paths in several places:

- `~/.codex/state_*.sqlite`, especially the `threads.cwd` column.
- `~/.codex/sessions/**/*.jsonl` and `~/.codex/archived_sessions/**/*.jsonl`, especially structured `"cwd"` fields.
- `~/.codex/.codex-global-state.json`, including saved workspace roots, project ordering, labels, collapsed sidebar groups, prompt history, and some per-thread permission state.
- `~/.codex/config.toml`, including `[projects."/path"]` trust entries and desktop per-path open target preferences.
- `~/.codex/sqlite/codex-dev.db` may contain automations with `cwds`, though none were present on this Mac during research.

Existing community work:

- `Adam-Bull/Codex-thread-toolkit` has a `thread-workspace-relink` script. It updates JSONL `cwd` fields and SQLite `threads.cwd`, optionally does a raw replace in `.codex-global-state.json`, and creates backups. It does not move folders and does not update `config.toml`.
- A smaller "Codex Thread Mover" gist updates one thread in SQLite and JSONL.
- No official OpenAI "move project" command or UI was found.

## Product Goal

Create a reliable command-line tool for Mac users:

```bash
codex-project-mover plan --old /old/project --new /new/project
codex-project-mover apply --old /old/project --new /new/project
codex-project-mover verify --old /old/project --new /new/project
codex-project-mover rollback --backup ~/.codex/codex-project-mover-backups/<id>
```

The tool should:

- Refuse to run `apply` while relevant Codex local-state processes are open, unless an explicit user override flag is supplied.
- Create a complete timestamped backup before modifying anything.
- Move the project folder itself when requested.
- Update Codex metadata for the old path to the new path.
- Verify that old-path references are gone from the supported state surfaces.
- Produce clear output that explains what was changed and where backups live.

## Language Decision

Use Rust.

Reasoning:

- This should be generally usable by other Codex Desktop users on Mac.
- A compiled binary avoids Python version, virtualenv, `pip`, and dependency problems.
- The tool edits sensitive local state under `~/.codex`, so distribution reliability matters.
- Rust gives strong typing, good CLI ergonomics, good SQLite/TOML/JSON libraries, and single-binary GitHub Releases distribution.

Python would be faster to prototype, but shifts too much environment setup risk to users. Shell should not be the main implementation because this task touches structured SQLite, JSONL, TOML, filesystem moves, backups, and rollback.

## Proposed Architecture

Use a Rust CLI with subcommands:

- `plan`: inspect state, validate paths, show every proposed change, no writes.
- `apply`: run the migration after safety checks and backup.
- `verify`: re-scan state surfaces and report old/new path counts.
- `rollback`: restore files from a backup manifest.

Likely crates:

- `clap` for CLI parsing.
- `rusqlite` for SQLite.
- `serde` and `serde_json` for JSON and JSONL.
- `toml_edit` for preserving and updating `config.toml`.
- `walkdir` for session file discovery.
- `sha2` or similar for optional pre/post file checksums.
- `tempfile` for tests.

## Migration Workflow

`apply` should be structured as a transactional workflow as much as practical:

1. Validate `old` exists and `new` does not exist unless using a relink-only mode.
2. Detect whether Codex Desktop is running and refuse by default.
3. Discover `CODEX_HOME`, defaulting to `~/.codex`.
4. Discover state files and databases.
5. Build a migration plan with exact files, database rows, and TOML keys to modify.
6. Write backup copies and a manifest before changing anything.
7. Move the project folder.
8. Update JSONL structured `cwd` fields.
9. Update SQLite `threads.cwd`.
10. Update `.codex-global-state.json`.
11. Update `config.toml`.
12. Optionally update automation `cwds` if present.
13. Verify supported old-path references are zero.
14. Print the backup path and next steps.

If any verification step fails, the tool should report the failure clearly and point to rollback. Automatic rollback can be considered, but manual rollback with a reliable manifest is acceptable for v1.

## Important Safety Rules

- Never do broad unstructured search/replace in session history except for explicitly supported state files.
- JSONL edits should target structured `cwd` fields, not arbitrary message text.
- SQLite updates should target known columns such as `threads.cwd`, not arbitrary text columns.
- `config.toml` should be edited with a TOML-aware library so comments/order are preserved as much as possible.
- `.codex-global-state.json` can be edited as JSON, but path replacement should be limited to string values equal to or containing the exact old path, with plan output showing affected keys.
- Backups must include every file changed and the original folder location metadata.
- Cross-volume moves should copy, verify, then delete the old folder only after verification succeeds.

## Initial Scope

Mac-first v1:

- Support Codex Desktop local state under `~/.codex`.
- Support Apple Silicon release binary first; Intel Mac can be added if easy.
- Support moving one project path at a time.
- Support exact absolute paths.
- Support dry-run/plan, apply, verify, rollback.
- Support relink-only mode for cases where the user already moved the folder.
- Support `--version`.
- Support rollback from either a backup directory or its `manifest.json`.

Out of scope for v1:

- Windows/Linux support.
- A GUI.
- An installed Codex skill as the primary interface.
- Cloud/project sync behavior beyond local Codex Desktop state.
- Editing arbitrary old-path text inside user messages.

Post-v1 CLI considerations:

- Machine-readable output, such as `--json`, can be added later if scripts need stable structured output. v1 output is human-oriented; scripts should rely on exit status.
- Global options such as top-level `--codex-home` can be considered later. v1 keeps options on each subcommand.
- Detailed exit-code taxonomy can be considered later. v1 uses success vs failure.

V1 freeze notes:

- Subcommand descriptions in top-level `--help` are cosmetic and do not block v1. The stable contract is the command names, flags, required arguments, and behavior.
- `rollback` intentionally does not accept `--codex-home`. It restores the absolute metadata paths recorded in the backup manifest, so an independent Codex home argument would be ignored or dangerous.
- Release mechanics are tracked separately from the CLI contract. See `RELEASING.md` for the v1 tag and artifact checklist.

## Open Questions

- Should the default `apply` move the folder, or should moving require an explicit `--move-folder` flag?
- Should the tool require the new path parent to exist, or create it automatically?
- How strict should Codex process detection be? `Codex.app`, `codex app-server`, standalone `codex` CLI processes, and related helper processes may all matter.
- Should rollback move the project folder back, or only restore Codex metadata by default?
- Should the tool use a single backup directory under `~/.codex/codex-project-mover-backups` or a backup directory adjacent to the repo?

## Resolved Decisions

- Normal `apply` moves the project folder by default.
- Normal `apply` requires the old folder to exist and the new path to not exist.
- Relink-only mode requires the old folder to not exist and the new folder to exist.
- New parent directories are created automatically.
- `apply`, `verify`, and `rollback` exit when Codex-related processes that can plausibly touch `CODEX_HOME` state are running: the main Codex Desktop process, `codex app-server`, and standalone `codex` CLI/`codex exec` processes. `plan` (dry run) instead reports any such processes in its output and continues, so a dry run previews this check without blocking. The guard does not stop or kill processes.
- Users can pass `--allow-running-codex` to bypass the process guard after they decide the detected process is unrelated to the move. This flag is deliberately explicit because concurrent Codex writes can race with metadata updates.
- Process detection ignores known non-writing false positives such as Electron helper processes, crashpad handlers, extension hosts, `node_modules` dependency paths, and the mover process itself. The reason is that many local tools include `codex` in paths or command lines without sharing the `~/.codex` state being rewritten.
- `codex exec` is treated as relevant even without a separate OS-level `codex app-server` process because it can load `CODEX_HOME`, initialize state DBs, and persist rollout files in-process.
- The old folder is moved to macOS Trash after copy and metadata verification.
- Backups are metadata backups under `~/.codex/codex-project-mover-backups/<id>` and include movement metadata for rollback cleanup.
- Rollback restores metadata from backup and moves the tool-created new folder to Trash when applicable. `rollback --backup` accepts either the backup directory printed by `apply` or the manifest file inside it.
- `.codex-global-state.json` updates JSON string values and JSON object keys exactly equal to the old path.
- `config.toml` updates exact string values, exact `[projects."/path"]` table keys, and exact per-path open-target preference keys.
- `verify` requires supported old-path references to be gone and supported new-path references to be present.

## Suggested Next Step

Start the new Codex thread in this project and ask it to turn this spec into an implementation plan. The first implementation milestone should be a read-only scanner that reports all old-path references across supported Codex state surfaces, with tests using fixture files and temporary SQLite databases.
