# Backlog Items 1-5 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement backlog items 1-5: GitHub Actions automation, stable exit codes, JSON output, automatic rollback after failed verification, and a Claude Code skill/plugin interface.

**Architecture:** Keep the Rust CLI as the authoritative implementation. Add a typed command output layer and typed application errors so human output, JSON output, and exit codes are derived from the same command results. Ship Claude Code skill/plugin files as wrappers around the stable CLI contract rather than duplicating move logic.

**Tech Stack:** Rust/Cargo, Clap, Serde JSON, GitHub Actions, Markdown, Claude Code skill/plugin metadata.

---

## Tasks

- [x] **Task 1: Record backlog and replace Codex skill wording**
  - Evidence: `docs/backlog.md` added; `SPEC.md` changed from Codex skill to Claude Code skill/plugin.
  - Verification: `rg -n "Codex skill|Installed Codex|Claude Code skill|Claude Code skill and plugin|Cloud Code skill|primary interface" docs/backlog.md SPEC.md README.md`; `cargo test`.

- [x] **Task 2: Add GitHub Actions validation and release automation**
  - Create `.github/workflows/ci.yml` for formatting, linting, tests, locked build, and Git worktree temp-dir test.
  - Create `.github/workflows/release.yml` for tag-triggered Apple Silicon macOS artifact and checksum upload.
  - Verification: `rg -n "cargo fmt --check|cargo clippy --all-targets -- -D warnings|cargo test|cargo build --release --locked|shasum -a 256|softprops/action-gh-release|TMPDIR=/tmp/codex-project-mover-release" .github/workflows` exited `0`; `git diff --check -- .github/workflows docs/superpowers/plans/2026-06-08-backlog-items-1-5.md` exited `0`.

- [x] **Task 3: Add typed error categories and stable exit codes**
  - Add `src/app_error.rs` with `ExitCode` and `AppError`.
  - Convert process guard and command validation/verification failures to typed errors.
  - Update `main.rs` to exit with the category-specific code.
  - Verification: `cargo test --test cli` exited `0`; tests assert exit code `3` for process guard, `4` for path validation, and `8` for verification failures.

- [x] **Task 4: Add machine-readable JSON output**
  - Add global `--json` output flag.
  - Add serializable output structs for `plan`, `apply`, `verify`, and `rollback`.
  - Preserve human output by rendering the same command results in text mode.
  - Verification: `cargo test --test cli` exited `0`; tests parse JSON for `plan`, `verify`, `apply --relink-only`, and runtime failure output.

- [x] **Task 5: Add automatic rollback after failed metadata verification**
  - Add an opt-in `apply --auto-rollback` flag.
  - Track apply phases so rollback only runs after metadata changes and backup creation.
  - Report rollback status in human and JSON output.
  - Verification: `cargo test --test cli apply_auto_rollback_restores_metadata_after_verification_failure` exited `0`; full `cargo test --test cli` also exited `0`.

- [x] **Task 6: Add Claude Code skill and plugin interface**
  - Add installable skill and plugin files under `claude-code/`.
  - Document safe move workflow, Codex-closed requirement, JSON usage, and rollback handling.
  - Verification: `python3 -m json.tool claude-code/plugins/codex-project-mover/.claude-plugin/plugin.json >/dev/null` exited `0`; `claude plugin validate ./claude-code/plugins/codex-project-mover` exited `0`; `rg -n "plan|apply|verify|rollback|--json|close Codex|closed|--auto-rollback|--plugin-dir" claude-code/plugins/codex-project-mover README.md` exited `0`.

- [x] **Task 7: Final verification**
  - Run `cargo fmt --check`.
  - Run `cargo clippy --all-targets -- -D warnings`.
  - Run `cargo test`.
  - Run `TMPDIR=/tmp/codex-project-mover-release cargo test --test git_worktree`.
  - Update this plan with execution evidence.
  - Evidence: `cargo fmt --check` exited `0`; `cargo clippy --all-targets -- -D warnings` exited `0`; `cargo test` exited `0`; `mkdir -p /tmp/codex-project-mover-release && TMPDIR=/tmp/codex-project-mover-release cargo test --test git_worktree` exited `0`; `cargo build --release --locked && ./target/release/codex-project-mover --version` exited `0` and printed `codex-project-mover 1.0.0` before the later `1.1.0` release bump; `git diff --check -- .github src tests README.md RELEASING.md SPEC.md docs claude-code` exited `0`.
