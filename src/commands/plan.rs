use anyhow::Result;

use crate::cli::MoveArgs;
use crate::git_worktree::{build_plan_for_existing_project, GitWorktreeKind};
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
    println!(
        "{} old-path reference(s) found",
        report.old_reference_count()
    );
    for reference in report.matches {
        println!(
            "- {:?}: {} {}",
            reference.surface,
            reference.file.display(),
            reference.location
        );
    }

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
    Ok(())
}
