# Git Worktree Auto-Repair Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add automatic Git worktree detection, repair, movement, and verification to `codex-project-mover`.

**Architecture:** Keep Git behavior isolated in a new `src/git_worktree.rs` module. Command modules call that module at explicit workflow points: `plan` reports Git state, `apply` repairs or moves before Codex metadata updates, `verify` validates Git state after Codex metadata checks.

**Tech Stack:** Rust 1.78, `std::process::Command`, existing `anyhow`, `walkdir`, `assert_cmd`, `predicates`, `tempfile`, and the system `git` binary for integration tests.

---

## File Structure

- Create `src/git_worktree.rs`: Git `.git` entry inspection, `git worktree list --porcelain -z` parsing, repair planning, repair execution, linked worktree move execution, and Git verification.
- Modify `src/lib.rs`: export the new module.
- Modify `src/commands/plan.rs`: print Git worktree planning information after Codex metadata matches.
- Modify `src/commands/apply.rs`: branch normal `apply` into non-Git, main-worktree, and linked-worktree flows; repair Git before metadata updates.
- Modify `src/commands/verify.rs`: keep Codex verification and add Git verification when Git state is detected.
- Create `tests/git_worktree.rs`: focused parser, path mapping, and Git detection tests.
- Create `tests/git_worktree_cli.rs`: end-to-end CLI tests with temporary Git repositories.
- Modify `README.md`: document automatic Git worktree repair behavior.
- Update this plan after each task starts and completes. Record evidence under "Execution Log".

## Execution Log

- 2026-05-29: Started Task 1.
- 2026-05-29: Step 2 red test evidence: `cargo test --test git_worktree` failed as expected with unresolved import `codex_project_mover::git_worktree`.
- 2026-05-29: Step 4 green test evidence: `cargo test --test git_worktree` passed, 2 tests passed, 0 failed.
- 2026-05-29: Task 1 changed `src/git_worktree.rs`, `src/lib.rs`, `tests/git_worktree.rs`, and this plan file.
- 2026-05-29: Task 1 commit evidence: `b03c85c7067fe71a4c92b2ffcc81744578742a3d`.
- 2026-05-29: Task 1 review fix evidence: added comparison-only worktree path normalization and stricter `locked`/`prunable` parser matching; `cargo test --test git_worktree` passed, 6 tests passed, 0 failed.
- 2026-05-29: Started Task 2 Step 1.
- 2026-05-29: Completed Task 2 Step 1; added Git detection helpers and main/linked worktree detection tests in `tests/git_worktree.rs`.
- 2026-05-29: Task 2 Step 2 red test evidence: `cargo test --test git_worktree` failed as expected with unresolved import `codex_project_mover::git_worktree::build_plan_for_existing_project`.
- 2026-05-29: Completed Task 2 Step 3; added `.git` inspection, `git worktree list --porcelain -z` command runner, and plan construction in `src/git_worktree.rs`.
- 2026-05-29: Task 2 Step 4 evidence: `cargo test --test git_worktree` passed, 8 tests passed, 0 failed. Detection tests canonicalize temp repo paths before comparing because Git reports macOS temp worktrees under `/private/var` while `tempfile` returns `/var`.
- 2026-05-29: Task 2 Step 5 commit evidence: `9b5d9598805c0e9cfffc401f273ddce21ed02f0a`; pre-commit verification `rustfmt --check src/git_worktree.rs tests/git_worktree.rs`, `git diff --check -- src/git_worktree.rs tests/git_worktree.rs docs/superpowers/plans/2026-05-29-git-worktree-auto-repair.md`, and `cargo test --test git_worktree` passed.
- 2026-05-29: Started Task 2 review follow-up to harden linked worktree detection, invalid `.git` handling, Git command diagnostics, and Git-backed tests.
- 2026-05-29: Task 2 review follow-up red test evidence: `cargo test --test git_worktree` failed as expected for invalid `.git` returning `Ok(NotGit)` and `--separate-git-dir` under a `worktrees` path being misclassified as `LinkedWorktree`; 9 passed, 2 failed.
- 2026-05-29: Task 2 review follow-up green evidence: hardened linked detection to require private gitdir `commondir`, invalid `.git` files now error with path context, Git command failures include command/cwd/status/stdout/stderr, Git-backed test helper disables global/system config, signing, hooks, and init templates; `cargo test --test git_worktree` passed, 11 tests passed, 0 failed.
- 2026-05-29: Task 2 review follow-up commit evidence: `0c4001efc1e730114357297cfb0e50310cbc5528`.
- 2026-05-29: Started Task 3 Step 1.
- 2026-05-29: Completed Task 3 Step 1; added Git operation tests in `tests/git_worktree.rs`.
- 2026-05-29: Started Task 3 Step 2.
- 2026-05-29: Task 3 Step 2 red test evidence: `cargo test --test git_worktree` failed as expected with unresolved imports for `move_linked_worktree`, `repair_main_worktree_after_copy`, and `verify_git_worktree_state`.
- 2026-05-29: Started Task 3 Step 3.
- 2026-05-29: Completed Task 3 Step 3; implemented Git repair, linked worktree move, verification, new-path verification, and dynamic Git argument execution in `src/git_worktree.rs`.
- 2026-05-29: Started Task 3 Step 4.
- 2026-05-29: Task 3 Step 4 evidence: `cargo test --test git_worktree` passed, 15 tests passed, 0 failed.
- 2026-05-29: Started Task 3 Step 5.
- 2026-05-29: Task 3 Step 5 commit evidence: `def99eb8b4c15eb8a04b47d034de4dc751bde0dd`; pre-commit verification `cargo fmt --check`, `git diff --check -- src/git_worktree.rs tests/git_worktree.rs docs/superpowers/plans/2026-05-29-git-worktree-auto-repair.md`, and `cargo test --test git_worktree` passed.
- 2026-05-29: Started Task 3 code-quality follow-up for bare-backed linked moves, post-operation verification errors, verification contract tests, main repair integration coverage, and ambient Git environment hardening.
- 2026-05-29: Task 3 follow-up red test evidence: `cargo test --test git_worktree` failed as expected; `verify_git_from_new_path_rejects_non_git_new_path` returned `Ok(())` for NotGit and `moves_linked_worktree_from_bare_backed_repository` ran `git worktree move` from the temp parent instead of the bare repo.
- 2026-05-29: Task 3 follow-up green evidence: `cargo test --test git_worktree` passed, 19 tests passed, 0 failed. Scoped checks `rustfmt --check src/git_worktree.rs tests/git_worktree.rs` and `git diff --check -- src/git_worktree.rs tests/git_worktree.rs docs/superpowers/plans/2026-05-29-git-worktree-auto-repair.md` passed.
- 2026-05-29: Task 3 follow-up commit evidence: `cebbefbed20dafc273a8c56a3b06e2ed3fecef0b`.
- 2026-05-29: Started Task 4 Step 1.
- 2026-05-29: Completed Task 4 Step 1; added CLI plan test in `tests/cli.rs`.
- 2026-05-29: Started Task 4 Step 2.
- 2026-05-29: Task 4 Step 2 red test evidence: `cargo test --test cli plan_reports_no_git_worktree_repair_for_plain_folder` failed as expected because stdout did not contain `Git worktree: none detected`.
- 2026-05-29: Started Task 4 Step 3.
- 2026-05-29: Completed Task 4 Step 3; `plan` now prints Git worktree planning details from `build_plan_for_existing_project`.
- 2026-05-29: Started Task 4 Step 4.
- 2026-05-29: Task 4 Step 4 evidence: `cargo test --test cli plan_reports_no_git_worktree_repair_for_plain_folder` passed, 1 test passed, 0 failed.
- 2026-05-29: Started Task 4 Step 5.
- 2026-05-29: Task 4 Step 5 commit evidence: `016f1485b2fb9da12e21f00c9eed6950dfe3c91d`; pre-commit verification `cargo test --test cli plan_reports_no_git_worktree_repair_for_plain_folder`, `rustfmt --check src/commands/plan.rs tests/cli.rs`, and `git diff --check -- src/commands/plan.rs tests/cli.rs docs/superpowers/plans/2026-05-29-git-worktree-auto-repair.md` passed.
- 2026-05-29: Started Task 4 quality fix for executable Git worktree plan commands.
- 2026-05-29: Task 4 quality fix red test evidence: `cargo test --test cli plan_reports_` failed as expected; `plan_reports_main_worktree_repair_paths` showed the main-worktree repair command omitted the mapped linked repair path, and `plan_reports_linked_worktree_move_cwd` showed the linked move command omitted `git -C <cwd>`.
- 2026-05-29: Task 4 quality fix green evidence: `cargo test --test cli plan_reports_` passed, 4 tests passed, 0 failed.
- 2026-05-29: Task 4 quality fix changed `src/commands/plan.rs`, `src/git_worktree.rs`, `tests/cli.rs`, and this plan file.
- 2026-05-29: Task 4 quality fix pre-commit verification evidence: `cargo test --test cli` passed, 12 tests passed, 0 failed; `cargo test --test git_worktree` passed, 19 tests passed, 0 failed; `rustfmt --check src/commands/plan.rs src/git_worktree.rs tests/cli.rs` passed; `git diff --check -- src/commands/plan.rs src/git_worktree.rs tests/cli.rs docs/superpowers/plans/2026-05-29-git-worktree-auto-repair.md` passed.

---

### Task 1: Add Worktree Entry Parsing And Path Mapping

**Files:**
- Create: `src/git_worktree.rs`
- Modify: `src/lib.rs`
- Create: `tests/git_worktree.rs`

- [x] **Step 1: Write failing parser and mapper tests**

Create `tests/git_worktree.rs` with this initial content:

```rust
use std::path::Path;

use codex_project_mover::git_worktree::{
    map_worktree_paths, parse_worktree_list, WorktreePathMove,
};

#[test]
fn parses_porcelain_z_worktree_entries() {
    let input = b"worktree /repo/main\0HEAD 1111111111111111111111111111111111111111\0branch refs/heads/main\0\0worktree /repo/linked\0HEAD 2222222222222222222222222222222222222222\0detached\0locked manual lock\0prunable gitdir file points to non-existent location\0\0";

    let entries = parse_worktree_list(input).unwrap();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].path, Path::new("/repo/main"));
    assert_eq!(entries[0].head.as_deref(), Some("1111111111111111111111111111111111111111"));
    assert_eq!(entries[0].branch.as_deref(), Some("refs/heads/main"));
    assert!(!entries[0].detached);
    assert_eq!(entries[1].path, Path::new("/repo/linked"));
    assert_eq!(entries[1].head.as_deref(), Some("2222222222222222222222222222222222222222"));
    assert!(entries[1].detached);
    assert_eq!(entries[1].locked.as_deref(), Some("manual lock"));
    assert_eq!(
        entries[1].prunable.as_deref(),
        Some("gitdir file points to non-existent location")
    );
}

#[test]
fn maps_worktree_paths_under_old_root_to_new_root() {
    let entries = parse_worktree_list(
        b"worktree /old/project\0HEAD a\0branch refs/heads/main\0\0worktree /old/project/.worktrees/feature\0HEAD b\0branch refs/heads/feature\0\0worktree /outside/linked\0HEAD c\0branch refs/heads/outside\0\0",
    )
    .unwrap();

    let moves = map_worktree_paths(
        &entries,
        Path::new("/old/project"),
        Path::new("/new/project"),
    );

    assert_eq!(
        moves,
        vec![
            WorktreePathMove {
                old_path: Path::new("/old/project").to_path_buf(),
                new_path: Path::new("/new/project").to_path_buf(),
            },
            WorktreePathMove {
                old_path: Path::new("/old/project/.worktrees/feature").to_path_buf(),
                new_path: Path::new("/new/project/.worktrees/feature").to_path_buf(),
            },
        ]
    );
}
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test --test git_worktree
```

Expected: FAIL because `codex_project_mover::git_worktree` does not exist.

- [x] **Step 3: Add the module and minimal parser implementation**

Create `src/git_worktree.rs` with:

```rust
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitWorktreeKind {
    NotGit,
    MainWorktree,
    LinkedWorktree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeEntry {
    pub path: PathBuf,
    pub head: Option<String>,
    pub branch: Option<String>,
    pub detached: bool,
    pub bare: bool,
    pub locked: Option<String>,
    pub prunable: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreePathMove {
    pub old_path: PathBuf,
    pub new_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitWorktreePlan {
    pub kind: GitWorktreeKind,
    pub project_path: PathBuf,
    pub new_project_path: PathBuf,
    pub entries: Vec<WorktreeEntry>,
    pub path_moves: Vec<WorktreePathMove>,
}

impl GitWorktreePlan {
    pub fn no_git(old: &Path, new: &Path) -> Self {
        Self {
            kind: GitWorktreeKind::NotGit,
            project_path: old.to_path_buf(),
            new_project_path: new.to_path_buf(),
            entries: Vec::new(),
            path_moves: Vec::new(),
        }
    }

    pub fn repair_paths(&self) -> Vec<PathBuf> {
        self.path_moves
            .iter()
            .filter(|path_move| path_move.old_path != self.project_path)
            .map(|path_move| path_move.new_path.clone())
            .collect()
    }
}

pub fn parse_worktree_list(bytes: &[u8]) -> Result<Vec<WorktreeEntry>> {
    let mut entries = Vec::new();
    let mut current: Option<WorktreeEntry> = None;

    for raw_field in bytes.split(|byte| *byte == b'\0') {
        if raw_field.is_empty() {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            continue;
        }

        let field = std::str::from_utf8(raw_field)?;
        if let Some(path) = field.strip_prefix("worktree ") {
            if let Some(entry) = current.replace(WorktreeEntry {
                path: PathBuf::from(path),
                head: None,
                branch: None,
                detached: false,
                bare: false,
                locked: None,
                prunable: None,
            }) {
                entries.push(entry);
            }
        } else if let Some(entry) = current.as_mut() {
            if let Some(head) = field.strip_prefix("HEAD ") {
                entry.head = Some(head.to_string());
            } else if let Some(branch) = field.strip_prefix("branch ") {
                entry.branch = Some(branch.to_string());
            } else if field == "detached" {
                entry.detached = true;
            } else if field == "bare" {
                entry.bare = true;
            } else if let Some(reason) = field.strip_prefix("locked") {
                entry.locked = Some(reason.trim_start().to_string());
            } else if let Some(reason) = field.strip_prefix("prunable") {
                entry.prunable = Some(reason.trim_start().to_string());
            }
        } else {
            bail!("git worktree porcelain output field appeared before worktree path: {field}");
        }
    }

    if let Some(entry) = current {
        entries.push(entry);
    }

    Ok(entries)
}

pub fn map_worktree_paths(entries: &[WorktreeEntry], old: &Path, new: &Path) -> Vec<WorktreePathMove> {
    entries
        .iter()
        .filter_map(|entry| {
            entry.path.strip_prefix(old).ok().map(|relative| WorktreePathMove {
                old_path: entry.path.clone(),
                new_path: new.join(relative),
            })
        })
        .collect()
}
```

Modify `src/lib.rs` by adding:

```rust
pub mod git_worktree;
```

- [x] **Step 4: Run tests to verify they pass**

Run:

```bash
cargo test --test git_worktree
```

Expected: PASS.

- [x] **Step 5: Commit Task 1**

Run:

```bash
git add src/lib.rs src/git_worktree.rs tests/git_worktree.rs docs/superpowers/plans/2026-05-29-git-worktree-auto-repair.md
git commit -m "feat: parse git worktree metadata"
```

Record the commit hash and test result in the Execution Log.

---

### Task 2: Detect Main And Linked Worktree Roots

**Files:**
- Modify: `src/git_worktree.rs`
- Modify: `tests/git_worktree.rs`

- [x] **Step 1: Add Git test helpers and failing detection tests**

Append this helper code to `tests/git_worktree.rs`:

```rust
use std::fs;
use std::process::Command;

use tempfile::tempdir;

use codex_project_mover::git_worktree::{build_plan_for_existing_project, GitWorktreeKind};

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn git(cwd: &std::path::Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn init_repo(path: &std::path::Path) {
    fs::create_dir_all(path).unwrap();
    git(path, &["init"]);
    git(path, &["config", "user.email", "test@example.com"]);
    git(path, &["config", "user.name", "Test User"]);
    fs::write(path.join("README.md"), "hello\n").unwrap();
    git(path, &["add", "README.md"]);
    git(path, &["commit", "-m", "initial"]);
}

#[test]
fn detects_main_worktree_root() {
    if !git_available() {
        eprintln!("skipping detects_main_worktree_root because git is unavailable");
        return;
    }

    let temp = tempdir().unwrap();
    let old = temp.path().join("repo");
    let new = temp.path().join("moved/repo");
    init_repo(&old);

    let plan = build_plan_for_existing_project(&old, &new).unwrap();

    assert_eq!(plan.kind, GitWorktreeKind::MainWorktree);
    assert_eq!(plan.project_path, old);
    assert_eq!(plan.new_project_path, new);
    assert_eq!(plan.entries.len(), 1);
}

#[test]
fn detects_linked_worktree_root() {
    if !git_available() {
        eprintln!("skipping detects_linked_worktree_root because git is unavailable");
        return;
    }

    let temp = tempdir().unwrap();
    let main = temp.path().join("repo");
    let linked = temp.path().join("linked");
    let new = temp.path().join("moved-linked");
    init_repo(&main);
    git(&main, &["worktree", "add", linked.to_str().unwrap(), "-b", "feature"]);

    let plan = build_plan_for_existing_project(&linked, &new).unwrap();

    assert_eq!(plan.kind, GitWorktreeKind::LinkedWorktree);
    assert_eq!(plan.project_path, linked);
    assert!(plan.entries.iter().any(|entry| entry.path == main));
    assert!(plan.entries.iter().any(|entry| entry.path == plan.project_path));
}
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test --test git_worktree
```

Expected: FAIL because `build_plan_for_existing_project` is not implemented.

- [x] **Step 3: Implement Git command runner and detection**

Append this implementation to `src/git_worktree.rs`, then adjust imports to include `std::fs` and `std::process::Command`:

```rust
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
enum GitDotfile {
    Missing,
    Directory(PathBuf),
    File { gitdir: PathBuf },
}

pub fn build_plan_for_existing_project(old: &Path, new: &Path) -> Result<GitWorktreePlan> {
    match inspect_dotgit(old)? {
        GitDotfile::Missing => Ok(GitWorktreePlan::no_git(old, new)),
        GitDotfile::Directory(_) => {
            let entries = git_worktree_entries(old)?;
            Ok(GitWorktreePlan {
                kind: GitWorktreeKind::MainWorktree,
                project_path: old.to_path_buf(),
                new_project_path: new.to_path_buf(),
                path_moves: map_worktree_paths(&entries, old, new),
                entries,
            })
        }
        GitDotfile::File { gitdir } => {
            let entries = git_worktree_entries(old)?;
            let kind = if gitdir.components().any(|component| component.as_os_str() == "worktrees") {
                GitWorktreeKind::LinkedWorktree
            } else {
                GitWorktreeKind::MainWorktree
            };
            Ok(GitWorktreePlan {
                kind,
                project_path: old.to_path_buf(),
                new_project_path: new.to_path_buf(),
                path_moves: map_worktree_paths(&entries, old, new),
                entries,
            })
        }
    }
}

fn inspect_dotgit(project: &Path) -> Result<GitDotfile> {
    let dotgit = project.join(".git");
    if !dotgit.exists() {
        return Ok(GitDotfile::Missing);
    }
    if dotgit.is_dir() {
        return Ok(GitDotfile::Directory(dotgit));
    }

    let contents = fs::read_to_string(&dotgit)?;
    let Some(raw_gitdir) = contents.trim().strip_prefix("gitdir:") else {
        return Ok(GitDotfile::Missing);
    };
    let raw_gitdir = raw_gitdir.trim();
    let gitdir = if Path::new(raw_gitdir).is_absolute() {
        PathBuf::from(raw_gitdir)
    } else {
        project.join(raw_gitdir)
    };

    Ok(GitDotfile::File { gitdir })
}

fn git_worktree_entries(cwd: &Path) -> Result<Vec<WorktreeEntry>> {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(["worktree", "list", "--porcelain", "-z"])
        .output()?;

    if !output.status.success() {
        bail!(
            "git worktree list failed in {}\nstdout:\n{}\nstderr:\n{}",
            cwd.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    parse_worktree_list(&output.stdout)
}
```

- [x] **Step 4: Run detection tests**

Run:

```bash
cargo test --test git_worktree
```

Expected: PASS.

- [x] **Step 5: Commit Task 2**

Run:

```bash
git add src/git_worktree.rs tests/git_worktree.rs docs/superpowers/plans/2026-05-29-git-worktree-auto-repair.md
git commit -m "feat: detect git worktree roots"
```

Record the commit hash and test result in the Execution Log.

---

### Task 3: Add Git Repair, Linked Move, And Verification Operations

**Files:**
- Modify: `src/git_worktree.rs`
- Modify: `tests/git_worktree.rs`

- [x] **Step 1: Add failing operation tests**

Append these tests to `tests/git_worktree.rs`:

```rust
use codex_project_mover::git_worktree::{
    move_linked_worktree, repair_main_worktree_after_copy, verify_git_worktree_state,
};

#[test]
fn moves_linked_worktree_with_git_command() {
    if !git_available() {
        eprintln!("skipping moves_linked_worktree_with_git_command because git is unavailable");
        return;
    }

    let temp = tempdir().unwrap();
    let main = temp.path().join("repo");
    let linked = temp.path().join("linked");
    let new_linked = temp.path().join("nested/moved-linked");
    init_repo(&main);
    git(&main, &["worktree", "add", linked.to_str().unwrap(), "-b", "feature"]);

    let plan = build_plan_for_existing_project(&linked, &new_linked).unwrap();
    move_linked_worktree(&plan).unwrap();

    assert!(!linked.exists());
    assert!(new_linked.exists());
    git(&new_linked, &["status", "--short"]);
}

#[test]
fn verifies_git_state_rejects_old_worktree_path() {
    let plan = codex_project_mover::git_worktree::GitWorktreePlan {
        kind: GitWorktreeKind::MainWorktree,
        project_path: Path::new("/old/project").to_path_buf(),
        new_project_path: Path::new("/new/project").to_path_buf(),
        entries: parse_worktree_list(b"worktree /old/project/.worktrees/feature\0HEAD a\0branch refs/heads/feature\0\0").unwrap(),
        path_moves: vec![],
    };

    let error = verify_git_worktree_state(&plan).unwrap_err().to_string();

    assert!(error.contains("old Git worktree path remains"));
}

#[test]
fn repair_paths_exclude_main_worktree_path() {
    let entries = parse_worktree_list(
        b"worktree /old/project\0HEAD a\0branch refs/heads/main\0\0worktree /old/project/.worktrees/feature\0HEAD b\0branch refs/heads/feature\0\0",
    )
    .unwrap();
    let plan = codex_project_mover::git_worktree::GitWorktreePlan {
        kind: GitWorktreeKind::MainWorktree,
        project_path: Path::new("/old/project").to_path_buf(),
        new_project_path: Path::new("/new/project").to_path_buf(),
        path_moves: map_worktree_paths(
            &entries,
            Path::new("/old/project"),
            Path::new("/new/project"),
        ),
        entries,
    };

    assert_eq!(
        plan.repair_paths(),
        vec![Path::new("/new/project/.worktrees/feature").to_path_buf()]
    );
}
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test --test git_worktree
```

Expected: FAIL because repair, move, and verify functions do not exist.

- [x] **Step 3: Implement operation functions**

Add this code to `src/git_worktree.rs`:

```rust
pub fn repair_main_worktree_after_copy(plan: &GitWorktreePlan) -> Result<()> {
    if plan.kind != GitWorktreeKind::MainWorktree {
        return Ok(());
    }

    let mut command = Command::new("git");
    command
        .current_dir(&plan.new_project_path)
        .args(["worktree", "repair"]);
    for path in plan.repair_paths() {
        command.arg(path);
    }
    run_git_command(command, "git worktree repair")
}

pub fn move_linked_worktree(plan: &GitWorktreePlan) -> Result<()> {
    if plan.kind != GitWorktreeKind::LinkedWorktree {
        return Ok(());
    }

    if let Some(parent) = plan.new_project_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let cwd = main_worktree_cwd(plan)?;
    let mut command = Command::new("git");
    command
        .current_dir(&cwd)
        .args(["worktree", "move"])
        .arg(&plan.project_path)
        .arg(&plan.new_project_path);
    run_git_command(command, "git worktree move")
}

pub fn verify_git_worktree_state(plan: &GitWorktreePlan) -> Result<()> {
    if plan.kind == GitWorktreeKind::NotGit {
        return Ok(());
    }

    for entry in &plan.entries {
        if entry.path.starts_with(&plan.project_path) {
            bail!("old Git worktree path remains: {}", entry.path.display());
        }
        if let Some(reason) = &entry.prunable {
            bail!(
                "Git worktree is prunable: {} ({})",
                entry.path.display(),
                reason
            );
        }
    }

    let status_cwd = if plan.new_project_path.exists() {
        &plan.new_project_path
    } else {
        &plan.project_path
    };
    let mut status = Command::new("git");
    status.current_dir(status_cwd).args(["status", "--short"]);
    run_git_command(status, "git status --short")
}

pub fn verify_git_from_new_path(old: &Path, new: &Path) -> Result<()> {
    let mut plan = build_plan_for_existing_project(new, new)?;
    plan.project_path = old.to_path_buf();
    verify_git_worktree_state(&plan)
}

fn main_worktree_cwd(plan: &GitWorktreePlan) -> Result<PathBuf> {
    plan.entries
        .iter()
        .find(|entry| entry.path != plan.project_path && !entry.bare)
        .map(|entry| entry.path.clone())
        .or_else(|| {
            plan.entries
                .iter()
                .find(|entry| entry.path == plan.project_path)
                .and_then(|_| plan.project_path.parent().map(Path::to_path_buf))
        })
        .ok_or_else(|| anyhow::anyhow!("could not resolve a main Git worktree cwd"))
}

fn run_git_command(mut command: Command, label: &str) -> Result<()> {
    let output = command.output()?;
    if output.status.success() {
        return Ok(());
    }

    bail!(
        "{label} failed with status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
```

- [x] **Step 4: Run operation tests**

Run:

```bash
cargo test --test git_worktree
```

Expected: PASS.

- [x] **Step 5: Commit Task 3**

Run:

```bash
git add src/git_worktree.rs tests/git_worktree.rs docs/superpowers/plans/2026-05-29-git-worktree-auto-repair.md
git commit -m "feat: repair and verify git worktrees"
```

Record the commit hash and test result in the Execution Log.

---

### Task 4: Add Git Worktree Information To `plan`

**Files:**
- Modify: `src/commands/plan.rs`
- Modify: `tests/cli.rs`

- [x] **Step 1: Add failing CLI plan test**

Append this test to `tests/cli.rs`:

```rust
#[test]
fn plan_reports_no_git_worktree_repair_for_plain_folder() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let old = temp.path().join("old-project");
    let new = temp.path().join("new-project");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::create_dir_all(&old).unwrap();

    mover()
        .args([
            "plan",
            "--old",
            old.to_str().unwrap(),
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("Git worktree: none detected"));
}
```

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test cli plan_reports_no_git_worktree_repair_for_plain_folder
```

Expected: FAIL because `plan` does not print Git worktree information.

- [x] **Step 3: Update `plan` command output**

Modify `src/commands/plan.rs` to import and print Git planning details:

```rust
use crate::git_worktree::{build_plan_for_existing_project, GitWorktreeKind};
```

Add this block after the existing metadata reference loop:

```rust
    let git_plan = build_plan_for_existing_project(&old, &new)?;
    match git_plan.kind {
        GitWorktreeKind::NotGit => {
            println!("Git worktree: none detected");
        }
        GitWorktreeKind::MainWorktree => {
            println!("Git worktree: main worktree");
            println!("Git worktree entries: {}", git_plan.entries.len());
            for path_move in &git_plan.path_moves {
                println!(
                    "- Git path move: {} -> {}",
                    path_move.old_path.display(),
                    path_move.new_path.display()
                );
            }
            println!("Git repair: git -C {} worktree repair", new.display());
        }
        GitWorktreeKind::LinkedWorktree => {
            println!("Git worktree: linked worktree");
            println!(
                "Git move: git worktree move {} {}",
                old.display(),
                new.display()
            );
        }
    }
```

- [x] **Step 4: Run plan test**

Run:

```bash
cargo test --test cli plan_reports_no_git_worktree_repair_for_plain_folder
```

Expected: PASS.

- [x] **Step 5: Commit Task 4**

Run:

```bash
git add src/commands/plan.rs tests/cli.rs docs/superpowers/plans/2026-05-29-git-worktree-auto-repair.md
git commit -m "feat: report git worktree move plan"
```

Record the commit hash and test result in the Execution Log.

---

### Task 5: Integrate Git Worktree Repair Into `apply`

**Files:**
- Modify: `src/commands/apply.rs`
- Create: `tests/git_worktree_cli.rs`

- [ ] **Step 1: Add CLI integration tests for main and linked worktrees**

Create `tests/git_worktree_cli.rs` with:

```rust
use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use std::process::Command as ProcessCommand;
use tempfile::tempdir;

fn mover() -> Command {
    let mut cmd = Command::cargo_bin("codex-project-mover").unwrap();
    cmd.env("CODEX_PROJECT_MOVER_TEST_SKIP_PROCESS_GUARD", "1");
    cmd
}

fn git_available() -> bool {
    ProcessCommand::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn git(cwd: &std::path::Path, args: &[&str]) {
    let output = ProcessCommand::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn init_repo(path: &std::path::Path) {
    fs::create_dir_all(path).unwrap();
    git(path, &["init"]);
    git(path, &["config", "user.email", "test@example.com"]);
    git(path, &["config", "user.name", "Test User"]);
    fs::write(path.join("README.md"), "hello\n").unwrap();
    git(path, &["add", "README.md"]);
    git(path, &["commit", "-m", "initial"]);
}

#[test]
fn apply_repairs_main_worktree_with_nested_linked_worktree() {
    if !git_available() {
        eprintln!("skipping apply_repairs_main_worktree_with_nested_linked_worktree because git is unavailable");
        return;
    }

    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let old = temp.path().join("old-project");
    let linked = old.join(".worktrees/feature");
    let new = temp.path().join("new-project");
    let new_linked = new.join(".worktrees/feature");
    let test_trash = temp.path().join("trash");
    fs::create_dir_all(home.join("sessions")).unwrap();
    init_repo(&old);
    git(&old, &["worktree", "add", linked.to_str().unwrap(), "-b", "feature"]);
    fs::write(
        home.join("sessions/thread.jsonl"),
        format!(r#"{{"cwd":"{}"}}"#, old.display()),
    )
    .unwrap();

    mover()
        .env("CODEX_PROJECT_MOVER_TEST_TRASH_DIR", &test_trash)
        .args([
            "apply",
            "--old",
            old.to_str().unwrap(),
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("Git worktree repair complete"))
        .stdout(contains("move complete: old project folder moved to Trash"));

    git(&new, &["status", "--short"]);
    git(&new_linked, &["status", "--short"]);
    let output = ProcessCommand::new("git")
        .current_dir(&new)
        .args(["worktree", "list", "--porcelain"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(new.to_str().unwrap()));
    assert!(stdout.contains(new_linked.to_str().unwrap()));
    assert!(!stdout.contains(old.to_str().unwrap()));
}

#[test]
fn apply_moves_linked_worktree_with_git_worktree_move() {
    if !git_available() {
        eprintln!("skipping apply_moves_linked_worktree_with_git_worktree_move because git is unavailable");
        return;
    }

    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let main = temp.path().join("main");
    let old = temp.path().join("linked");
    let new = temp.path().join("moved/linked");
    fs::create_dir_all(home.join("sessions")).unwrap();
    init_repo(&main);
    git(&main, &["worktree", "add", old.to_str().unwrap(), "-b", "feature"]);
    fs::write(
        home.join("sessions/thread.jsonl"),
        format!(r#"{{"cwd":"{}"}}"#, old.display()),
    )
    .unwrap();

    mover()
        .args([
            "apply",
            "--old",
            old.to_str().unwrap(),
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("Git worktree move complete"));

    assert!(!old.exists());
    assert!(new.exists());
    git(&new, &["status", "--short"]);
    assert!(
        fs::read_to_string(home.join("sessions/thread.jsonl"))
            .unwrap()
            .contains(new.to_str().unwrap())
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test --test git_worktree_cli
```

Expected: FAIL because `apply` does not call Git repair or linked worktree move yet.

- [ ] **Step 3: Update `apply` imports and branch on Git kind**

Modify `src/commands/apply.rs` imports:

```rust
use crate::git_worktree::{
    build_plan_for_existing_project, move_linked_worktree, repair_main_worktree_after_copy,
    verify_git_from_new_path, GitWorktreeKind,
};
```

Replace the current `if !args.relink_only { copy_project_tree... }` block with:

```rust
    let git_plan = if args.relink_only {
        None
    } else {
        Some(build_plan_for_existing_project(&old, &new)?)
    };

    if !args.relink_only {
        match git_plan.as_ref().map(|plan| &plan.kind) {
            Some(GitWorktreeKind::LinkedWorktree) => {
                move_linked_worktree(git_plan.as_ref().unwrap())?;
                println!("Git worktree move complete");
            }
            Some(GitWorktreeKind::MainWorktree) => {
                copy_project_tree(&old, &new)?;
                verify_project_tree(&old, &new)
                    .with_context(|| "copied project tree failed verification; metadata was not changed and old folder was not moved to Trash")?;
                repair_main_worktree_after_copy(git_plan.as_ref().unwrap())?;
                verify_git_from_new_path(&old, &new)
                    .with_context(|| "Git worktree verification failed after repair; metadata was not changed and old folder was not moved to Trash")?;
                println!("Git worktree repair complete");
            }
            Some(GitWorktreeKind::NotGit) | None => {
                copy_project_tree(&old, &new)?;
                verify_project_tree(&old, &new)
                    .with_context(|| "copied project tree failed verification; metadata was not changed and old folder was not moved to Trash")?;
            }
        }
    }
```

Replace the final old-folder cleanup:

```rust
    if !args.relink_only {
        move_to_trash(&old)?;
    }
```

with:

```rust
    if !args.relink_only
        && !matches!(
            git_plan.as_ref().map(|plan| &plan.kind),
            Some(GitWorktreeKind::LinkedWorktree)
        )
    {
        move_to_trash(&old)?;
    }
```

Update the final success message branch:

```rust
    } else if matches!(
        git_plan.as_ref().map(|plan| &plan.kind),
        Some(GitWorktreeKind::LinkedWorktree)
    ) {
        println!("move complete: linked Git worktree moved with git worktree move");
    } else {
        println!("move complete: old project folder moved to Trash");
    }
```

- [ ] **Step 4: Run apply Git integration tests**

Run:

```bash
cargo test --test git_worktree_cli
```

Expected: PASS.

- [ ] **Step 5: Commit Task 5**

Run:

```bash
git add src/commands/apply.rs tests/git_worktree_cli.rs docs/superpowers/plans/2026-05-29-git-worktree-auto-repair.md
git commit -m "feat: auto repair git worktrees during apply"
```

Record the commit hash and test result in the Execution Log.

---

### Task 6: Add Git Repair To `relink-only`

**Files:**
- Modify: `src/git_worktree.rs`
- Modify: `src/commands/apply.rs`
- Modify: `tests/git_worktree_cli.rs`

- [ ] **Step 1: Add failing relink-only test**

Append this test to `tests/git_worktree_cli.rs`:

```rust
#[test]
fn apply_relink_only_repairs_manually_moved_main_worktree() {
    if !git_available() {
        eprintln!("skipping apply_relink_only_repairs_manually_moved_main_worktree because git is unavailable");
        return;
    }

    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let old = temp.path().join("old-project");
    let linked = old.join(".worktrees/feature");
    let new = temp.path().join("new-project");
    let new_linked = new.join(".worktrees/feature");
    fs::create_dir_all(home.join("sessions")).unwrap();
    init_repo(&old);
    git(&old, &["worktree", "add", linked.to_str().unwrap(), "-b", "feature"]);
    fs::rename(&old, &new).unwrap();
    fs::write(
        home.join("sessions/thread.jsonl"),
        format!(r#"{{"cwd":"{}"}}"#, old.display()),
    )
    .unwrap();

    mover()
        .args([
            "apply",
            "--old",
            old.to_str().unwrap(),
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
            "--relink-only",
        ])
        .assert()
        .success()
        .stdout(contains("Git worktree repair complete"))
        .stdout(contains("relink-only complete: project folder was not moved"));

    git(&new, &["status", "--short"]);
    git(&new_linked, &["status", "--short"]);
}

#[test]
fn apply_relink_only_repairs_manually_moved_linked_worktree() {
    if !git_available() {
        eprintln!("skipping apply_relink_only_repairs_manually_moved_linked_worktree because git is unavailable");
        return;
    }

    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let main = temp.path().join("main");
    let old = temp.path().join("linked");
    let new = temp.path().join("moved/linked");
    fs::create_dir_all(home.join("sessions")).unwrap();
    init_repo(&main);
    git(&main, &["worktree", "add", old.to_str().unwrap(), "-b", "feature"]);
    fs::create_dir_all(new.parent().unwrap()).unwrap();
    fs::rename(&old, &new).unwrap();
    fs::write(
        home.join("sessions/thread.jsonl"),
        format!(r#"{{"cwd":"{}"}}"#, old.display()),
    )
    .unwrap();

    mover()
        .args([
            "apply",
            "--old",
            old.to_str().unwrap(),
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
            "--relink-only",
        ])
        .assert()
        .success()
        .stdout(contains("Git worktree repair complete"))
        .stdout(contains("relink-only complete: project folder was not moved"));

    git(&new, &["status", "--short"]);
    let output = ProcessCommand::new("git")
        .current_dir(&main)
        .args(["worktree", "list", "--porcelain"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(new.to_str().unwrap()));
    assert!(!stdout.contains(old.to_str().unwrap()));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test git_worktree_cli
```

Expected: FAIL because relink-only does not run Git repair.

- [ ] **Step 3: Add relink plan builder and apply integration**

Add this function to `src/git_worktree.rs`:

```rust
pub fn build_plan_for_relink_only(old: &Path, new: &Path) -> Result<GitWorktreePlan> {
    match inspect_dotgit(new)? {
        GitDotfile::Missing => Ok(GitWorktreePlan::no_git(old, new)),
        GitDotfile::Directory(_) => {
            let entries = git_worktree_entries(new)?;
            Ok(GitWorktreePlan {
                kind: GitWorktreeKind::MainWorktree,
                project_path: old.to_path_buf(),
                new_project_path: new.to_path_buf(),
                path_moves: map_worktree_paths(&entries, old, new),
                entries,
            })
        }
        GitDotfile::File { gitdir } => {
            let entries = git_worktree_entries_from_gitdir(&gitdir)?;
            Ok(GitWorktreePlan {
                kind: GitWorktreeKind::LinkedWorktree,
                project_path: old.to_path_buf(),
                new_project_path: new.to_path_buf(),
                path_moves: map_worktree_paths(&entries, old, new),
                entries,
            })
        }
    }
}
```

Add these helpers to `src/git_worktree.rs`:

```rust
fn git_worktree_entries_from_gitdir(gitdir: &Path) -> Result<Vec<WorktreeEntry>> {
    let common_dir = resolve_common_dir(gitdir)?;
    let main_worktree = common_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("could not resolve main worktree from {}", common_dir.display()))?;
    git_worktree_entries(main_worktree)
}

fn resolve_common_dir(gitdir: &Path) -> Result<PathBuf> {
    let commondir = gitdir.join("commondir");
    if !commondir.exists() {
        return Ok(gitdir.to_path_buf());
    }

    let raw = fs::read_to_string(&commondir)?;
    let path = PathBuf::from(raw.trim());
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(gitdir.join(path))
    }
}

pub fn repair_linked_worktree_after_manual_move(plan: &GitWorktreePlan) -> Result<()> {
    if plan.kind != GitWorktreeKind::LinkedWorktree {
        return Ok(());
    }

    let cwd = main_worktree_cwd(plan)?;
    let mut command = Command::new("git");
    command
        .current_dir(&cwd)
        .args(["worktree", "repair"])
        .arg(&plan.new_project_path);
    run_git_command(command, "git worktree repair")
}
```

Modify `src/commands/apply.rs` imports to include:

```rust
    build_plan_for_relink_only, repair_linked_worktree_after_manual_move,
```

Replace the `git_plan` initialization with:

```rust
    let git_plan = if args.relink_only {
        Some(build_plan_for_relink_only(&old, &new)?)
    } else {
        Some(build_plan_for_existing_project(&old, &new)?)
    };
```

Add this block before Codex metadata updates:

```rust
    if args.relink_only {
        if let Some(plan) = &git_plan {
            match plan.kind {
                GitWorktreeKind::MainWorktree => {
                    repair_main_worktree_after_copy(plan)?;
                    verify_git_from_new_path(&old, &new)
                        .with_context(|| "Git worktree verification failed after relink-only repair; metadata was not changed")?;
                    println!("Git worktree repair complete");
                }
                GitWorktreeKind::LinkedWorktree => {
                    repair_linked_worktree_after_manual_move(plan)?;
                    verify_git_from_new_path(&old, &new)
                        .with_context(|| "Git linked worktree verification failed after relink-only repair; metadata was not changed")?;
                    println!("Git worktree repair complete");
                }
                GitWorktreeKind::NotGit => {}
            }
        }
    }
```

- [ ] **Step 4: Run relink-only test**

Run:

```bash
cargo test --test git_worktree_cli
```

Expected: PASS.

- [ ] **Step 5: Commit Task 6**

Run:

```bash
git add src/git_worktree.rs src/commands/apply.rs tests/git_worktree_cli.rs docs/superpowers/plans/2026-05-29-git-worktree-auto-repair.md
git commit -m "feat: repair git worktrees during relink-only"
```

Record the commit hash and test result in the Execution Log.

---

### Task 7: Integrate Git Validation Into `verify`

**Files:**
- Modify: `src/commands/verify.rs`
- Modify: `tests/git_worktree_cli.rs`

- [ ] **Step 1: Add failing verify CLI tests**

Append this test to `tests/git_worktree_cli.rs`:

```rust
#[test]
fn verify_reports_git_validation_for_moved_repo() {
    if !git_available() {
        eprintln!("skipping verify_reports_git_validation_for_moved_repo because git is unavailable");
        return;
    }

    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let old = temp.path().join("old-project");
    let new = temp.path().join("new-project");
    fs::create_dir_all(home.join("sessions")).unwrap();
    init_repo(&new);
    fs::write(
        home.join("sessions/thread.jsonl"),
        format!(r#"{{"cwd":"{}"}}"#, new.display()),
    )
    .unwrap();

    mover()
        .args([
            "verify",
            "--old",
            old.to_str().unwrap(),
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("Git worktree verification passed"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test git_worktree_cli verify_reports_git_validation_for_moved_repo
```

Expected: FAIL because `verify` does not print Git verification output.

- [ ] **Step 3: Update verify command**

Modify `src/commands/verify.rs` imports:

```rust
use crate::git_worktree::{build_plan_for_relink_only, verify_git_from_new_path, GitWorktreeKind};
```

Add this block after Codex metadata verification passes and before the final `println!`:

```rust
    let git_plan = build_plan_for_relink_only(&old, &new)?;
    if git_plan.kind != GitWorktreeKind::NotGit {
        verify_git_from_new_path(&old, &new)?;
        println!("Git worktree verification passed");
    }
```

- [ ] **Step 4: Run verify test**

Run:

```bash
cargo test --test git_worktree_cli verify_reports_git_validation_for_moved_repo
```

Expected: PASS.

- [ ] **Step 5: Commit Task 7**

Run:

```bash
git add src/commands/verify.rs tests/git_worktree_cli.rs docs/superpowers/plans/2026-05-29-git-worktree-auto-repair.md
git commit -m "feat: verify git worktree state"
```

Record the commit hash and test result in the Execution Log.

---

### Task 8: Document Behavior And Run Full Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-05-29-git-worktree-auto-repair.md`

- [ ] **Step 1: Update README**

Add this paragraph after the normal `apply` behavior paragraph in `README.md`:

```markdown
When the project folder is a Git worktree root, `apply` automatically repairs Git worktree metadata. Main worktrees are copied and then repaired with `git worktree repair` before Codex metadata changes. Linked worktrees are moved with `git worktree move` instead of a generic filesystem copy. Codex paths should always point at the checkout root, not at `.git/worktrees/...` internals.
```

Add this paragraph after the relink-only paragraph:

```markdown
Relink-only also attempts Git worktree repair from the new path when the project has already been moved manually. If Git repair fails, the tool stops before updating Codex metadata and prints the Git command context.
```

- [ ] **Step 2: Run formatting and full tests**

Run:

```bash
cargo fmt
cargo test
```

Expected: PASS for all tests. Git-dependent tests may print skip messages only when `git` is unavailable.

- [ ] **Step 3: Run CLI help smoke test**

Run:

```bash
cargo run -- --help
```

Expected: PASS and output includes `plan`, `apply`, `verify`, and `rollback`.

- [ ] **Step 4: Commit Task 8**

Run:

```bash
git add README.md docs/superpowers/plans/2026-05-29-git-worktree-auto-repair.md
git commit -m "docs: describe git worktree auto repair"
```

Record the commit hash and test result in the Execution Log.

---

## Final Verification

- [ ] Run `git status --short`.
- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo test`.
- [ ] Run `cargo run -- --help`.
- [ ] Update this plan's Execution Log with final verification commands and results.
- [ ] Report changed files, commits, and any residual risk.
