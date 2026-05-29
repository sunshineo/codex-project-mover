use std::path::Path;

use codex_project_mover::git_worktree::{
    map_worktree_paths, parse_worktree_list, GitWorktreeKind, GitWorktreePlan, WorktreePathMove,
};

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
