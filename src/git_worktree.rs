use std::{
    fs,
    path::{Component, Path, PathBuf},
    process::{Command, Output},
};

use anyhow::{bail, Context, Result};

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
        let project_path = normalize_path_for_compare(&self.project_path);

        self.path_moves
            .iter()
            .filter(|path_move| normalize_path_for_compare(&path_move.old_path) != project_path)
            .map(|path_move| path_move.new_path.clone())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GitDotfile {
    Missing,
    Directory,
    File { gitdir: PathBuf },
}

pub fn build_plan_for_existing_project(old: &Path, new: &Path) -> Result<GitWorktreePlan> {
    match inspect_dotgit(old)? {
        GitDotfile::Missing => Ok(GitWorktreePlan::no_git(old, new)),
        GitDotfile::Directory => {
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
            let kind = if is_linked_worktree_gitdir(&gitdir) {
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
        return Ok(GitDotfile::Directory);
    }

    let contents = fs::read_to_string(&dotgit)?;
    let Some(raw_gitdir) = contents.trim().strip_prefix("gitdir:") else {
        bail!(
            "invalid .git file at {}: expected `gitdir:` pointer",
            dotgit.display()
        );
    };
    let raw_gitdir = raw_gitdir.trim();
    if raw_gitdir.is_empty() {
        bail!(
            "invalid .git file at {}: expected non-empty `gitdir:` pointer",
            dotgit.display()
        );
    }
    let gitdir = if Path::new(raw_gitdir).is_absolute() {
        PathBuf::from(raw_gitdir)
    } else {
        project.join(raw_gitdir)
    };

    Ok(GitDotfile::File { gitdir })
}

fn is_linked_worktree_gitdir(gitdir: &Path) -> bool {
    gitdir.join("commondir").is_file()
}

fn git_worktree_entries(cwd: &Path) -> Result<Vec<WorktreeEntry>> {
    let output = run_git(cwd, &["worktree", "list", "--porcelain", "-z"])?;
    parse_worktree_list(&output.stdout)
}

fn run_git(cwd: &Path, args: &[&str]) -> Result<Output> {
    let command_label = git_command_label(args);
    let output = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .with_context(|| format!("failed to spawn `{command_label}` in {}", cwd.display()))?;

    if !output.status.success() {
        bail!(
            "git command failed: `{}`\ncwd: {}\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
            command_label,
            cwd.display(),
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(output)
}

fn git_command_label(args: &[&str]) -> String {
    std::iter::once("git")
        .chain(args.iter().copied())
        .collect::<Vec<_>>()
        .join(" ")
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
            } else if field == "locked" {
                entry.locked = Some(String::new());
            } else if let Some(reason) = field.strip_prefix("locked ") {
                entry.locked = Some(reason.to_string());
            } else if field == "prunable" {
                entry.prunable = Some(String::new());
            } else if let Some(reason) = field.strip_prefix("prunable ") {
                entry.prunable = Some(reason.to_string());
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

pub fn map_worktree_paths(
    entries: &[WorktreeEntry],
    old: &Path,
    new: &Path,
) -> Vec<WorktreePathMove> {
    let old = normalize_path_for_compare(old);

    entries
        .iter()
        .filter_map(|entry| {
            let entry_path = normalize_path_for_compare(&entry.path);
            entry_path
                .strip_prefix(&old)
                .ok()
                .map(|relative| WorktreePathMove {
                    old_path: entry.path.clone(),
                    new_path: new.join(relative),
                })
        })
        .collect()
}

fn normalize_path_for_compare(path: &Path) -> PathBuf {
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

    match cleaned.strip_prefix("/private/var") {
        Ok(relative) if relative.as_os_str().is_empty() => PathBuf::from("/var"),
        Ok(relative) => PathBuf::from("/var").join(relative),
        Err(_) => cleaned,
    }
}
