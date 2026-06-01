use std::{fs, path::Path, process::Command};

use codex_project_mover::git_worktree::{
    build_plan_for_existing_project, map_worktree_paths, move_linked_worktree, parse_worktree_list,
    repair_main_worktree_after_copy, verify_git_from_new_path, verify_git_worktree_state,
    GitWorktreeKind, GitWorktreePlan, WorktreePathMove,
};
use tempfile::tempdir;

#[test]
fn parses_porcelain_z_worktree_entries() {
    let input = b"worktree /repo/main\0HEAD 1111111111111111111111111111111111111111\0branch refs/heads/main\0\0worktree /repo/linked\0HEAD 2222222222222222222222222222222222222222\0detached\0locked manual lock\0prunable gitdir file points to non-existent location\0\0";

    let entries = parse_worktree_list(input).unwrap();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].path, Path::new("/repo/main"));
    assert_eq!(
        entries[0].head.as_deref(),
        Some("1111111111111111111111111111111111111111")
    );
    assert_eq!(entries[0].branch.as_deref(), Some("refs/heads/main"));
    assert!(!entries[0].detached);
    assert_eq!(entries[1].path, Path::new("/repo/linked"));
    assert_eq!(
        entries[1].head.as_deref(),
        Some("2222222222222222222222222222222222222222")
    );
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

#[test]
fn maps_private_var_worktree_paths_against_var_root_without_rewriting_output_paths() {
    let entries = parse_worktree_list(
        b"worktree /private/var/tmp/old/project\0HEAD a\0branch refs/heads/main\0\0worktree /private/var/tmp/old/project/.worktrees/feature\0HEAD b\0branch refs/heads/feature\0\0",
    )
    .unwrap();

    let moves = map_worktree_paths(
        &entries,
        Path::new("/var/tmp/old/project"),
        Path::new("/new/project"),
    );

    assert_eq!(
        moves,
        vec![
            WorktreePathMove {
                old_path: Path::new("/private/var/tmp/old/project").to_path_buf(),
                new_path: Path::new("/new/project").to_path_buf(),
            },
            WorktreePathMove {
                old_path: Path::new("/private/var/tmp/old/project/.worktrees/feature")
                    .to_path_buf(),
                new_path: Path::new("/new/project/.worktrees/feature").to_path_buf(),
            },
        ]
    );
}

#[cfg(unix)]
#[test]
fn maps_worktree_paths_through_symlinked_old_root() {
    let temp = tempdir().unwrap();
    let real_root = temp.path().join("real-root");
    let link_root = temp.path().join("link-root");
    let real_old = real_root.join("repo");
    let link_old = link_root.join("repo");
    let new = temp.path().join("moved/repo");
    fs::create_dir_all(real_old.join(".worktrees/feature")).unwrap();
    std::os::unix::fs::symlink(&real_root, &link_root).unwrap();

    let entries = vec![
        codex_project_mover::git_worktree::WorktreeEntry {
            path: real_old.clone(),
            head: Some("a".to_string()),
            branch: Some("refs/heads/main".to_string()),
            detached: false,
            bare: false,
            locked: None,
            prunable: None,
        },
        codex_project_mover::git_worktree::WorktreeEntry {
            path: real_old.join(".worktrees/feature"),
            head: Some("b".to_string()),
            branch: Some("refs/heads/feature".to_string()),
            detached: false,
            bare: false,
            locked: None,
            prunable: None,
        },
    ];

    let moves = map_worktree_paths(&entries, &link_old, &new);

    assert_eq!(
        moves,
        vec![
            WorktreePathMove {
                old_path: real_old.clone(),
                new_path: new.clone(),
            },
            WorktreePathMove {
                old_path: real_old.join(".worktrees/feature"),
                new_path: new.join(".worktrees/feature"),
            },
        ]
    );
}

#[test]
fn repair_paths_excludes_main_project_path_after_cleaning_for_comparison() {
    let plan = GitWorktreePlan {
        kind: GitWorktreeKind::MainWorktree,
        project_path: Path::new("/var/tmp/old/project").to_path_buf(),
        new_project_path: Path::new("/new/project").to_path_buf(),
        entries: Vec::new(),
        path_moves: vec![
            WorktreePathMove {
                old_path: Path::new("/private/var/tmp/old/./project").to_path_buf(),
                new_path: Path::new("/new/project").to_path_buf(),
            },
            WorktreePathMove {
                old_path: Path::new("/private/var/tmp/old/project/.worktrees/feature")
                    .to_path_buf(),
                new_path: Path::new("/new/project/.worktrees/feature").to_path_buf(),
            },
        ],
    };

    assert_eq!(
        plan.repair_paths(),
        vec![Path::new("/new/project/.worktrees/feature").to_path_buf()]
    );
}

#[test]
fn parses_exact_locked_and_prunable_without_reasons() {
    let entries = parse_worktree_list(
        b"worktree /repo/linked\0HEAD a\0branch refs/heads/main\0locked\0prunable\0\0",
    )
    .unwrap();

    assert_eq!(entries[0].locked.as_deref(), Some(""));
    assert_eq!(entries[0].prunable.as_deref(), Some(""));
}

#[test]
fn ignores_unknown_fields_that_start_with_locked() {
    let entries = parse_worktree_list(
        b"worktree /repo/linked\0HEAD a\0branch refs/heads/main\0lockedness future field\0\0",
    )
    .unwrap();

    assert_eq!(entries[0].locked, None);
}

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(cwd)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "init.templateDir=",
        ])
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

fn init_repo(path: &Path) {
    fs::create_dir_all(path).unwrap();
    git(path, &["init"]);
    git(path, &["config", "user.email", "test@example.com"]);
    git(path, &["config", "user.name", "Test User"]);
    fs::write(path.join("README.md"), "hello\n").unwrap();
    git(path, &["add", "README.md"]);
    git(path, &["commit", "-m", "initial"]);
}

#[test]
fn detects_non_git_directory_as_not_git() {
    let temp = tempdir().unwrap();
    let old = temp.path().join("plain");
    let new = temp.path().join("moved/plain");
    fs::create_dir_all(&old).unwrap();

    let plan = build_plan_for_existing_project(&old, &new).unwrap();

    assert_eq!(plan.kind, GitWorktreeKind::NotGit);
    assert_eq!(plan.project_path, old);
    assert_eq!(plan.new_project_path, new);
    assert!(plan.entries.is_empty());
    assert!(plan.path_moves.is_empty());
}

#[test]
fn invalid_dotgit_file_returns_path_context_error() {
    let temp = tempdir().unwrap();
    let old = temp.path().join("repo");
    let new = temp.path().join("moved/repo");
    fs::create_dir_all(&old).unwrap();
    let dotgit = old.join(".git");
    fs::write(&dotgit, "not a gitdir pointer\n").unwrap();

    let error = build_plan_for_existing_project(&old, &new).unwrap_err();
    let message = error.to_string();

    assert!(message.contains(dotgit.to_str().unwrap()));
    assert!(message.contains("gitdir:"));
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
    let old = fs::canonicalize(old).unwrap();

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
    git(
        &main,
        &["worktree", "add", linked.to_str().unwrap(), "-b", "feature"],
    );
    let main = fs::canonicalize(main).unwrap();
    let linked = fs::canonicalize(linked).unwrap();

    let plan = build_plan_for_existing_project(&linked, &new).unwrap();

    assert_eq!(plan.kind, GitWorktreeKind::LinkedWorktree);
    assert_eq!(plan.project_path, linked);
    assert_eq!(plan.new_project_path, new);
    assert!(plan.entries.iter().any(|entry| entry.path == main));
    assert!(plan
        .entries
        .iter()
        .any(|entry| entry.path == plan.project_path));
    assert!(plan
        .path_moves
        .iter()
        .any(|path_move| path_move.old_path == plan.project_path
            && path_move.new_path == plan.new_project_path));
}

#[test]
fn separate_git_dir_under_worktrees_component_is_main_worktree() {
    if !git_available() {
        eprintln!(
            "skipping separate_git_dir_under_worktrees_component_is_main_worktree because git is unavailable"
        );
        return;
    }

    let temp = tempdir().unwrap();
    let worktree = temp.path().join("repo");
    let separate_git_dir = temp.path().join("worktrees/repo.git");
    let new = temp.path().join("moved/repo");
    fs::create_dir_all(&worktree).unwrap();
    fs::create_dir_all(separate_git_dir.parent().unwrap()).unwrap();
    git(
        &worktree,
        &[
            "init",
            "--separate-git-dir",
            separate_git_dir.to_str().unwrap(),
        ],
    );
    git(&worktree, &["config", "user.email", "test@example.com"]);
    git(&worktree, &["config", "user.name", "Test User"]);
    fs::write(worktree.join("README.md"), "hello\n").unwrap();
    git(&worktree, &["add", "README.md"]);
    git(&worktree, &["commit", "-m", "initial"]);
    let worktree = fs::canonicalize(worktree).unwrap();

    let plan = build_plan_for_existing_project(&worktree, &new).unwrap();

    assert_eq!(plan.kind, GitWorktreeKind::MainWorktree);
    assert_eq!(plan.project_path, worktree);
}

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
    git(
        &main,
        &["worktree", "add", linked.to_str().unwrap(), "-b", "feature"],
    );

    let plan = build_plan_for_existing_project(&linked, &new_linked).unwrap();
    move_linked_worktree(&plan).unwrap();

    assert!(!linked.exists());
    assert!(new_linked.exists());
    git(&new_linked, &["status", "--short"]);
}

#[test]
fn moves_linked_worktree_from_bare_backed_repository() {
    if !git_available() {
        eprintln!(
            "skipping moves_linked_worktree_from_bare_backed_repository because git is unavailable"
        );
        return;
    }

    let temp = tempdir().unwrap();
    let seed = temp.path().join("seed");
    let bare = temp.path().join("repo.git");
    let linked = temp.path().join("linked");
    let new_linked = temp.path().join("nested/moved-linked");
    init_repo(&seed);
    git(&seed, &["branch", "-M", "main"]);
    git(
        temp.path(),
        &[
            "clone",
            "--bare",
            seed.to_str().unwrap(),
            bare.to_str().unwrap(),
        ],
    );
    git(
        &bare,
        &["worktree", "add", linked.to_str().unwrap(), "main"],
    );

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
        entries: parse_worktree_list(
            b"worktree /old/project/.worktrees/feature\0HEAD a\0branch refs/heads/feature\0\0",
        )
        .unwrap(),
        path_moves: vec![],
    };

    let error = verify_git_worktree_state(&plan).unwrap_err().to_string();

    assert!(error.contains("old Git worktree path remains"));
}

#[test]
fn verify_git_from_new_path_rejects_non_git_new_path() {
    let temp = tempdir().unwrap();
    let old = temp.path().join("old");
    let new = temp.path().join("new");
    fs::create_dir_all(&new).unwrap();

    let error = verify_git_from_new_path(&old, &new)
        .unwrap_err()
        .to_string();

    assert!(error.contains("not a Git worktree"));
}

#[test]
fn verify_git_from_new_path_succeeds_after_linked_worktree_move() {
    if !git_available() {
        eprintln!(
            "skipping verify_git_from_new_path_succeeds_after_linked_worktree_move because git is unavailable"
        );
        return;
    }

    let temp = tempdir().unwrap();
    let main = temp.path().join("repo");
    let linked = temp.path().join("linked");
    let new_linked = temp.path().join("nested/moved-linked");
    init_repo(&main);
    git(
        &main,
        &["worktree", "add", linked.to_str().unwrap(), "-b", "feature"],
    );

    let plan = build_plan_for_existing_project(&linked, &new_linked).unwrap();
    move_linked_worktree(&plan).unwrap();

    verify_git_from_new_path(&linked, &new_linked).unwrap();
}

#[test]
fn repair_main_worktree_after_copy_repairs_nested_linked_worktree() {
    if !git_available() {
        eprintln!(
            "skipping repair_main_worktree_after_copy_repairs_nested_linked_worktree because git is unavailable"
        );
        return;
    }

    let temp = tempdir().unwrap();
    let main = temp.path().join("repo");
    let linked_parent = main.join(".worktrees");
    let linked = linked_parent.join("feature");
    let new_main = temp.path().join("moved/repo");
    let new_linked = new_main.join(".worktrees/feature");
    init_repo(&main);
    fs::create_dir_all(&linked_parent).unwrap();
    git(
        &main,
        &["worktree", "add", linked.to_str().unwrap(), "-b", "feature"],
    );

    let plan = build_plan_for_existing_project(&main, &new_main).unwrap();
    fs::create_dir_all(new_main.parent().unwrap()).unwrap();
    fs::rename(&main, &new_main).unwrap();
    repair_main_worktree_after_copy(&plan).unwrap();

    git(&new_main, &["status", "--short"]);
    git(&new_linked, &["status", "--short"]);
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

#[test]
fn repair_main_worktree_after_copy_noops_for_non_git_plan() {
    let plan = GitWorktreePlan::no_git(Path::new("/old/project"), Path::new("/new/project"));

    repair_main_worktree_after_copy(&plan).unwrap();
}
