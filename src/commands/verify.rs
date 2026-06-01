use anyhow::{bail, Result};

use crate::cli::MoveArgs;
use crate::git_worktree::{build_plan_for_relink_only, verify_git_from_new_path, GitWorktreeKind};
use crate::pathing::{codex_home_from_arg, normalize_project_path};
use crate::process_guard::assert_no_codex_processes;
use crate::scanner::scan_codex_home;

pub fn run(args: MoveArgs) -> Result<()> {
    assert_no_codex_processes(args.allow_running_codex)?;
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

    let git_plan = build_plan_for_relink_only(&old, &new)?;
    if git_plan.kind != GitWorktreeKind::NotGit {
        verify_git_from_new_path(&old, &new)?;
        println!("Git worktree verification passed");
    }

    println!(
        "verification passed: no supported old-path references remain; {} new-path reference(s) found",
        new_report.old_reference_count()
    );
    Ok(())
}
