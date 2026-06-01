# Codex Project Mover

`codex-project-mover` is a Mac-first CLI for moving a Codex Desktop project folder and updating local Codex metadata so existing conversations continue to point at the new path.

## Installation

### Homebrew

```bash
brew install sunshineo/tap/codex-project-mover
codex-project-mover --version
```

### Build From Source

With Rust 1.85 or newer installed:

```bash
cargo install --locked --git https://github.com/sunshineo/codex-project-mover --tag v1.0.0
codex-project-mover --version
```

From a local checkout:

```bash
git clone https://github.com/sunshineo/codex-project-mover.git
cd codex-project-mover
git checkout v1.0.0
cargo build --release --locked
./target/release/codex-project-mover --help
./target/release/codex-project-mover --version
```

### GitHub Release Binary

The first prebuilt release target is Apple Silicon macOS.

```bash
curl -LO https://github.com/sunshineo/codex-project-mover/releases/download/v1.0.0/codex-project-mover-aarch64-apple-darwin
curl -LO https://github.com/sunshineo/codex-project-mover/releases/download/v1.0.0/codex-project-mover-aarch64-apple-darwin.sha256
shasum -a 256 -c codex-project-mover-aarch64-apple-darwin.sha256
chmod +x codex-project-mover-aarch64-apple-darwin
xattr -d com.apple.quarantine ./codex-project-mover-aarch64-apple-darwin 2>/dev/null || true
./codex-project-mover-aarch64-apple-darwin --version
```

This v1 binary is unsigned and unnotarized. If macOS blocks it after download, the `xattr` command above removes the quarantine attribute:

```bash
xattr -d com.apple.quarantine ./codex-project-mover-aarch64-apple-darwin
```

To install the binary somewhere on your `PATH`:

```bash
mkdir -p "$HOME/.local/bin"
mv codex-project-mover-aarch64-apple-darwin "$HOME/.local/bin/codex-project-mover"
```

Make sure `$HOME/.local/bin` is in your shell `PATH` before running `codex-project-mover`.

## Commands

```bash
codex-project-mover plan --old /old/project --new /new/project
codex-project-mover apply --old /old/project --new /new/project
codex-project-mover apply --old /old/project --new /new/project --relink-only
codex-project-mover verify --old /old/project --new /new/project
codex-project-mover rollback --backup ~/.codex/codex-project-mover-backups/<id>/manifest.json
codex-project-mover rollback --backup ~/.codex/codex-project-mover-backups/<id>
```

Close Codex before running commands. By default, the tool exits if it sees a Codex process that can plausibly read or write the same local `CODEX_HOME` state that this tool edits: the main Codex Desktop process, `codex app-server`, or a standalone `codex` CLI/`codex exec` process. It does not stop or kill those processes.

The guard is intentionally focused. `codex exec` can persist rollout files and initialize state databases in-process, so a separate `codex app-server` process is not the only risky shape. Electron helpers, crashpad handlers, extension hosts, and `node_modules` dependency paths are ignored to avoid false positives from tools that merely contain `codex` in a path or command line.

If you know the detected process is unrelated to the project being moved, pass `--allow-running-codex` to proceed. This is an explicit user override; the tool still backs up metadata and verifies the move, but concurrent Codex writes can race with it.

Because Codex should normally be fully closed during the move, you usually can't drive this tool from inside Codex itself. Run it from a plain terminal, or have a different AI coding assistant (such as Claude Code) run it for you. Use `--allow-running-codex` only when you have checked the reported processes and accept the risk.

Normal `apply` backs up Codex metadata, copies the old folder to the new path, verifies the copy, updates supported metadata, verifies the old path is gone and new references are present, and moves the old folder to macOS Trash.

When the project folder is a Git worktree root, `apply` automatically repairs Git worktree metadata. Main worktrees are copied and then repaired with `git worktree repair` before Codex metadata changes. Linked worktrees are moved with `git worktree move` instead of a generic filesystem copy. Codex paths should always point at the checkout root, not at `.git/worktrees/...` internals.

Relink-only mode is for folders already moved by the user. It requires the old path to be missing and the new path to exist.

Relink-only also attempts Git worktree repair from the new path when the project has already been moved manually. If Git repair fails, the tool stops before updating Codex metadata and prints the Git command context.

Verify checks that supported old-path references are gone and supported new-path references are present.

Rollback restores Codex metadata from a backup manifest. `--backup` accepts either the backup directory printed by `apply` or the `manifest.json` file inside it. If the tool created the new project folder during normal apply, rollback moves that new folder to Trash. It does not restore the old folder from Trash.

## Build From Checkout

```bash
cargo build --release --locked
./target/release/codex-project-mover --help
./target/release/codex-project-mover --version
```

Intel macOS can be added with a second release artifact after the v1 workflow is stable.

## License

Licensed under the Apache License, Version 2.0.
