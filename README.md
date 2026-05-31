# Codex Project Mover

`codex-project-mover` is a Mac-first CLI for moving a Codex Desktop project folder and updating local Codex metadata so existing conversations continue to point at the new path.

## Commands

```bash
codex-project-mover plan --old /old/project --new /new/project
codex-project-mover apply --old /old/project --new /new/project
codex-project-mover apply --old /old/project --new /new/project --relink-only
codex-project-mover verify --old /old/project --new /new/project
codex-project-mover rollback --backup ~/.codex/codex-project-mover-backups/<id>/manifest.json
```

Close Codex before running commands. The tool exits if it sees Codex-related processes, and there is no force override.

Because Codex must be fully closed during the move, you can't drive this tool from inside Codex itself. Run it from a plain terminal, or have a different AI coding assistant (such as Claude Code) run it for you.

Normal `apply` backs up Codex metadata, copies the old folder to the new path, verifies the copy, updates supported metadata, verifies the old path is gone and new references are present, and moves the old folder to macOS Trash.

When the project folder is a Git worktree root, `apply` automatically repairs Git worktree metadata. Main worktrees are copied and then repaired with `git worktree repair` before Codex metadata changes. Linked worktrees are moved with `git worktree move` instead of a generic filesystem copy. Codex paths should always point at the checkout root, not at `.git/worktrees/...` internals.

Relink-only mode is for folders already moved by the user. It requires the old path to be missing and the new path to exist.

Relink-only also attempts Git worktree repair from the new path when the project has already been moved manually. If Git repair fails, the tool stops before updating Codex metadata and prints the Git command context.

Verify checks that supported old-path references are gone and supported new-path references are present.

Rollback restores Codex metadata from a backup manifest. If the tool created the new project folder during normal apply, rollback moves that new folder to Trash. It does not restore the old folder from Trash.

## Build

```bash
cargo build --release
./target/release/codex-project-mover --help
```

The first release target is Apple Silicon macOS. Intel macOS can be added with a second release artifact after the v1 workflow is stable.

## License

Licensed under the Apache License, Version 2.0.
