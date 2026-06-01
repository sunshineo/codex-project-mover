# Codex Project Mover Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Build a Mac-first Rust CLI that moves a Codex project folder and updates supported local Codex metadata from an old exact absolute path to a new exact absolute path.

**Architecture:** The CLI is split into small modules: command parsing, path normalization, process guarding, state discovery, per-surface scanning/updating, metadata backups, project-folder copy verification, Trash handling, and command orchestration. Each metadata surface has a scanner and updater backed by fixture tests so `plan`, `verify`, `apply`, and `rollback` share the same detection logic.

**Tech Stack:** Rust 2021, `clap`, `anyhow`, `serde`, `serde_json`, `rusqlite`, `toml_edit`, `walkdir`, `sha2`, `sysinfo`, `trash`, `tempfile`, `assert_cmd`, `predicates`.

---

## Locked Product Decisions

- Normal `apply --old <old> --new <new>` moves the folder by default.
- Normal apply requires `old` to exist and `new` to not exist.
- Relink-only mode requires `old` to not exist and `new` to exist.
- New parent directories are created automatically.
- Codex process detection is focused on local Codex state writers: the main Codex Desktop process, `codex app-server`, and standalone `codex` CLI/`codex exec` processes. The guard reports and exits by default, never stops processes, and can be bypassed only with the explicit `--allow-running-codex` user override.
- Project movement uses metadata scan, metadata backup, copy, copy verification, metadata update, metadata verification, then moves the old folder to macOS Trash.
- Backups are metadata backups under `~/.codex/codex-project-mover-backups/<id>` and include movement metadata for rollback cleanup.
- Rollback restores metadata files from a backup manifest and moves the created new project folder to Trash when the manifest records one. It does not restore the old folder from Trash.
- `.codex-global-state.json` updates JSON string values and JSON object keys only when the whole string/key exactly equals the old path.
- Session JSONL updates only structured fields named `cwd` when the whole string exactly equals the old path.
- `config.toml` updates exact string values, exact `[projects."/path"]` table keys, and exact per-path open-target preference keys.
- `verify` fails unless supported old-path references are gone and supported new-path references are present.
- Paths are normalized to absolute, trailing-slash-free paths before matching.

## File Structure

- Create `Cargo.toml`: package metadata and crate dependencies.
- Create `src/main.rs`: binary entrypoint that prints user-facing errors.
- Create `src/lib.rs`: public module wiring and `run`.
- Create `src/cli.rs`: `clap` command definitions.
- Create `src/error.rs`: small helpers for consistent error context.
- Create `src/pathing.rs`: path normalization and Codex home discovery.
- Create `src/process_guard.rs`: aggressive Codex process detection.
- Create `src/model.rs`: shared structs for plans, matches, backups, and command results.
- Create `src/discovery.rs`: discover supported Codex state surfaces.
- Create `src/surfaces/mod.rs`: surface module exports.
- Create `src/surfaces/jsonl.rs`: scan and update JSONL `cwd` fields.
- Create `src/surfaces/sqlite_threads.rs`: scan and update `state_*.sqlite` `threads.cwd`.
- Create `src/surfaces/global_state.rs`: scan and update exact JSON string values and exact path object keys.
- Create `src/surfaces/config_toml.rs`: scan and update TOML string values, exact project table keys, and exact per-path preference keys.
- Create `src/surfaces/automation_db.rs`: scan and update SQLite `cwds` columns when stored as JSON arrays or exact strings.
- Create `src/backup.rs`: metadata backup directory, movement metadata, and manifest handling.
- Create `src/project_copy.rs`: recursive copy, tree hashing, and copy verification.
- Create `src/trash.rs`: thin wrapper around macOS Trash behavior.
- Create `src/commands/mod.rs`: command module exports.
- Create `src/commands/plan.rs`: read-only migration plan output.
- Create `src/commands/verify.rs`: old/new reference count verification.
- Create `src/commands/apply.rs`: normal move and relink-only orchestration.
- Create `src/commands/rollback.rs`: metadata restore orchestration.
- Create `tests/fixtures.rs`: helpers that create temporary Codex homes, JSONL files, TOML files, and SQLite DBs.
- Create `tests/cli.rs`: end-to-end CLI behavior tests.

## Task 1: Scaffold the Rust CLI

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/cli.rs`
- Create: `src/error.rs`
- Test: `tests/cli.rs`

- [x] **Step 1: Write the failing CLI smoke test**

```rust
// tests/cli.rs
use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn help_lists_core_subcommands() {
    let mut cmd = Command::cargo_bin("codex-project-mover").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("plan"))
        .stdout(contains("apply"))
        .stdout(contains("verify"))
        .stdout(contains("rollback"));
}
```

- [x] **Step 2: Run the smoke test and verify it fails**

Run: `cargo test help_lists_core_subcommands --test cli`

Expected: FAIL because the crate has not been scaffolded yet.

- [x] **Step 3: Create the CLI scaffold**

```toml
# Cargo.toml
[package]
name = "codex-project-mover"
version = "0.1.0"
edition = "2021"
rust-version = "1.78"

[dependencies]
anyhow = "1"
chrono = { version = "0.4", default-features = false, features = ["clock", "serde"] }
clap = { version = "4.5", features = ["derive"] }
dirs = "5"
rusqlite = { version = "0.32", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
sysinfo = "0.33"
toml_edit = "0.22"
trash = "5"
walkdir = "2"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

```rust
// src/main.rs
fn main() {
    if let Err(error) = codex_project_mover::run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
```

```rust
// src/lib.rs
pub mod cli;
pub mod error;

pub fn run() -> anyhow::Result<()> {
    let cli = <cli::Cli as clap::Parser>::parse();
    match cli.command {
        cli::Command::Plan(args) => {
            println!("planning move from {} to {}", args.old.display(), args.new.display());
            Ok(())
        }
        cli::Command::Apply(args) => {
            println!("applying move from {} to {}", args.old.display(), args.new.display());
            Ok(())
        }
        cli::Command::Verify(args) => {
            println!("verifying move from {} to {}", args.old.display(), args.new.display());
            Ok(())
        }
        cli::Command::Rollback(args) => {
            println!("rolling back from {}", args.backup.display());
            Ok(())
        }
    }
}
```

```rust
// src/cli.rs
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "codex-project-mover")]
#[command(about = "Move a Codex Desktop project and update local Codex metadata")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Plan(MoveArgs),
    Apply(ApplyArgs),
    Verify(MoveArgs),
    Rollback(RollbackArgs),
}

#[derive(Debug, Clone, Args)]
pub struct MoveArgs {
    #[arg(long)]
    pub old: PathBuf,
    #[arg(long)]
    pub new: PathBuf,
    #[arg(long)]
    pub codex_home: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct ApplyArgs {
    #[arg(long)]
    pub old: PathBuf,
    #[arg(long)]
    pub new: PathBuf,
    #[arg(long)]
    pub codex_home: Option<PathBuf>,
    #[arg(long)]
    pub relink_only: bool,
}

#[derive(Debug, Clone, Args)]
pub struct RollbackArgs {
    #[arg(long)]
    pub backup: PathBuf,
}
```

```rust
// src/error.rs
pub fn user_error(message: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!(message.into())
}
```

- [x] **Step 4: Run the smoke test and verify it passes**

Run: `cargo test help_lists_core_subcommands --test cli`

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add Cargo.toml src/main.rs src/lib.rs src/cli.rs src/error.rs tests/cli.rs
git commit -m "chore: scaffold Rust CLI"
```

## Task 2: Normalize Project Paths and Discover Codex Home

**Files:**
- Create: `src/pathing.rs`
- Modify: `src/lib.rs`
- Test: `tests/pathing.rs`

- [x] **Step 1: Write failing path tests**

```rust
// tests/pathing.rs
use std::path::PathBuf;

use codex_project_mover::pathing::{codex_home_from_arg, normalize_project_path};

#[test]
fn normalize_project_path_makes_relative_path_absolute() {
    let normalized = normalize_project_path(PathBuf::from("relative/project")).unwrap();

    assert!(normalized.is_absolute());
    assert!(normalized.ends_with("relative/project"));
}

#[test]
fn normalize_project_path_removes_dot_components() {
    let normalized = normalize_project_path(PathBuf::from("/tmp/./old/../old/project/")).unwrap();

    assert_eq!(normalized, PathBuf::from("/tmp/old/project"));
}

#[test]
fn codex_home_uses_explicit_arg_first() {
    let home = codex_home_from_arg(Some(PathBuf::from("/tmp/custom-codex"))).unwrap();

    assert_eq!(home, PathBuf::from("/tmp/custom-codex"));
}
```

- [x] **Step 2: Run tests and verify they fail**

Run: `cargo test --test pathing`

Expected: FAIL because `src/pathing.rs` does not exist.

- [x] **Step 3: Implement path normalization and Codex home discovery**

```rust
// src/lib.rs
pub mod cli;
pub mod error;
pub mod pathing;
```

```rust
// src/pathing.rs
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};

pub fn normalize_project_path(path: PathBuf) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .context("read current directory")?
            .join(path)
    };

    Ok(clean_path(&absolute))
}

pub fn codex_home_from_arg(path: Option<PathBuf>) -> Result<PathBuf> {
    match path {
        Some(path) => Ok(clean_path(&path)),
        None => dirs::home_dir()
            .map(|home| home.join(".codex"))
            .map(|path| clean_path(&path))
            .context("could not determine home directory for default ~/.codex"),
    }
}

fn clean_path(path: &Path) -> PathBuf {
    let mut cleaned = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                cleaned.pop();
            }
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
                cleaned.push(component.as_os_str());
            }
        }
    }
    cleaned
}
```

- [x] **Step 4: Run tests and verify they pass**

Run: `cargo test --test pathing`

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add src/lib.rs src/pathing.rs tests/pathing.rs
git commit -m "feat: normalize project paths"
```

## Task 3: Add Aggressive Codex Process Guard

**Files:**
- Create: `src/process_guard.rs`
- Modify: `src/lib.rs`
- Test: `tests/process_guard.rs`

- [x] **Step 1: Write failing process matching tests**

```rust
// tests/process_guard.rs
use codex_project_mover::process_guard::{find_codex_processes, ProcessInfo};

#[test]
fn detects_codex_in_process_name_or_command() {
    let processes = vec![
        ProcessInfo::new(10, "zsh", "/bin/zsh"),
        ProcessInfo::new(11, "Codex", "/Applications/Codex.app/Contents/MacOS/Codex"),
        ProcessInfo::new(12, "node", "node /tmp/codex app-server"),
    ];

    let matches = find_codex_processes(&processes, 99);

    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].pid, 11);
    assert_eq!(matches[1].pid, 12);
}

#[test]
fn excludes_current_mover_process() {
    let processes = vec![ProcessInfo::new(
        22,
        "codex-project-mover",
        "/usr/local/bin/codex-project-mover apply",
    )];

    let matches = find_codex_processes(&processes, 22);

    assert!(matches.is_empty());
}
```

- [x] **Step 2: Run tests and verify they fail**

Run: `cargo test --test process_guard`

Expected: FAIL because `src/process_guard.rs` does not exist.

- [x] **Step 3: Implement process matching and live guard**

```rust
// src/lib.rs
pub mod cli;
pub mod error;
pub mod pathing;
pub mod process_guard;
```

```rust
// src/process_guard.rs
use anyhow::{bail, Result};
use sysinfo::System;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command: String,
}

impl ProcessInfo {
    pub fn new(pid: u32, name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            pid,
            name: name.into(),
            command: command.into(),
        }
    }
}

pub fn assert_no_codex_processes() -> Result<()> {
    let current_pid = std::process::id();
    let mut system = System::new_all();
    system.refresh_all();

    let processes = system
        .processes()
        .iter()
        .map(|(pid, process)| {
            ProcessInfo::new(
                pid.as_u32(),
                process.name().to_string_lossy(),
                process.cmd().join(" ").to_string_lossy(),
            )
        })
        .collect::<Vec<_>>();

    let matches = find_codex_processes(&processes, current_pid);
    if matches.is_empty() {
        return Ok(());
    }

    let rendered = matches
        .iter()
        .map(|process| format!("pid {}: {} {}", process.pid, process.name, process.command))
        .collect::<Vec<_>>()
        .join("\n");

    bail!(
        "Codex-related processes are running. Close Codex and related helper processes, then retry.\n{}",
        rendered
    );
}

pub fn find_codex_processes(processes: &[ProcessInfo], current_pid: u32) -> Vec<ProcessInfo> {
    processes
        .iter()
        .filter(|process| process.pid != current_pid)
        .filter(|process| {
            let haystack = format!("{} {}", process.name, process.command).to_ascii_lowercase();
            haystack.contains("codex") && !haystack.contains("codex-project-mover")
        })
        .cloned()
        .collect()
}
```

- [x] **Step 4: Run tests and verify they pass**

Run: `cargo test --test process_guard`

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add src/lib.rs src/process_guard.rs tests/process_guard.rs
git commit -m "feat: block when Codex processes are running"
```

## Task 4: Define Discovery and Migration Plan Models

**Files:**
- Create: `src/model.rs`
- Create: `src/discovery.rs`
- Modify: `src/lib.rs`
- Test: `tests/discovery.rs`

- [x] **Step 1: Write failing discovery tests**

```rust
// tests/discovery.rs
use std::fs;

use codex_project_mover::discovery::discover_state;
use tempfile::tempdir;

#[test]
fn discovers_supported_codex_state_files() {
    let temp = tempdir().unwrap();
    let codex_home = temp.path();
    fs::create_dir_all(codex_home.join("sessions/2026")).unwrap();
    fs::create_dir_all(codex_home.join("archived_sessions/2025")).unwrap();
    fs::create_dir_all(codex_home.join("sqlite")).unwrap();
    fs::write(codex_home.join("state_main.sqlite"), "").unwrap();
    fs::write(codex_home.join("sessions/2026/thread.jsonl"), "").unwrap();
    fs::write(codex_home.join("archived_sessions/2025/thread.jsonl"), "").unwrap();
    fs::write(codex_home.join(".codex-global-state.json"), "{}").unwrap();
    fs::write(codex_home.join("config.toml"), "").unwrap();
    fs::write(codex_home.join("sqlite/codex-dev.db"), "").unwrap();

    let state = discover_state(codex_home).unwrap();

    assert_eq!(state.sqlite_state_dbs.len(), 1);
    assert_eq!(state.jsonl_files.len(), 2);
    assert!(state.global_state_json.is_some());
    assert!(state.config_toml.is_some());
    assert!(state.automation_db.is_some());
}
```

- [x] **Step 2: Run tests and verify they fail**

Run: `cargo test --test discovery`

Expected: FAIL because discovery modules do not exist.

- [x] **Step 3: Implement shared models and state discovery**

```rust
// src/lib.rs
pub mod cli;
pub mod discovery;
pub mod error;
pub mod model;
pub mod pathing;
pub mod process_guard;
```

```rust
// src/model.rs
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceKind {
    JsonlCwd,
    SqliteThreadsCwd,
    GlobalStateJson,
    ConfigToml,
    AutomationDbCwds,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceMatch {
    pub surface: SurfaceKind,
    pub file: PathBuf,
    pub location: String,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ScanReport {
    pub matches: Vec<ReferenceMatch>,
}

impl ScanReport {
    pub fn old_reference_count(&self) -> usize {
        self.matches.len()
    }
}
```

```rust
// src/discovery.rs
use std::path::{Path, PathBuf};

use anyhow::Result;
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CodexStateFiles {
    pub sqlite_state_dbs: Vec<PathBuf>,
    pub jsonl_files: Vec<PathBuf>,
    pub global_state_json: Option<PathBuf>,
    pub config_toml: Option<PathBuf>,
    pub automation_db: Option<PathBuf>,
}

pub fn discover_state(codex_home: &Path) -> Result<CodexStateFiles> {
    let mut files = CodexStateFiles::default();

    if codex_home.exists() {
        for entry in std::fs::read_dir(codex_home)? {
            let entry = entry?;
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if name.starts_with("state_") && name.ends_with(".sqlite") {
                files.sqlite_state_dbs.push(path);
            }
        }
    }

    for folder in ["sessions", "archived_sessions"] {
        let root = codex_home.join(folder);
        if root.exists() {
            for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
                    files.jsonl_files.push(path.to_path_buf());
                }
            }
        }
    }

    let global_state = codex_home.join(".codex-global-state.json");
    if global_state.exists() {
        files.global_state_json = Some(global_state);
    }

    let config = codex_home.join("config.toml");
    if config.exists() {
        files.config_toml = Some(config);
    }

    let automation_db = codex_home.join("sqlite/codex-dev.db");
    if automation_db.exists() {
        files.automation_db = Some(automation_db);
    }

    files.sqlite_state_dbs.sort();
    files.jsonl_files.sort();
    Ok(files)
}
```

- [x] **Step 4: Run tests and verify they pass**

Run: `cargo test --test discovery`

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add src/lib.rs src/model.rs src/discovery.rs tests/discovery.rs
git commit -m "feat: discover Codex state files"
```

## Task 5: Scan and Update JSONL Session `cwd` Fields

**Files:**
- Create: `src/surfaces/mod.rs`
- Create: `src/surfaces/jsonl.rs`
- Modify: `src/lib.rs`
- Test: `tests/jsonl_surface.rs`

- [x] **Step 1: Write failing JSONL tests**

```rust
// tests/jsonl_surface.rs
use std::fs;

use codex_project_mover::surfaces::jsonl::{scan_jsonl_file, update_jsonl_file};
use tempfile::tempdir;

#[test]
fn scans_only_structured_cwd_fields_equal_to_old_path() {
    let temp = tempdir().unwrap();
    let file = temp.path().join("thread.jsonl");
    fs::write(
        &file,
        r#"{"cwd":"/old/project","message":"do not edit /old/project in text"}"#.to_owned()
            + "\n"
            + r#"{"payload":{"cwd":"/old/project/subdir"}}"#
            + "\n"
            + r#"{"payload":{"cwd":"/old/project"}}"#
            + "\n",
    )
    .unwrap();

    let matches = scan_jsonl_file(&file, "/old/project", "/new/project").unwrap();

    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].location, "line 1 /cwd");
    assert_eq!(matches[1].location, "line 3 /payload/cwd");
}

#[test]
fn updates_only_matching_cwd_fields() {
    let temp = tempdir().unwrap();
    let file = temp.path().join("thread.jsonl");
    fs::write(
        &file,
        r#"{"cwd":"/old/project","message":"do not edit /old/project in text"}"#.to_owned()
            + "\n"
            + r#"{"payload":{"cwd":"/old/project/subdir"}}"#
            + "\n",
    )
    .unwrap();

    let count = update_jsonl_file(&file, "/old/project", "/new/project").unwrap();
    let updated = fs::read_to_string(&file).unwrap();

    assert_eq!(count, 1);
    assert!(updated.contains(r#""cwd":"/new/project""#));
    assert!(updated.contains(r#""message":"do not edit /old/project in text""#));
    assert!(updated.contains(r#""cwd":"/old/project/subdir""#));
}
```

- [x] **Step 2: Run tests and verify they fail**

Run: `cargo test --test jsonl_surface`

Expected: FAIL because JSONL surface code does not exist.

- [x] **Step 3: Implement JSONL scan and update**

```rust
// src/lib.rs
pub mod cli;
pub mod discovery;
pub mod error;
pub mod model;
pub mod pathing;
pub mod process_guard;
pub mod surfaces;
```

```rust
// src/surfaces/mod.rs
pub mod jsonl;
```

```rust
// src/surfaces/jsonl.rs
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::model::{ReferenceMatch, SurfaceKind};

pub fn scan_jsonl_file(path: &Path, old: &str, new: &str) -> Result<Vec<ReferenceMatch>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("read JSONL session file {}", path.display()))?;
    let mut matches = Vec::new();

    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line)
            .with_context(|| format!("parse JSONL line {} in {}", index + 1, path.display()))?;
        collect_cwd_matches(
            &value,
            old,
            new,
            path,
            format!("line {}", index + 1),
            String::new(),
            &mut matches,
        );
    }

    Ok(matches)
}

pub fn update_jsonl_file(path: &Path, old: &str, new: &str) -> Result<usize> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("read JSONL session file {}", path.display()))?;
    let mut changed_count = 0;
    let mut output = String::new();

    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            output.push('\n');
            continue;
        }
        let mut value: Value = serde_json::from_str(line)
            .with_context(|| format!("parse JSONL line {} in {}", index + 1, path.display()))?;
        changed_count += replace_cwd_values(&mut value, old, new);
        output.push_str(&serde_json::to_string(&value)?);
        output.push('\n');
    }

    fs::write(path, output).with_context(|| format!("write JSONL session file {}", path.display()))?;
    Ok(changed_count)
}

fn collect_cwd_matches(
    value: &Value,
    old: &str,
    new: &str,
    file: &Path,
    line_label: String,
    pointer: String,
    matches: &mut Vec<ReferenceMatch>,
) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_pointer = format!("{}/{}", pointer, escape_pointer(key));
                if key == "cwd" && child.as_str() == Some(old) {
                    matches.push(ReferenceMatch {
                        surface: SurfaceKind::JsonlCwd,
                        file: file.to_path_buf(),
                        location: format!("{} {}", line_label, child_pointer),
                        old_value: old.to_string(),
                        new_value: new.to_string(),
                    });
                }
                collect_cwd_matches(child, old, new, file, line_label.clone(), child_pointer, matches);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect_cwd_matches(
                    child,
                    old,
                    new,
                    file,
                    line_label.clone(),
                    format!("{}/{}", pointer, index),
                    matches,
                );
            }
        }
        _ => {}
    }
}

fn replace_cwd_values(value: &mut Value, old: &str, new: &str) -> usize {
    match value {
        Value::Object(map) => map
            .iter_mut()
            .map(|(key, child)| {
                let direct = if key == "cwd" && child.as_str() == Some(old) {
                    *child = Value::String(new.to_string());
                    1
                } else {
                    0
                };
                direct + replace_cwd_values(child, old, new)
            })
            .sum(),
        Value::Array(items) => items.iter_mut().map(|child| replace_cwd_values(child, old, new)).sum(),
        _ => 0,
    }
}

fn escape_pointer(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}
```

- [x] **Step 4: Run tests and verify they pass**

Run: `cargo test --test jsonl_surface`

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add src/lib.rs src/surfaces/mod.rs src/surfaces/jsonl.rs tests/jsonl_surface.rs
git commit -m "feat: update JSONL cwd references"
```

## Task 6: Scan and Update SQLite `threads.cwd`

**Files:**
- Create: `src/surfaces/sqlite_threads.rs`
- Modify: `src/surfaces/mod.rs`
- Test: `tests/sqlite_threads_surface.rs`

- [x] **Step 1: Write failing SQLite tests**

```rust
// tests/sqlite_threads_surface.rs
use codex_project_mover::surfaces::sqlite_threads::{scan_threads_db, update_threads_db};
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn scans_and_updates_only_threads_cwd_equal_to_old_path() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("state_test.sqlite");
    let conn = Connection::open(&db).unwrap();
    conn.execute("CREATE TABLE threads (id TEXT PRIMARY KEY, cwd TEXT)", []).unwrap();
    conn.execute("INSERT INTO threads (id, cwd) VALUES ('a', '/old/project')", []).unwrap();
    conn.execute("INSERT INTO threads (id, cwd) VALUES ('b', '/old/project/subdir')", []).unwrap();

    let matches = scan_threads_db(&db, "/old/project", "/new/project").unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].location, "threads.id=a cwd");

    let changed = update_threads_db(&db, "/old/project", "/new/project").unwrap();
    assert_eq!(changed, 1);

    let updated: String = conn
        .query_row("SELECT cwd FROM threads WHERE id = 'a'", [], |row| row.get(0))
        .unwrap();
    let untouched: String = conn
        .query_row("SELECT cwd FROM threads WHERE id = 'b'", [], |row| row.get(0))
        .unwrap();
    assert_eq!(updated, "/new/project");
    assert_eq!(untouched, "/old/project/subdir");
}
```

- [x] **Step 2: Run tests and verify they fail**

Run: `cargo test --test sqlite_threads_surface`

Expected: FAIL because SQLite thread surface code does not exist.

- [x] **Step 3: Implement SQLite `threads.cwd` scan and update**

```rust
// src/surfaces/mod.rs
pub mod jsonl;
pub mod sqlite_threads;
```

```rust
// src/surfaces/sqlite_threads.rs
use std::path::Path;

use anyhow::Result;
use rusqlite::{params, Connection};

use crate::model::{ReferenceMatch, SurfaceKind};

pub fn scan_threads_db(path: &Path, old: &str, new: &str) -> Result<Vec<ReferenceMatch>> {
    let conn = Connection::open(path)?;
    if !has_table_column(&conn, "threads", "cwd")? {
        return Ok(Vec::new());
    }

    let mut stmt = conn.prepare("SELECT id, cwd FROM threads WHERE cwd = ?1")?;
    let rows = stmt.query_map(params![old], |row| {
        let id: String = row.get(0)?;
        Ok(ReferenceMatch {
            surface: SurfaceKind::SqliteThreadsCwd,
            file: path.to_path_buf(),
            location: format!("threads.id={} cwd", id),
            old_value: old.to_string(),
            new_value: new.to_string(),
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn update_threads_db(path: &Path, old: &str, new: &str) -> Result<usize> {
    let conn = Connection::open(path)?;
    if !has_table_column(&conn, "threads", "cwd")? {
        return Ok(0);
    }

    let changed = conn.execute("UPDATE threads SET cwd = ?1 WHERE cwd = ?2", params![new, old])?;
    Ok(changed)
}

fn has_table_column(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let table_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        params![table],
        |row| row.get(0),
    )?;
    if table_count == 0 {
        return Ok(false);
    }

    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for name in columns {
        if name? == column {
            return Ok(true);
        }
    }
    Ok(false)
}
```

- [x] **Step 4: Run tests and verify they pass**

Run: `cargo test --test sqlite_threads_surface`

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add src/surfaces/mod.rs src/surfaces/sqlite_threads.rs tests/sqlite_threads_surface.rs
git commit -m "feat: update SQLite thread cwd references"
```

## Task 7: Scan and Update Global State JSON Exact Strings and Path Keys

**Files:**
- Create: `src/surfaces/global_state.rs`
- Modify: `src/surfaces/mod.rs`
- Test: `tests/global_state_surface.rs`

- [x] **Step 1: Write failing global state tests**

```rust
// tests/global_state_surface.rs
use std::fs;

use codex_project_mover::surfaces::global_state::{scan_global_state, update_global_state};
use tempfile::tempdir;

#[test]
fn scans_and_updates_only_exact_string_values_and_exact_path_keys() {
    let temp = tempdir().unwrap();
    let file = temp.path().join(".codex-global-state.json");
    fs::write(
        &file,
        r#"{
          "workspaceRoots":["/old/project","/old/project/subdir"],
          "electron-workspace-root-labels":{"/old/project":"Old Project","/old/project/subdir":"Nested"},
          "nested":{"cwd":"/old/project"},
          "message":"do not edit /old/project inside text"
        }"#,
    )
    .unwrap();

    let matches = scan_global_state(&file, "/old/project", "/new/project").unwrap();
    let mut locations = matches.iter().map(|m| m.location.as_str()).collect::<Vec<_>>();
    locations.sort();
    assert_eq!(matches.len(), 3);
    assert_eq!(
        locations,
        vec![
            "/electron-workspace-root-labels/~1old~1project",
            "/nested/cwd",
            "/workspaceRoots/0",
        ]
    );

    let changed = update_global_state(&file, "/old/project", "/new/project").unwrap();
    let updated = fs::read_to_string(&file).unwrap();

    assert_eq!(changed, 3);
    assert!(updated.contains(r#""/new/project""#));
    assert!(updated.contains(r#""/new/project": "Old Project""#));
    assert!(updated.contains(r#""/old/project/subdir""#));
    assert!(updated.contains(r#""do not edit /old/project inside text""#));
}
```

- [x] **Step 2: Run tests and verify they fail**

Run: `cargo test --test global_state_surface`

Expected: FAIL because global state code does not exist.

- [x] **Step 3: Implement exact JSON string and object-key replacement**

```rust
// src/surfaces/mod.rs
pub mod global_state;
pub mod jsonl;
pub mod sqlite_threads;
```

```rust
// src/surfaces/global_state.rs
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::model::{ReferenceMatch, SurfaceKind};

pub fn scan_global_state(path: &Path, old: &str, new: &str) -> Result<Vec<ReferenceMatch>> {
    let value = read_json(path)?;
    let mut matches = Vec::new();
    collect_exact_string_and_key_matches(&value, old, new, path, String::new(), &mut matches);
    Ok(matches)
}

pub fn update_global_state(path: &Path, old: &str, new: &str) -> Result<usize> {
    let mut value = read_json(path)?;
    let changed = replace_exact_string_values_and_keys(&mut value, old, new);
    fs::write(path, serde_json::to_string_pretty(&value)? + "\n")
        .with_context(|| format!("write global state JSON {}", path.display()))?;
    Ok(changed)
}

fn read_json(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("read global state JSON {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("parse global state JSON {}", path.display()))
}

fn collect_exact_string_and_key_matches(
    value: &Value,
    old: &str,
    new: &str,
    file: &Path,
    pointer: String,
    matches: &mut Vec<ReferenceMatch>,
) {
    match value {
        Value::String(current) if current == old => matches.push(ReferenceMatch {
            surface: SurfaceKind::GlobalStateJson,
            file: file.to_path_buf(),
            location: if pointer.is_empty() { "/".to_string() } else { pointer },
            old_value: old.to_string(),
            new_value: new.to_string(),
        }),
        Value::Object(map) => {
            for (key, child) in map {
                let child_pointer = format!("{}/{}", pointer, escape_pointer(key));
                if key == old {
                    matches.push(ReferenceMatch {
                        surface: SurfaceKind::GlobalStateJson,
                        file: file.to_path_buf(),
                        location: child_pointer.clone(),
                        old_value: old.to_string(),
                        new_value: new.to_string(),
                    });
                }
                collect_exact_string_and_key_matches(
                    child,
                    old,
                    new,
                    file,
                    child_pointer,
                    matches,
                );
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect_exact_string_and_key_matches(child, old, new, file, format!("{}/{}", pointer, index), matches);
            }
        }
        _ => {}
    }
}

fn replace_exact_string_values_and_keys(value: &mut Value, old: &str, new: &str) -> usize {
    match value {
        Value::String(current) if current == old => {
            *current = new.to_string();
            1
        }
        Value::Object(map) => {
            let mut changed = map
                .values_mut()
                .map(|child| replace_exact_string_values_and_keys(child, old, new))
                .sum::<usize>();
            if let Some(item) = map.remove(old) {
                map.insert(new.to_string(), item);
                changed += 1;
            }
            changed
        }
        Value::Array(items) => items
            .iter_mut()
            .map(|child| replace_exact_string_values_and_keys(child, old, new))
            .sum(),
        _ => 0,
    }
}

fn escape_pointer(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}
```

- [x] **Step 4: Run tests and verify they pass**

Run: `cargo test --test global_state_surface`

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add src/surfaces/mod.rs src/surfaces/global_state.rs tests/global_state_surface.rs
git commit -m "feat: update global state exact path references"
```

## Task 8: Scan and Update `config.toml`

**Files:**
- Create: `src/surfaces/config_toml.rs`
- Modify: `src/surfaces/mod.rs`
- Test: `tests/config_toml_surface.rs`

- [x] **Step 1: Write failing TOML tests**

```rust
// tests/config_toml_surface.rs
use std::fs;

use codex_project_mover::surfaces::config_toml::{scan_config_toml, update_config_toml};
use tempfile::tempdir;

#[test]
fn updates_exact_project_table_key_per_path_key_and_exact_string_values() {
    let temp = tempdir().unwrap();
    let file = temp.path().join("config.toml");
    fs::write(
        &file,
        r#"
[projects."/old/project"]
trust_level = "trusted"

[desktop]
open_target = "/old/project"
message = "do not edit /old/project inside text"

[desktop.open-in-target-preferences.perPath]
"/old/project" = "vscode"
"/old/project/subdir" = "iterm2"
"#,
    )
    .unwrap();

    let matches = scan_config_toml(&file, "/old/project", "/new/project").unwrap();
    assert_eq!(matches.len(), 3);

    let changed = update_config_toml(&file, "/old/project", "/new/project").unwrap();
    let updated = fs::read_to_string(&file).unwrap();

    assert_eq!(changed, 3);
    assert!(updated.contains(r#"[projects."/new/project"]"#));
    assert!(updated.contains(r#"open_target = "/new/project""#));
    assert!(updated.contains(r#""/new/project" = "vscode""#));
    assert!(updated.contains(r#"message = "do not edit /old/project inside text""#));
}
```

- [x] **Step 2: Run tests and verify they fail**

Run: `cargo test --test config_toml_surface`

Expected: FAIL because TOML surface code does not exist.

- [x] **Step 3: Implement TOML scan and update**

```rust
// src/surfaces/mod.rs
pub mod config_toml;
pub mod global_state;
pub mod jsonl;
pub mod sqlite_threads;
```

```rust
// src/surfaces/config_toml.rs
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use toml_edit::{DocumentMut, Item, Value};

use crate::model::{ReferenceMatch, SurfaceKind};

pub fn scan_config_toml(path: &Path, old: &str, new: &str) -> Result<Vec<ReferenceMatch>> {
    let document = read_document(path)?;
    let mut matches = Vec::new();
    collect_toml_matches(document.as_item(), old, new, path, String::new(), &mut matches);
    collect_project_table_key_match(&document, old, new, path, &mut matches);
    collect_per_path_key_match(&document, old, new, path, &mut matches);
    Ok(matches)
}

pub fn update_config_toml(path: &Path, old: &str, new: &str) -> Result<usize> {
    let mut document = read_document(path)?;
    let mut changed = replace_toml_values(document.as_item_mut(), old, new);
    changed += rename_project_table_key(&mut document, old, new);
    changed += rename_per_path_key(&mut document, old, new);
    fs::write(path, document.to_string()).with_context(|| format!("write config TOML {}", path.display()))?;
    Ok(changed)
}

fn read_document(path: &Path) -> Result<DocumentMut> {
    let content = fs::read_to_string(path).with_context(|| format!("read config TOML {}", path.display()))?;
    content.parse::<DocumentMut>().with_context(|| format!("parse config TOML {}", path.display()))
}

fn collect_toml_matches(
    item: &Item,
    old: &str,
    new: &str,
    file: &Path,
    location: String,
    matches: &mut Vec<ReferenceMatch>,
) {
    match item {
        Item::Value(value) if value.as_str() == Some(old) => matches.push(ReferenceMatch {
            surface: SurfaceKind::ConfigToml,
            file: file.to_path_buf(),
            location,
            old_value: old.to_string(),
            new_value: new.to_string(),
        }),
        Item::Table(table) => {
            for (key, child) in table.iter() {
                collect_toml_matches(child, old, new, file, format!("{}/{}", location, key), matches);
            }
        }
        _ => {}
    }
}

fn replace_toml_values(item: &mut Item, old: &str, new: &str) -> usize {
    match item {
        Item::Value(value) if value.as_str() == Some(old) => {
            *value = Value::from(new);
            1
        }
        Item::Table(table) => table.iter_mut().map(|(_, child)| replace_toml_values(child, old, new)).sum(),
        _ => 0,
    }
}

fn collect_project_table_key_match(
    document: &DocumentMut,
    old: &str,
    new: &str,
    file: &Path,
    matches: &mut Vec<ReferenceMatch>,
) {
    if document["projects"].as_table().and_then(|projects| projects.get(old)).is_some() {
        matches.push(ReferenceMatch {
            surface: SurfaceKind::ConfigToml,
            file: file.to_path_buf(),
            location: format!("/projects/{}", old),
            old_value: old.to_string(),
            new_value: new.to_string(),
        });
    }
}

fn collect_per_path_key_match(
    document: &DocumentMut,
    old: &str,
    new: &str,
    file: &Path,
    matches: &mut Vec<ReferenceMatch>,
) {
    if document["desktop"]["open-in-target-preferences"]["perPath"]
        .as_table()
        .and_then(|per_path| per_path.get(old))
        .is_some()
    {
        matches.push(ReferenceMatch {
            surface: SurfaceKind::ConfigToml,
            file: file.to_path_buf(),
            location: format!("/desktop/open-in-target-preferences/perPath/{}", old),
            old_value: old.to_string(),
            new_value: new.to_string(),
        });
    }
}

fn rename_project_table_key(document: &mut DocumentMut, old: &str, new: &str) -> usize {
    let Some(projects) = document["projects"].as_table_mut() else {
        return 0;
    };
    let Some(old_item) = projects.remove(old) else {
        return 0;
    };
    projects.insert(new, old_item);
    1
}

fn rename_per_path_key(document: &mut DocumentMut, old: &str, new: &str) -> usize {
    let Some(per_path) = document["desktop"]["open-in-target-preferences"]["perPath"].as_table_mut() else {
        return 0;
    };
    let Some(old_item) = per_path.remove(old) else {
        return 0;
    };
    per_path.insert(new, old_item);
    1
}
```

- [x] **Step 4: Run tests and verify they pass**

Run: `cargo test --test config_toml_surface`

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add src/surfaces/mod.rs src/surfaces/config_toml.rs tests/config_toml_surface.rs
git commit -m "feat: update Codex config TOML paths"
```

## Task 9: Scan and Update Automation DB `cwds`

**Files:**
- Create: `src/surfaces/automation_db.rs`
- Modify: `src/surfaces/mod.rs`
- Test: `tests/automation_db_surface.rs`

- [x] **Step 1: Write failing automation DB tests**

```rust
// tests/automation_db_surface.rs
use codex_project_mover::surfaces::automation_db::{scan_automation_db, update_automation_db};
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn updates_cwds_json_array_elements_equal_to_old_path() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("codex-dev.db");
    let conn = Connection::open(&db).unwrap();
    conn.execute("CREATE TABLE automations (id TEXT PRIMARY KEY, cwds TEXT)", []).unwrap();
    conn.execute(
        r#"INSERT INTO automations (id, cwds) VALUES ('a', '["/old/project","/old/project/subdir"]')"#,
        [],
    )
    .unwrap();

    let matches = scan_automation_db(&db, "/old/project", "/new/project").unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].location, "automations.id=a cwds[0]");

    let changed = update_automation_db(&db, "/old/project", "/new/project").unwrap();
    assert_eq!(changed, 1);

    let updated: String = conn
        .query_row("SELECT cwds FROM automations WHERE id = 'a'", [], |row| row.get(0))
        .unwrap();
    assert_eq!(updated, r#"["/new/project","/old/project/subdir"]"#);
}
```

- [x] **Step 2: Run tests and verify they fail**

Run: `cargo test --test automation_db_surface`

Expected: FAIL because automation DB code does not exist.

- [x] **Step 3: Implement `cwds` column scan and update**

```rust
// src/surfaces/mod.rs
pub mod automation_db;
pub mod config_toml;
pub mod global_state;
pub mod jsonl;
pub mod sqlite_threads;
```

```rust
// src/surfaces/automation_db.rs
use std::path::Path;

use anyhow::Result;
use rusqlite::{params, Connection};
use serde_json::Value;

use crate::model::{ReferenceMatch, SurfaceKind};

pub fn scan_automation_db(path: &Path, old: &str, new: &str) -> Result<Vec<ReferenceMatch>> {
    let conn = Connection::open(path)?;
    let mut matches = Vec::new();
    for table in tables_with_cwds(&conn)? {
        let sql = format!("SELECT id, cwds FROM {} WHERE cwds IS NOT NULL", quote_identifier(&table));
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?;
        for row in rows {
            let (id, cwds) = row?;
            collect_cwds_matches(path, &table, &id, &cwds, old, new, &mut matches);
        }
    }
    Ok(matches)
}

pub fn update_automation_db(path: &Path, old: &str, new: &str) -> Result<usize> {
    let conn = Connection::open(path)?;
    let mut changed = 0;
    for table in tables_with_cwds(&conn)? {
        let select_sql = format!("SELECT id, cwds FROM {} WHERE cwds IS NOT NULL", quote_identifier(&table));
        let rows = {
            let mut stmt = conn.prepare(&select_sql)?;
            stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?
                .collect::<Result<Vec<_>, _>>()?
        };
        for (id, cwds) in rows {
            if let Some(updated) = replace_cwds_value(&cwds, old, new) {
                let update_sql = format!("UPDATE {} SET cwds = ?1 WHERE id = ?2", quote_identifier(&table));
                conn.execute(&update_sql, params![updated, id])?;
                changed += 1;
            }
        }
    }
    Ok(changed)
}

fn collect_cwds_matches(
    file: &Path,
    table: &str,
    id: &str,
    cwds: &str,
    old: &str,
    new: &str,
    matches: &mut Vec<ReferenceMatch>,
) {
    if cwds == old {
        matches.push(match_for(file, table, id, "cwds", old, new));
        return;
    }

    let Ok(Value::Array(items)) = serde_json::from_str::<Value>(cwds) else {
        return;
    };
    for (index, item) in items.iter().enumerate() {
        if item.as_str() == Some(old) {
            matches.push(match_for(file, table, id, &format!("cwds[{}]", index), old, new));
        }
    }
}

fn replace_cwds_value(cwds: &str, old: &str, new: &str) -> Option<String> {
    if cwds == old {
        return Some(new.to_string());
    }
    let Ok(Value::Array(mut items)) = serde_json::from_str::<Value>(cwds) else {
        return None;
    };
    let mut changed = false;
    for item in &mut items {
        if item.as_str() == Some(old) {
            *item = Value::String(new.to_string());
            changed = true;
        }
    }
    changed.then(|| serde_json::to_string(&items).unwrap())
}

fn tables_with_cwds(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type = 'table'")?;
    let names = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let mut tables = Vec::new();
    for name in names {
        let name = name?;
        let pragma = format!("PRAGMA table_info({})", quote_identifier(&name));
        let mut column_stmt = conn.prepare(&pragma)?;
        let columns = column_stmt.query_map([], |row| row.get::<_, String>(1))?;
        for column in columns {
            if column? == "cwds" {
                tables.push(name.clone());
                break;
            }
        }
    }
    Ok(tables)
}

fn match_for(file: &Path, table: &str, id: &str, field: &str, old: &str, new: &str) -> ReferenceMatch {
    ReferenceMatch {
        surface: SurfaceKind::AutomationDbCwds,
        file: file.to_path_buf(),
        location: format!("{}.id={} {}", table, id, field),
        old_value: old.to_string(),
        new_value: new.to_string(),
    }
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}
```

- [x] **Step 4: Run tests and verify they pass**

Run: `cargo test --test automation_db_surface`

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add src/surfaces/mod.rs src/surfaces/automation_db.rs tests/automation_db_surface.rs
git commit -m "feat: update automation cwd metadata"
```

## Task 10: Build Metadata Backup and Rollback Manifest

**Files:**
- Create: `src/backup.rs`
- Modify: `src/lib.rs`
- Test: `tests/backup.rs`

- [x] **Step 1: Write failing backup tests**

```rust
// tests/backup.rs
use std::fs;

use codex_project_mover::backup::{create_metadata_backup, restore_metadata_backup};
use tempfile::tempdir;

#[test]
fn backs_up_and_restores_metadata_files() {
    let temp = tempdir().unwrap();
    let codex_home = temp.path().join(".codex");
    let backups_root = codex_home.join("codex-project-mover-backups");
    fs::create_dir_all(&codex_home).unwrap();
    let metadata = codex_home.join("config.toml");
    fs::write(&metadata, "before").unwrap();

    let backup = create_metadata_backup(
        &backups_root,
        "/old/project",
        "/new/project",
        None,
        &[metadata.clone()],
    )
    .unwrap();

    fs::write(&metadata, "after").unwrap();
    restore_metadata_backup(&backup.manifest_path).unwrap();

    assert_eq!(fs::read_to_string(&metadata).unwrap(), "before");
    assert!(backup.manifest_path.exists());
}
```

- [x] **Step 2: Run tests and verify they fail**

Run: `cargo test --test backup`

Expected: FAIL because backup code does not exist.

- [x] **Step 3: Implement metadata backup and restore**

```rust
// src/lib.rs
pub mod backup;
pub mod cli;
pub mod discovery;
pub mod error;
pub mod model;
pub mod pathing;
pub mod process_guard;
pub mod surfaces;
```

```rust
// src/backup.rs
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct BackupResult {
    pub backup_dir: PathBuf,
    pub manifest_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub version: u32,
    pub created_at_utc: String,
    pub old_path: String,
    pub new_path: String,
    pub created_new_project_path: Option<PathBuf>,
    pub entries: Vec<BackupEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub original_path: PathBuf,
    pub backup_path: PathBuf,
    pub sha256_before: String,
    pub byte_len: u64,
}

pub fn create_metadata_backup(
    backups_root: &Path,
    old_path: &str,
    new_path: &str,
    created_new_project_path: Option<PathBuf>,
    files: &[PathBuf],
) -> Result<BackupResult> {
    let id = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let backup_dir = backups_root.join(id);
    let files_dir = backup_dir.join("files");
    fs::create_dir_all(&files_dir).with_context(|| format!("create backup dir {}", files_dir.display()))?;

    let mut entries = Vec::new();
    for (index, original_path) in files.iter().enumerate() {
        let bytes = fs::read(original_path)
            .with_context(|| format!("read metadata file for backup {}", original_path.display()))?;
        let backup_path = files_dir.join(format!("{:04}.bak", index));
        fs::write(&backup_path, &bytes)
            .with_context(|| format!("write metadata backup {}", backup_path.display()))?;
        entries.push(BackupEntry {
            original_path: original_path.clone(),
            backup_path,
            sha256_before: sha256_hex(&bytes),
            byte_len: bytes.len() as u64,
        });
    }

    let manifest = BackupManifest {
        version: 1,
        created_at_utc: Utc::now().to_rfc3339(),
        old_path: old_path.to_string(),
        new_path: new_path.to_string(),
        created_new_project_path,
        entries,
    };
    let manifest_path = backup_dir.join("manifest.json");
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)? + "\n")
        .with_context(|| format!("write backup manifest {}", manifest_path.display()))?;

    Ok(BackupResult {
        backup_dir,
        manifest_path,
    })
}

pub fn restore_metadata_backup(manifest_path: &Path) -> Result<BackupManifest> {
    let manifest: BackupManifest = serde_json::from_slice(
        &fs::read(manifest_path).with_context(|| format!("read backup manifest {}", manifest_path.display()))?,
    )
    .with_context(|| format!("parse backup manifest {}", manifest_path.display()))?;

    for entry in &manifest.entries {
        let bytes = fs::read(&entry.backup_path)
            .with_context(|| format!("read backup file {}", entry.backup_path.display()))?;
        anyhow::ensure!(
            sha256_hex(&bytes) == entry.sha256_before,
            "backup checksum mismatch for {}",
            entry.backup_path.display()
        );
        fs::write(&entry.original_path, bytes)
            .with_context(|| format!("restore metadata file {}", entry.original_path.display()))?;
    }
    Ok(manifest)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
```

- [x] **Step 4: Run tests and verify they pass**

Run: `cargo test --test backup`

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add src/lib.rs src/backup.rs tests/backup.rs
git commit -m "feat: add metadata backup manifest"
```

## Task 11: Copy, Verify, and Trash Project Folders

**Files:**
- Create: `src/project_copy.rs`
- Create: `src/trash.rs`
- Modify: `src/lib.rs`
- Test: `tests/project_copy.rs`

- [x] **Step 1: Write failing project copy tests**

```rust
// tests/project_copy.rs
use std::fs;

use codex_project_mover::project_copy::{copy_project_tree, verify_project_tree};
use tempfile::tempdir;

#[test]
fn copies_and_verifies_project_tree() {
    let temp = tempdir().unwrap();
    let old = temp.path().join("old-project");
    let new = temp.path().join("nested/new-project");
    fs::create_dir_all(old.join("src")).unwrap();
    fs::write(old.join("src/main.rs"), "fn main() {}\n").unwrap();

    copy_project_tree(&old, &new).unwrap();
    verify_project_tree(&old, &new).unwrap();

    assert_eq!(fs::read_to_string(new.join("src/main.rs")).unwrap(), "fn main() {}\n");
}
```

- [x] **Step 2: Run tests and verify they fail**

Run: `cargo test --test project_copy`

Expected: FAIL because project copy code does not exist.

- [x] **Step 3: Implement recursive copy and tree verification**

```rust
// src/lib.rs
pub mod backup;
pub mod cli;
pub mod discovery;
pub mod error;
pub mod model;
pub mod pathing;
pub mod process_guard;
pub mod project_copy;
pub mod surfaces;
pub mod trash;
```

```rust
// src/project_copy.rs
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

pub fn copy_project_tree(old: &Path, new: &Path) -> Result<()> {
    if let Some(parent) = new.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create new project parent {}", parent.display()))?;
    }

    for entry in WalkDir::new(old).follow_links(false) {
        let entry = entry?;
        let source = entry.path();
        let relative = source.strip_prefix(old)?;
        let target = new.join(relative);
        let file_type = entry.file_type();

        if relative.as_os_str().is_empty() {
            fs::create_dir_all(&target)?;
        } else if file_type.is_dir() {
            fs::create_dir_all(&target)?;
        } else if file_type.is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(source, &target)
                .with_context(|| format!("copy {} to {}", source.display(), target.display()))?;
            fs::set_permissions(&target, fs::metadata(source)?.permissions())?;
        } else if file_type.is_symlink() {
            let link_target = fs::read_link(source)?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(link_target, &target)?;
        }
    }

    Ok(())
}

pub fn verify_project_tree(old: &Path, new: &Path) -> Result<()> {
    let old_entries = tree_fingerprints(old)?;
    let new_entries = tree_fingerprints(new)?;
    anyhow::ensure!(old_entries == new_entries, "copied project tree does not match source");
    Ok(())
}

fn tree_fingerprints(root: &Path) -> Result<Vec<(PathBuf, String)>> {
    let mut entries = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(root)?.to_path_buf();
        if relative.as_os_str().is_empty() {
            continue;
        }

        let kind_hash = if entry.file_type().is_dir() {
            "dir".to_string()
        } else if entry.file_type().is_symlink() {
            format!("symlink:{}", fs::read_link(path)?.display())
        } else {
            format!("file:{}", sha256_file(path)?)
        };
        entries.push((relative, kind_hash));
    }
    entries.sort();
    Ok(entries)
}

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
```

```rust
// src/trash.rs
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

pub fn move_to_trash(path: &Path) -> Result<()> {
    if let Ok(test_trash_dir) = std::env::var("CODEX_PROJECT_MOVER_TEST_TRASH_DIR") {
        let file_name = path
            .file_name()
            .context("old project path must have a final path component")?;
        let target = Path::new(&test_trash_dir).join(file_name);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(path, &target)
            .with_context(|| format!("move old project folder to test trash: {}", target.display()))?;
        return Ok(());
    }

    trash::delete(path).with_context(|| format!("move old project folder to Trash: {}", path.display()))
}
```

- [x] **Step 4: Run tests and verify they pass**

Run: `cargo test --test project_copy`

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add src/lib.rs src/project_copy.rs src/trash.rs tests/project_copy.rs
git commit -m "feat: copy and verify project folders"
```

## Task 12: Build Shared Scanner Across All Supported Surfaces

**Files:**
- Create: `src/scanner.rs`
- Modify: `src/lib.rs`
- Test: `tests/scanner.rs`

- [x] **Step 1: Write failing scanner integration test**

```rust
// tests/scanner.rs
use std::fs;

use codex_project_mover::scanner::scan_codex_home;
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn scanner_combines_matches_from_supported_surfaces() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::write(home.join("sessions/thread.jsonl"), r#"{"cwd":"/old/project"}"#).unwrap();
    fs::write(home.join(".codex-global-state.json"), r#"{"roots":["/old/project"]}"#).unwrap();
    fs::write(home.join("config.toml"), "[desktop]\nopen_target = \"/old/project\"\n").unwrap();

    let db = home.join("state_main.sqlite");
    let conn = Connection::open(&db).unwrap();
    conn.execute("CREATE TABLE threads (id TEXT PRIMARY KEY, cwd TEXT)", []).unwrap();
    conn.execute("INSERT INTO threads (id, cwd) VALUES ('a', '/old/project')", []).unwrap();

    let report = scan_codex_home(&home, "/old/project", "/new/project").unwrap();

    assert_eq!(report.old_reference_count(), 4);
}
```

- [x] **Step 2: Run test and verify it fails**

Run: `cargo test --test scanner`

Expected: FAIL because `src/scanner.rs` does not exist.

- [x] **Step 3: Implement shared scanner**

```rust
// src/lib.rs
pub mod backup;
pub mod cli;
pub mod discovery;
pub mod error;
pub mod model;
pub mod pathing;
pub mod process_guard;
pub mod project_copy;
pub mod scanner;
pub mod surfaces;
pub mod trash;
```

```rust
// src/scanner.rs
use std::path::Path;

use anyhow::Result;

use crate::discovery::discover_state;
use crate::model::ScanReport;
use crate::surfaces::{
    automation_db::scan_automation_db,
    config_toml::scan_config_toml,
    global_state::scan_global_state,
    jsonl::scan_jsonl_file,
    sqlite_threads::scan_threads_db,
};

pub fn scan_codex_home(codex_home: &Path, old: &str, new: &str) -> Result<ScanReport> {
    let state = discover_state(codex_home)?;
    let mut report = ScanReport::default();

    for file in state.jsonl_files {
        report.matches.extend(scan_jsonl_file(&file, old, new)?);
    }

    for db in state.sqlite_state_dbs {
        report.matches.extend(scan_threads_db(&db, old, new)?);
    }

    if let Some(file) = state.global_state_json {
        report.matches.extend(scan_global_state(&file, old, new)?);
    }

    if let Some(file) = state.config_toml {
        report.matches.extend(scan_config_toml(&file, old, new)?);
    }

    if let Some(db) = state.automation_db {
        report.matches.extend(scan_automation_db(&db, old, new)?);
    }

    report.matches.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then_with(|| a.location.cmp(&b.location))
    });
    Ok(report)
}
```

- [x] **Step 4: Run scanner test and full surface tests**

Run: `cargo test --test scanner --test jsonl_surface --test sqlite_threads_surface --test global_state_surface --test config_toml_surface --test automation_db_surface`

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add src/lib.rs src/scanner.rs tests/scanner.rs
git commit -m "feat: scan all supported Codex metadata"
```

## Task 13: Implement `plan` and `verify` Commands

**Files:**
- Create: `src/commands/mod.rs`
- Create: `src/commands/plan.rs`
- Create: `src/commands/verify.rs`
- Modify: `src/lib.rs`
- Test: `tests/cli.rs`

- [x] **Step 1: Extend CLI tests for `plan` and `verify`**

```rust
// tests/cli.rs
use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

#[test]
fn plan_reports_supported_references() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::write(home.join("sessions/thread.jsonl"), r#"{"cwd":"/old/project"}"#).unwrap();

    Command::cargo_bin("codex-project-mover")
        .unwrap()
        .args([
            "plan",
            "--old",
            "/old/project",
            "--new",
            "/new/project",
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("1 old-path reference"))
        .stdout(contains("JsonlCwd"));
}

#[test]
fn verify_fails_when_old_references_remain() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::write(home.join("sessions/thread.jsonl"), r#"{"cwd":"/old/project"}"#).unwrap();

    Command::cargo_bin("codex-project-mover")
        .unwrap()
        .args([
            "verify",
            "--old",
            "/old/project",
            "--new",
            "/new/project",
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(contains("verification failed: 1 old-path reference remains"));
}

#[test]
fn verify_passes_when_old_references_are_gone_and_new_references_exist() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::write(home.join("sessions/thread.jsonl"), r#"{"cwd":"/new/project"}"#).unwrap();

    Command::cargo_bin("codex-project-mover")
        .unwrap()
        .args([
            "verify",
            "--old",
            "/old/project",
            "--new",
            "/new/project",
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("verification passed"))
        .stdout(contains("1 new-path reference"));
}
```

- [x] **Step 2: Run CLI tests and verify they fail**

Run: `cargo test --test cli plan_reports_supported_references verify_fails_when_old_references_remain verify_passes_when_old_references_are_gone_and_new_references_exist`

Expected: FAIL because commands still print stub output.

- [x] **Step 3: Implement command routing and read-only commands**

```rust
// src/lib.rs
pub mod backup;
pub mod cli;
pub mod commands;
pub mod discovery;
pub mod error;
pub mod model;
pub mod pathing;
pub mod process_guard;
pub mod project_copy;
pub mod scanner;
pub mod surfaces;
pub mod trash;

pub fn run() -> anyhow::Result<()> {
    let cli = <cli::Cli as clap::Parser>::parse();
    match cli.command {
        cli::Command::Plan(args) => commands::plan::run(args),
        cli::Command::Apply(args) => commands::apply::run(args),
        cli::Command::Verify(args) => commands::verify::run(args),
        cli::Command::Rollback(args) => commands::rollback::run(args),
    }
}
```

```rust
// src/commands/mod.rs
pub mod apply;
pub mod plan;
pub mod rollback;
pub mod verify;
```

```rust
// src/commands/plan.rs
use anyhow::Result;

use crate::cli::MoveArgs;
use crate::pathing::{codex_home_from_arg, normalize_project_path};
use crate::process_guard::assert_no_codex_processes;
use crate::scanner::scan_codex_home;

pub fn run(args: MoveArgs) -> Result<()> {
    assert_no_codex_processes()?;
    let old = normalize_project_path(args.old)?;
    let new = normalize_project_path(args.new)?;
    let codex_home = codex_home_from_arg(args.codex_home)?;
    let report = scan_codex_home(&codex_home, &old.to_string_lossy(), &new.to_string_lossy())?;

    println!("Plan: {} -> {}", old.display(), new.display());
    println!("{} old-path reference(s) found", report.old_reference_count());
    for reference in report.matches {
        println!(
            "- {:?}: {} {}",
            reference.surface,
            reference.file.display(),
            reference.location
        );
    }
    Ok(())
}
```

```rust
// src/commands/verify.rs
use anyhow::{bail, Result};

use crate::cli::MoveArgs;
use crate::pathing::{codex_home_from_arg, normalize_project_path};
use crate::process_guard::assert_no_codex_processes;
use crate::scanner::scan_codex_home;

pub fn run(args: MoveArgs) -> Result<()> {
    assert_no_codex_processes()?;
    let old = normalize_project_path(args.old)?;
    let new = normalize_project_path(args.new)?;
    let codex_home = codex_home_from_arg(args.codex_home)?;
    let old_str = old.to_string_lossy().to_string();
    let new_str = new.to_string_lossy().to_string();
    let old_report = scan_codex_home(&codex_home, &old_str, &new_str)?;
    let new_report = scan_codex_home(&codex_home, &new_str, &old_str)?;

    if old_report.old_reference_count() > 0 {
        bail!(
            "verification failed: {} old-path reference remains",
            old_report.old_reference_count()
        );
    }

    if new_report.old_reference_count() == 0 {
        bail!("verification failed: no supported new-path references found");
    }

    println!(
        "verification passed: no supported old-path references remain; {} new-path reference(s) found",
        new_report.old_reference_count()
    );
    Ok(())
}
```

```rust
// src/commands/apply.rs
use anyhow::Result;

use crate::cli::ApplyArgs;

pub fn run(_args: ApplyArgs) -> Result<()> {
    anyhow::bail!("apply command is introduced by Task 15")
}
```

```rust
// src/commands/rollback.rs
use anyhow::Result;

use crate::cli::RollbackArgs;

pub fn run(_args: RollbackArgs) -> Result<()> {
    anyhow::bail!("rollback command is introduced by Task 16")
}
```

- [x] **Step 4: Run CLI tests and verify they pass**

Run: `cargo test --test cli plan_reports_supported_references verify_fails_when_old_references_remain verify_passes_when_old_references_are_gone_and_new_references_exist`

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add src/lib.rs src/commands/mod.rs src/commands/plan.rs src/commands/verify.rs src/commands/apply.rs src/commands/rollback.rs tests/cli.rs
git commit -m "feat: add plan and verify commands"
```

## Task 14: Implement Metadata Update Orchestration

**Files:**
- Create: `src/updater.rs`
- Modify: `src/lib.rs`
- Test: `tests/updater.rs`

- [x] **Step 1: Write failing updater test**

```rust
// tests/updater.rs
use std::fs;

use codex_project_mover::scanner::scan_codex_home;
use codex_project_mover::updater::update_codex_home;
use tempfile::tempdir;

#[test]
fn updates_all_supported_metadata_files() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::write(home.join("sessions/thread.jsonl"), r#"{"cwd":"/old/project"}"#).unwrap();
    fs::write(home.join(".codex-global-state.json"), r#"{"roots":["/old/project"]}"#).unwrap();
    fs::write(home.join("config.toml"), "[desktop]\nopen_target = \"/old/project\"\n").unwrap();

    let changed = update_codex_home(&home, "/old/project", "/new/project").unwrap();
    let remaining = scan_codex_home(&home, "/old/project", "/new/project").unwrap();

    assert_eq!(changed, 3);
    assert_eq!(remaining.old_reference_count(), 0);
}
```

- [x] **Step 2: Run test and verify it fails**

Run: `cargo test --test updater`

Expected: FAIL because updater code does not exist.

- [x] **Step 3: Implement shared metadata updater**

```rust
// src/lib.rs
pub mod backup;
pub mod cli;
pub mod commands;
pub mod discovery;
pub mod error;
pub mod model;
pub mod pathing;
pub mod process_guard;
pub mod project_copy;
pub mod scanner;
pub mod surfaces;
pub mod trash;
pub mod updater;
```

```rust
// src/updater.rs
use std::path::Path;

use anyhow::Result;

use crate::discovery::discover_state;
use crate::surfaces::{
    automation_db::update_automation_db,
    config_toml::update_config_toml,
    global_state::update_global_state,
    jsonl::update_jsonl_file,
    sqlite_threads::update_threads_db,
};

pub fn update_codex_home(codex_home: &Path, old: &str, new: &str) -> Result<usize> {
    let state = discover_state(codex_home)?;
    let mut changed = 0;

    for file in state.jsonl_files {
        changed += update_jsonl_file(&file, old, new)?;
    }

    for db in state.sqlite_state_dbs {
        changed += update_threads_db(&db, old, new)?;
    }

    if let Some(file) = state.global_state_json {
        changed += update_global_state(&file, old, new)?;
    }

    if let Some(file) = state.config_toml {
        changed += update_config_toml(&file, old, new)?;
    }

    if let Some(db) = state.automation_db {
        changed += update_automation_db(&db, old, new)?;
    }

    Ok(changed)
}
```

- [x] **Step 4: Run updater and scanner tests**

Run: `cargo test --test updater --test scanner`

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add src/lib.rs src/updater.rs tests/updater.rs
git commit -m "feat: update all supported Codex metadata"
```

## Task 15: Implement `apply` for Normal Move and Relink-Only

**Files:**
- Modify: `src/commands/apply.rs`
- Modify: `tests/cli.rs`

- [x] **Step 1: Write failing CLI tests for apply modes**

```rust
// tests/cli.rs
use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

#[test]
fn apply_relink_only_updates_metadata_when_old_is_missing_and_new_exists() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let new = temp.path().join("new-project");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::create_dir_all(&new).unwrap();
    fs::write(home.join("sessions/thread.jsonl"), r#"{"cwd":"/old/project"}"#).unwrap();

    Command::cargo_bin("codex-project-mover")
        .unwrap()
        .args([
            "apply",
            "--old",
            "/old/project",
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
            "--relink-only",
        ])
        .assert()
        .success()
        .stdout(contains("metadata backup:"))
        .stdout(contains("updated 1 metadata reference"));

    assert!(fs::read_to_string(home.join("sessions/thread.jsonl"))
        .unwrap()
        .contains(new.to_str().unwrap()));
}

#[test]
fn apply_relink_only_fails_when_old_exists() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let old = temp.path().join("old-project");
    let new = temp.path().join("new-project");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&old).unwrap();
    fs::create_dir_all(&new).unwrap();

    Command::cargo_bin("codex-project-mover")
        .unwrap()
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
        .failure()
        .stderr(contains("relink-only requires old path to not exist"));
}
```

- [x] **Step 2: Run tests and verify they fail**

Run: `cargo test --test cli apply_relink_only_updates_metadata_when_old_is_missing_and_new_exists apply_relink_only_fails_when_old_exists`

Expected: FAIL because `apply` is still a stub.

- [x] **Step 3: Implement `apply` orchestration**

```rust
// src/commands/apply.rs
use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::backup::create_metadata_backup;
use crate::cli::ApplyArgs;
use crate::pathing::{codex_home_from_arg, normalize_project_path};
use crate::process_guard::assert_no_codex_processes;
use crate::project_copy::{copy_project_tree, verify_project_tree};
use crate::scanner::scan_codex_home;
use crate::trash::move_to_trash;
use crate::updater::update_codex_home;

pub fn run(args: ApplyArgs) -> Result<()> {
    assert_no_codex_processes()?;

    let old = normalize_project_path(args.old)?;
    let new = normalize_project_path(args.new)?;
    let codex_home = codex_home_from_arg(args.codex_home)?;
    let old_str = old.to_string_lossy().to_string();
    let new_str = new.to_string_lossy().to_string();

    if args.relink_only {
        validate_relink_only(&old, &new)?;
    } else {
        validate_normal_move(&old, &new)?;
    }

    let report = scan_codex_home(&codex_home, &old_str, &new_str)?;
    let changed_files = changed_metadata_files(&report);
    let backup = create_metadata_backup(
        &codex_home.join("codex-project-mover-backups"),
        &old_str,
        &new_str,
        (!args.relink_only).then(|| new.clone()),
        &changed_files,
    )?;

    if !args.relink_only {
        copy_project_tree(&old, &new)?;
        verify_project_tree(&old, &new)
            .with_context(|| "copied project tree failed verification; metadata was not changed and old folder was not moved to Trash")?;
    }

    let changed = update_codex_home(&codex_home, &old_str, &new_str)?;
    let remaining = scan_codex_home(&codex_home, &old_str, &new_str)?;
    if remaining.old_reference_count() > 0 {
        bail!(
            "metadata verification failed: {} old-path reference(s) remain. Restore metadata with: codex-project-mover rollback --backup {}",
            remaining.old_reference_count(),
            backup.manifest_path.display()
        );
    }
    let new_references = scan_codex_home(&codex_home, &new_str, &old_str)?;
    if changed > 0 && new_references.old_reference_count() < changed {
        bail!(
            "metadata verification failed: expected at least {} new-path reference(s), found {}. Restore metadata with: codex-project-mover rollback --backup {}",
            changed,
            new_references.old_reference_count(),
            backup.manifest_path.display()
        );
    }

    if !args.relink_only {
        move_to_trash(&old)?;
    }

    println!("metadata backup: {}", backup.backup_dir.display());
    println!("updated {} metadata reference(s)", changed);
    if args.relink_only {
        println!("relink-only complete: project folder was not moved");
    } else {
        println!("move complete: old project folder moved to Trash");
    }
    Ok(())
}

fn validate_normal_move(old: &std::path::Path, new: &std::path::Path) -> Result<()> {
    if !old.is_dir() {
        bail!("normal apply requires old path to exist as a directory: {}", old.display());
    }
    if new.exists() {
        bail!("normal apply requires new path to not exist: {}", new.display());
    }
    if new.starts_with(old) {
        bail!("normal apply requires new path to not be inside old path: {}", new.display());
    }
    Ok(())
}

fn validate_relink_only(old: &std::path::Path, new: &std::path::Path) -> Result<()> {
    if old.exists() {
        bail!("relink-only requires old path to not exist: {}", old.display());
    }
    if !new.is_dir() {
        bail!("relink-only requires new path to exist as a directory: {}", new.display());
    }
    Ok(())
}

fn changed_metadata_files(report: &crate::model::ScanReport) -> Vec<PathBuf> {
    report
        .matches
        .iter()
        .map(|reference| reference.file.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
```

- [x] **Step 4: Add and run the normal move CLI test**

Append this test to `tests/cli.rs`:

```rust
#[test]
fn apply_normal_move_copies_updates_and_moves_old_folder_to_test_trash() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let old = temp.path().join("old-project");
    let new = temp.path().join("nested/new-project");
    let test_trash = temp.path().join("trash");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::create_dir_all(old.join("src")).unwrap();
    fs::write(old.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(
        home.join("sessions/thread.jsonl"),
        format!(r#"{{"cwd":"{}"}}"#, old.display()),
    )
    .unwrap();

    let assert = Command::cargo_bin("codex-project-mover")
        .unwrap()
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
        .assert();

    assert
        .success()
        .stdout(contains("metadata backup:"))
        .stdout(contains("move complete: old project folder moved to Trash"));

    assert!(new.join("src/main.rs").exists());
    assert!(!old.exists());
    assert!(test_trash.join("old-project").exists());
    assert!(fs::read_to_string(home.join("sessions/thread.jsonl"))
        .unwrap()
        .contains(new.to_str().unwrap()));
}
```

Run: `cargo test --test cli apply_relink_only_updates_metadata_when_old_is_missing_and_new_exists apply_relink_only_fails_when_old_exists apply_normal_move_copies_updates_and_moves_old_folder_to_test_trash`

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add src/commands/apply.rs tests/cli.rs
git commit -m "feat: apply metadata relinks and project moves"
```

## Task 16: Implement `rollback`

**Files:**
- Modify: `src/commands/rollback.rs`
- Modify: `tests/cli.rs`

- [x] **Step 1: Write failing rollback CLI test**

```rust
// tests/cli.rs
use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

#[test]
fn rollback_restores_metadata_from_manifest() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let new = temp.path().join("new-project");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::create_dir_all(&new).unwrap();
    let jsonl = home.join("sessions/thread.jsonl");
    fs::write(&jsonl, r#"{"cwd":"/old/project"}"#).unwrap();

    let apply_output = Command::cargo_bin("codex-project-mover")
        .unwrap()
        .args([
            "apply",
            "--old",
            "/old/project",
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
            "--relink-only",
        ])
        .output()
        .unwrap();
    assert!(apply_output.status.success());

    let stdout = String::from_utf8(apply_output.stdout).unwrap();
    let backup_dir = stdout
        .lines()
        .find_map(|line| line.strip_prefix("metadata backup: "))
        .unwrap();

    fs::write(&jsonl, r#"{"cwd":"broken"}"#).unwrap();

    Command::cargo_bin("codex-project-mover")
        .unwrap()
        .args(["rollback", "--backup", &format!("{}/manifest.json", backup_dir)])
        .assert()
        .success()
        .stdout(contains("metadata rollback complete"));

    assert_eq!(fs::read_to_string(&jsonl).unwrap(), r#"{"cwd":"/old/project"}"#);
}

#[test]
fn rollback_after_normal_move_removes_created_new_folder() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let old = temp.path().join("old-project");
    let new = temp.path().join("nested/new-project");
    let test_trash = temp.path().join("trash");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::create_dir_all(old.join("src")).unwrap();
    fs::write(old.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(
        home.join("sessions/thread.jsonl"),
        format!(r#"{{"cwd":"{}"}}"#, old.display()),
    )
    .unwrap();

    let apply_output = Command::cargo_bin("codex-project-mover")
        .unwrap()
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
        .output()
        .unwrap();
    assert!(apply_output.status.success());

    let stdout = String::from_utf8(apply_output.stdout).unwrap();
    let backup_dir = stdout
        .lines()
        .find_map(|line| line.strip_prefix("metadata backup: "))
        .unwrap();

    Command::cargo_bin("codex-project-mover")
        .unwrap()
        .env("CODEX_PROJECT_MOVER_TEST_TRASH_DIR", &test_trash)
        .args(["rollback", "--backup", &format!("{}/manifest.json", backup_dir)])
        .assert()
        .success()
        .stdout(contains("removed created new project folder"))
        .stdout(contains("metadata rollback complete"));

    assert!(!new.exists());
    assert!(test_trash.join("new-project").exists());
}
```

- [x] **Step 2: Run rollback test and verify it fails**

Run: `cargo test --test cli rollback_restores_metadata_from_manifest rollback_after_normal_move_removes_created_new_folder`

Expected: FAIL because rollback is still a stub.

- [x] **Step 3: Implement rollback command**

```rust
// src/commands/rollback.rs
use anyhow::Result;

use crate::backup::restore_metadata_backup;
use crate::cli::RollbackArgs;
use crate::process_guard::assert_no_codex_processes;
use crate::trash::move_to_trash;

pub fn run(args: RollbackArgs) -> Result<()> {
    assert_no_codex_processes()?;
    let manifest = restore_metadata_backup(&args.backup)?;
    if let Some(created_new_project_path) = manifest.created_new_project_path {
        if created_new_project_path.exists() {
            move_to_trash(&created_new_project_path)?;
            println!(
                "removed created new project folder: {}",
                created_new_project_path.display()
            );
        }
    }
    println!("metadata rollback complete");
    println!("old project folder was not restored from Trash");
    Ok(())
}
```

- [x] **Step 4: Run rollback test and full CLI suite**

Run: `cargo test --test cli`

Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add src/commands/rollback.rs tests/cli.rs
git commit -m "feat: restore metadata from backups"
```

## Task 17: Add End-to-End Fixture Coverage and User Documentation

**Files:**
- Create: `tests/e2e.rs`
- Create: `README.md`
- Modify: `SPEC.md`

- [x] **Step 1: Write end-to-end fixture test**

```rust
// tests/e2e.rs
use assert_cmd::Command;
use rusqlite::Connection;
use std::fs;
use tempfile::tempdir;

#[test]
fn relink_only_end_to_end_updates_all_fixture_surfaces() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    let new = temp.path().join("new-project");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::create_dir_all(home.join("archived_sessions")).unwrap();
    fs::create_dir_all(home.join("sqlite")).unwrap();
    fs::create_dir_all(&new).unwrap();

    fs::write(home.join("sessions/live.jsonl"), r#"{"cwd":"/old/project"}"#).unwrap();
    fs::write(home.join("archived_sessions/old.jsonl"), r#"{"payload":{"cwd":"/old/project"}}"#).unwrap();
    fs::write(home.join(".codex-global-state.json"), r#"{"roots":["/old/project"]}"#).unwrap();
    fs::write(home.join("config.toml"), "[desktop]\nopen_target = \"/old/project\"\n").unwrap();

    let state_db = home.join("state_main.sqlite");
    let state_conn = Connection::open(&state_db).unwrap();
    state_conn.execute("CREATE TABLE threads (id TEXT PRIMARY KEY, cwd TEXT)", []).unwrap();
    state_conn.execute("INSERT INTO threads (id, cwd) VALUES ('t1', '/old/project')", []).unwrap();

    let automation_db = home.join("sqlite/codex-dev.db");
    let automation_conn = Connection::open(&automation_db).unwrap();
    automation_conn.execute("CREATE TABLE automations (id TEXT PRIMARY KEY, cwds TEXT)", []).unwrap();
    automation_conn
        .execute(r#"INSERT INTO automations (id, cwds) VALUES ('a1', '["/old/project"]')"#, [])
        .unwrap();

    Command::cargo_bin("codex-project-mover")
        .unwrap()
        .args([
            "apply",
            "--old",
            "/old/project",
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
            "--relink-only",
        ])
        .assert()
        .success();

    Command::cargo_bin("codex-project-mover")
        .unwrap()
        .args([
            "verify",
            "--old",
            "/old/project",
            "--new",
            new.to_str().unwrap(),
            "--codex-home",
            home.to_str().unwrap(),
        ])
        .assert()
        .success();
}
```

- [x] **Step 2: Run end-to-end test and verify it passes**

Run: `cargo test --test e2e`

Expected: PASS.

- [x] **Step 3: Write README usage documentation**

```markdown
# Codex Project Mover

`codex-project-mover` is a Mac-first CLI for moving a Codex Desktop project folder and updating local Codex metadata so existing conversations continue to point at the new path.

## Commands

```bash
codex-project-mover plan --old /old/project --new /new/project
codex-project-mover apply --old /old/project --new /new/project
codex-project-mover apply --old /old/project --new /new/project --relink-only
codex-project-mover verify --old /old/project --new /new/project
codex-project-mover rollback --backup ~/.codex/codex-project-mover-backups/<id>/manifest.json
codex-project-mover rollback --backup ~/.codex/codex-project-mover-backups/<id>
```

Close Codex before running commands. The tool exits if it sees the main Codex Desktop process, `codex app-server`, or a standalone `codex` CLI/`codex exec` process. Pass `--allow-running-codex` only after deciding the reported process is unrelated to the move.

Normal `apply` backs up Codex metadata, copies the old folder to the new path, verifies the copy, updates supported metadata, verifies the old path is gone and new references are present, and moves the old folder to macOS Trash.

Relink-only mode is for folders already moved by the user. It requires the old path to be missing and the new path to exist.

Verify checks that supported old-path references are gone and supported new-path references are present.

Rollback restores Codex metadata from a backup manifest. If the tool created the new project folder during normal apply, rollback moves that new folder to Trash. It does not restore the old folder from Trash.
```

- [x] **Step 4: Update `SPEC.md` decisions**

Add a short `## Resolved Decisions` section after `Open Questions` with these bullets:

```markdown
## Resolved Decisions

- Normal `apply` moves the project folder by default.
- Normal `apply` requires the old folder to exist and the new path to not exist.
- Relink-only mode requires the old folder to not exist and the new folder to exist.
- New parent directories are created automatically.
- Commands exit when relevant Codex local-state processes are running, excluding the mover process itself. Users can pass `--allow-running-codex` to proceed after reviewing the reported process list.
- The old folder is moved to macOS Trash after copy and metadata verification.
- Backups are metadata backups under `~/.codex/codex-project-mover-backups/<id>` and include movement metadata for rollback cleanup.
- Rollback restores metadata from backup and moves the tool-created new folder to Trash when applicable.
- `.codex-global-state.json` updates JSON string values and JSON object keys exactly equal to the old path.
- `config.toml` updates exact string values, exact `[projects."/path"]` table keys, and exact per-path open-target preference keys.
- `verify` requires supported old-path references to be gone and supported new-path references to be present.
```

- [x] **Step 5: Run all tests**

Run: `cargo test`

Expected: PASS.

- [x] **Step 6: Commit**

```bash
git add README.md SPEC.md tests/e2e.rs
git commit -m "docs: document Codex project mover workflow"
```

## Task 18: Release Readiness Checks

**Files:**
- Modify: `README.md`
- Create: `.github/workflows/ci.yml`

- [x] **Step 1: Add CI workflow**

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
  pull_request:

jobs:
  test:
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test
      - run: cargo build --release
```

- [x] **Step 2: Run local formatting and tests**

Run:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

Expected: all commands PASS.

- [x] **Step 3: Add release build note to README**

```markdown
## Build

```bash
cargo build --release
./target/release/codex-project-mover --help
```

The first release target is Apple Silicon macOS. Intel macOS can be added with a second release artifact after the v1 workflow is stable.
```

- [x] **Step 4: Commit**

```bash
git add README.md .github/workflows/ci.yml
git commit -m "ci: add macOS Rust checks"
```

## Self-Review

- Spec coverage: The plan covers Rust CLI scaffolding, `plan`, `apply`, `verify`, `rollback`, path normalization, focused Codex process detection with an explicit user override, metadata backups with movement metadata, copy-before-metadata-update movement, Trash cleanup, JSONL `cwd`, SQLite `threads.cwd`, global state exact JSON strings and exact path keys, `config.toml` exact values and exact path keys, automation `cwds`, relink-only mode, and tests.
- Clarified decisions: The plan reflects the follow-up decisions that normal apply moves by default, relink-only requires the old folder to be absent, parent directories are created automatically, old folders go to macOS Trash, rollback restores metadata and removes the created new folder when applicable, `.codex-global-state.json` requires whole-string/key exact matches, and `verify` checks expected new references.
- Safety coverage: The plan never uses broad substring replacement in session history or global state, updates known SQLite columns only, backs up every changed metadata file before copying or editing metadata, verifies copied code before metadata changes, verifies no supported old-path references remain, verifies new-path references are present, and moves project folders to Trash instead of deleting them directly.
- Placeholder scan: No task relies on unspecified future behavior. Each task names files, tests, commands, expected results, and concrete implementation interfaces.
- Type consistency: Shared types are introduced in `src/model.rs`; scanner and updater tasks use the same `ReferenceMatch`, `SurfaceKind`, and `ScanReport` names; command tasks call the scanner/updater APIs defined earlier.

## Execution Notes

- Completed Tasks 1-18 on branch `implement-codex-project-mover`.
- Implementation commit: `133be58` (`feat: implement Codex project mover`).
- Files changed: Rust CLI sources under `src/`, integration/unit tests under `tests/`, `Cargo.toml`, `Cargo.lock`, `README.md`, `SPEC.md`, `.github/workflows/ci.yml`, `.gitignore`, and this plan.
- Verification run after implementation:
  - `cargo fmt --check`: PASS
  - `cargo clippy --all-targets -- -D warnings`: PASS
  - `cargo test`: PASS
  - `cargo build --release`: PASS
- Notable implementation adjustment: CLI integration tests set `CODEX_PROJECT_MOVER_TEST_SKIP_PROCESS_GUARD=1` so tests can exercise temp Codex homes while the real Codex app is open. The public CLI also has an explicit `--allow-running-codex` user override for cases where the detected process is unrelated to the project being moved.
- V1 API hardening follow-up: `--version` is supported, and `rollback --backup` accepts either the backup directory printed by `apply` or the `manifest.json` file inside it.
