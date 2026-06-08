use serde::Serialize;

use crate::app_error::{AppError, AppResult, ExitCode, ResultExitCodeExt};
use crate::cli::MoveArgs;
use crate::git_worktree::{build_plan_for_relink_only, verify_git_from_new_path, GitWorktreeKind};
use crate::output::{path_string, print_json, OutputMode};
use crate::pathing::{codex_home_from_arg, normalize_project_path};
use crate::process_guard::assert_no_codex_processes;
use crate::scanner::scan_codex_home;

#[derive(Debug, Serialize)]
struct VerifyOutput {
    command: &'static str,
    status: &'static str,
    old_path: String,
    new_path: String,
    codex_home: String,
    old_reference_count: usize,
    new_reference_count: usize,
    git_worktree_verification: &'static str,
}

pub fn run(args: MoveArgs, output_mode: OutputMode) -> AppResult<()> {
    assert_no_codex_processes(args.allow_running_codex).exit_code(ExitCode::ProcessGuard)?;
    let old = normalize_project_path(args.old).exit_code(ExitCode::PathValidation)?;
    let new = normalize_project_path(args.new).exit_code(ExitCode::PathValidation)?;
    let codex_home = codex_home_from_arg(args.codex_home).exit_code(ExitCode::PathValidation)?;
    let old_str = old.to_string_lossy().to_string();
    let new_str = new.to_string_lossy().to_string();
    let old_report =
        scan_codex_home(&codex_home, &old_str, &new_str).exit_code(ExitCode::Verification)?;
    let new_report =
        scan_codex_home(&codex_home, &new_str, &old_str).exit_code(ExitCode::Verification)?;

    if old_report.old_reference_count() > 0 {
        return Err(AppError::message(
            ExitCode::Verification,
            format!(
                "verification failed: {} old-path reference remains",
                old_report.old_reference_count()
            ),
        ));
    }

    if new_report.old_reference_count() == 0 {
        return Err(AppError::message(
            ExitCode::Verification,
            "verification failed: no supported new-path references found",
        ));
    }

    let git_plan = build_plan_for_relink_only(&old, &new).exit_code(ExitCode::Verification)?;
    let git_worktree_verification = if git_plan.kind != GitWorktreeKind::NotGit {
        "passed"
    } else {
        "not_applicable"
    };
    if git_plan.kind != GitWorktreeKind::NotGit {
        verify_git_from_new_path(&old, &new).exit_code(ExitCode::Verification)?;
        if !output_mode.is_json() {
            println!("Git worktree verification passed");
        }
    }

    if output_mode.is_json() {
        let output = VerifyOutput {
            command: "verify",
            status: "ok",
            old_path: path_string(&old),
            new_path: path_string(&new),
            codex_home: path_string(&codex_home),
            old_reference_count: old_report.old_reference_count(),
            new_reference_count: new_report.old_reference_count(),
            git_worktree_verification,
        };
        return print_json(&output);
    }

    println!(
        "verification passed: no supported old-path references remain; {} new-path reference(s) found",
        new_report.old_reference_count()
    );
    Ok(())
}
