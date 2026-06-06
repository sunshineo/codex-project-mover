use anyhow::Result;

use crate::cli::MoveArgs;
use crate::git_worktree::{
    build_plan_for_existing_project, linked_worktree_move_cwd, GitWorktreeKind,
};
use crate::pathing::{codex_home_from_arg, normalize_project_path};
use crate::process_guard::{detect_codex_processes, render_process_report};
use crate::scanner::scan_codex_home;

pub fn run(args: MoveArgs) -> Result<()> {
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
            let repair_paths = git_plan.repair_paths();
            let repair_path_args = repair_paths
                .iter()
                .map(|path| format!(" {}", path.display()))
                .collect::<String>();
            println!(
                "Git repair: git -C {} worktree repair{}",
                git_plan.new_project_path.display(),
                repair_path_args
            );
        }
        GitWorktreeKind::LinkedWorktree => {
            println!("Git worktree: linked worktree");
            let cwd = linked_worktree_move_cwd(&git_plan)?;
            println!(
                "Git move: git -C {} worktree move {} {}",
                cwd.display(),
                git_plan.project_path.display(),
                git_plan.new_project_path.display()
            );
        }
    }

    for line in render_process_report(&detect_codex_processes(), args.allow_running_codex) {
        println!("{line}");
    }
    Ok(())
}
