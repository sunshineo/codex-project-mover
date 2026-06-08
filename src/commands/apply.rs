use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::Context;
use serde::Serialize;

use crate::app_error::{AppError, AppResult, ExitCode, ResultExitCodeExt};
use crate::backup::{create_metadata_backup, restore_metadata_backup};
use crate::cli::ApplyArgs;
use crate::git_worktree::{
    build_plan_for_existing_project, build_plan_for_relink_only, move_linked_worktree,
    repair_linked_worktree_after_manual_move, repair_main_worktree_after_copy,
    stop_fsmonitor_daemons_for_move, verify_git_from_new_path, GitWorktreeKind,
};
use crate::output::{path_string, print_json, OutputMode, RollbackOutput};
use crate::pathing::{codex_home_from_arg, normalize_project_path};
use crate::process_guard::assert_no_codex_processes;
use crate::project_copy::{copy_project_tree, verify_project_tree};
use crate::scanner::scan_codex_home;
use crate::trash::move_to_trash;
use crate::updater::update_codex_home;

#[derive(Debug, Serialize)]
struct ApplyOutput {
    command: &'static str,
    status: &'static str,
    old_path: String,
    new_path: String,
    codex_home: String,
    relink_only: bool,
    backup_dir: String,
    backup_manifest: String,
    changed_reference_count: usize,
    new_reference_count: usize,
    git_worktree_action: &'static str,
    project_folder_action: &'static str,
    rollback: RollbackOutput,
}

pub fn run(args: ApplyArgs, output_mode: OutputMode) -> AppResult<()> {
    assert_no_codex_processes(args.allow_running_codex).exit_code(ExitCode::ProcessGuard)?;

    let old = normalize_project_path(args.old).exit_code(ExitCode::PathValidation)?;
    let new = normalize_project_path(args.new).exit_code(ExitCode::PathValidation)?;
    let codex_home = codex_home_from_arg(args.codex_home).exit_code(ExitCode::PathValidation)?;
    let old_str = old.to_string_lossy().to_string();
    let new_str = new.to_string_lossy().to_string();
    let auto_rollback = args.auto_rollback;

    if args.relink_only {
        validate_relink_only(&old, &new)?;
    } else {
        validate_normal_move(&old, &new)?;
    }

    let git_plan = if args.relink_only {
        Some(build_plan_for_relink_only(&old, &new).exit_code(ExitCode::MoveOperation)?)
    } else {
        Some(build_plan_for_existing_project(&old, &new).exit_code(ExitCode::MoveOperation)?)
    };
    let created_new_project_path = if !args.relink_only
        && !matches!(
            git_plan.as_ref().map(|plan| &plan.kind),
            Some(GitWorktreeKind::LinkedWorktree)
        ) {
        Some(new.clone())
    } else {
        None
    };

    let report =
        scan_codex_home(&codex_home, &old_str, &new_str).exit_code(ExitCode::MetadataUpdate)?;
    let changed_files = changed_metadata_files(&report);
    let backup = create_metadata_backup(
        &codex_home.join("codex-project-mover-backups"),
        &old_str,
        &new_str,
        created_new_project_path,
        &changed_files,
    )
    .exit_code(ExitCode::Backup)?;

    if !args.relink_only {
        if let Some(plan) = git_plan.as_ref() {
            let stopped =
                stop_fsmonitor_daemons_for_move(plan).exit_code(ExitCode::MoveOperation)?;
            if !stopped.is_empty() && !output_mode.is_json() {
                println!("Git fsmonitor stopped for {} worktree(s)", stopped.len());
            }
        }

        match git_plan.as_ref() {
            Some(plan) if plan.kind == GitWorktreeKind::LinkedWorktree => {
                move_linked_worktree(plan).exit_code(ExitCode::MoveOperation)?;
                verify_git_from_new_path(&old, &new)
                    .with_context(|| {
                        "Git worktree verification failed after move; metadata was not changed"
                    })
                    .exit_code(ExitCode::Verification)?;
                if !output_mode.is_json() {
                    println!("Git worktree move complete");
                }
            }
            Some(plan) if plan.kind == GitWorktreeKind::MainWorktree => {
                copy_project_tree(&old, &new).exit_code(ExitCode::MoveOperation)?;
                verify_project_tree(&old, &new)
                    .with_context(|| "copied project tree failed verification; metadata was not changed and old folder was not moved to Trash")
                    .exit_code(ExitCode::Verification)?;
                repair_main_worktree_after_copy(plan).exit_code(ExitCode::MoveOperation)?;
                verify_git_from_new_path(&old, &new).with_context(|| {
                    "Git worktree verification failed after repair; metadata was not changed and old folder was not moved to Trash"
                }).exit_code(ExitCode::Verification)?;
                if !output_mode.is_json() {
                    println!("Git worktree repair complete");
                }
            }
            Some(_) | None => {
                copy_project_tree(&old, &new).exit_code(ExitCode::MoveOperation)?;
                verify_project_tree(&old, &new)
                    .with_context(|| "copied project tree failed verification; metadata was not changed and old folder was not moved to Trash")
                    .exit_code(ExitCode::Verification)?;
            }
        }
    }

    if args.relink_only {
        match git_plan.as_ref() {
            Some(plan) if plan.kind == GitWorktreeKind::MainWorktree => {
                repair_main_worktree_after_copy(plan).exit_code(ExitCode::MoveOperation)?;
                verify_git_from_new_path(&old, &new).with_context(|| {
                    "Git worktree verification failed after relink-only repair; metadata was not changed"
                }).exit_code(ExitCode::Verification)?;
                if !output_mode.is_json() {
                    println!("Git worktree repair complete");
                }
            }
            Some(plan) if plan.kind == GitWorktreeKind::LinkedWorktree => {
                repair_linked_worktree_after_manual_move(plan)
                    .exit_code(ExitCode::MoveOperation)?;
                verify_git_from_new_path(&old, &new).with_context(|| {
                    "Git linked worktree verification failed after relink-only repair; metadata was not changed"
                }).exit_code(ExitCode::Verification)?;
                if !output_mode.is_json() {
                    println!("Git worktree repair complete");
                }
            }
            Some(_) | None => {}
        }
    }

    let changed =
        update_codex_home(&codex_home, &old_str, &new_str).exit_code(ExitCode::MetadataUpdate)?;
    if force_post_update_verification_failure_for_tests() {
        return Err(metadata_verification_error(
            "metadata verification failed: forced post-update verification failure for test",
            auto_rollback,
            &backup.manifest_path,
        ));
    }
    let remaining =
        scan_codex_home(&codex_home, &old_str, &new_str).exit_code(ExitCode::Verification)?;
    if remaining.old_reference_count() > 0 {
        return Err(metadata_verification_error(
            format!(
                "metadata verification failed: {} old-path reference(s) remain. Restore metadata with: codex-project-mover rollback --backup {}",
            remaining.old_reference_count(),
            backup.manifest_path.display()
            ),
            auto_rollback,
            &backup.manifest_path,
        ));
    }
    // Scan for new_str by passing it as the "old" argument — scan_codex_home finds exact
    // matches of its second argument, so swapping old/new here finds occurrences of new_str.
    let new_references =
        scan_codex_home(&codex_home, &new_str, &old_str).exit_code(ExitCode::Verification)?;
    if changed > 0 && new_references.old_reference_count() < changed {
        return Err(metadata_verification_error(
            format!(
                "metadata verification failed: expected at least {} new-path reference(s), found {}. Restore metadata with: codex-project-mover rollback --backup {}",
            changed,
            new_references.old_reference_count(),
            backup.manifest_path.display()
            ),
            auto_rollback,
            &backup.manifest_path,
        ));
    }

    if !args.relink_only
        && !matches!(
            git_plan.as_ref().map(|plan| &plan.kind),
            Some(GitWorktreeKind::LinkedWorktree)
        )
    {
        move_to_trash(&old).exit_code(ExitCode::MoveOperation)?;
    }

    let git_worktree_action = match git_plan.as_ref().map(|plan| &plan.kind) {
        Some(GitWorktreeKind::LinkedWorktree) if args.relink_only => "repair_linked_worktree",
        Some(GitWorktreeKind::LinkedWorktree) => "move_linked_worktree",
        Some(GitWorktreeKind::MainWorktree) => "repair_main_worktree",
        Some(GitWorktreeKind::NotGit) | None => "none",
    };
    let project_folder_action = if args.relink_only {
        "not_moved"
    } else if matches!(
        git_plan.as_ref().map(|plan| &plan.kind),
        Some(GitWorktreeKind::LinkedWorktree)
    ) {
        "moved_by_git"
    } else {
        "moved_to_trash"
    };

    if output_mode.is_json() {
        let output = ApplyOutput {
            command: "apply",
            status: "ok",
            old_path: old_str,
            new_path: new_str,
            codex_home: path_string(&codex_home),
            relink_only: args.relink_only,
            backup_dir: path_string(&backup.backup_dir),
            backup_manifest: path_string(&backup.manifest_path),
            changed_reference_count: changed,
            new_reference_count: new_references.old_reference_count(),
            git_worktree_action,
            project_folder_action,
            rollback: RollbackOutput::not_needed(),
        };
        return print_json(&output);
    }

    println!("metadata backup: {}", backup.backup_dir.display());
    println!("updated {} metadata reference(s)", changed);
    if args.relink_only {
        println!("relink-only complete: project folder was not moved");
    } else if matches!(
        git_plan.as_ref().map(|plan| &plan.kind),
        Some(GitWorktreeKind::LinkedWorktree)
    ) {
        println!("move complete: linked Git worktree moved with git worktree move");
    } else {
        println!("move complete: old project folder moved to Trash");
    }
    Ok(())
}

fn validate_normal_move(old: &std::path::Path, new: &std::path::Path) -> AppResult<()> {
    if !old.is_dir() {
        return Err(AppError::message(
            ExitCode::PathValidation,
            format!(
                "normal apply requires old path to exist as a directory: {}",
                old.display()
            ),
        ));
    }
    if new.exists() {
        return Err(AppError::message(
            ExitCode::PathValidation,
            format!(
                "normal apply requires new path to not exist: {}",
                new.display()
            ),
        ));
    }
    if new.starts_with(old) {
        return Err(AppError::message(
            ExitCode::PathValidation,
            format!(
                "normal apply requires new path to not be inside old path: {}",
                new.display()
            ),
        ));
    }
    Ok(())
}

fn validate_relink_only(old: &std::path::Path, new: &std::path::Path) -> AppResult<()> {
    if old.exists() {
        return Err(AppError::message(
            ExitCode::PathValidation,
            format!(
                "relink-only requires old path to not exist: {}",
                old.display()
            ),
        ));
    }
    if !new.is_dir() {
        return Err(AppError::message(
            ExitCode::PathValidation,
            format!(
                "relink-only requires new path to exist as a directory: {}",
                new.display()
            ),
        ));
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

fn metadata_verification_error(
    message: impl Into<String>,
    auto_rollback: bool,
    backup_manifest: &std::path::Path,
) -> AppError {
    let message = message.into();
    let rollback = if auto_rollback {
        attempt_automatic_rollback(backup_manifest)
    } else {
        RollbackOutput {
            status: "not_attempted",
            backup_manifest: Some(path_string(backup_manifest)),
            message: Some(
                "rerun with --auto-rollback to restore metadata automatically".to_string(),
            ),
        }
    };
    let rollback_message = match rollback.status {
        "succeeded" => "automatic rollback succeeded",
        "failed" => "automatic rollback failed",
        _ => "automatic rollback was not attempted",
    };
    AppError::message(
        ExitCode::Verification,
        format!("{message}; {rollback_message}"),
    )
    .with_details(serde_json::json!({ "rollback": rollback }))
}

fn attempt_automatic_rollback(backup_manifest: &std::path::Path) -> RollbackOutput {
    match restore_metadata_backup(backup_manifest) {
        Ok(manifest) => {
            if let Some(created_new_project_path) = manifest.created_new_project_path {
                if created_new_project_path.exists() {
                    if let Err(error) = move_to_trash(&created_new_project_path) {
                        return RollbackOutput {
                            status: "failed",
                            backup_manifest: Some(path_string(backup_manifest)),
                            message: Some(format!("{error:#}")),
                        };
                    }
                }
            }
            RollbackOutput {
                status: "succeeded",
                backup_manifest: Some(path_string(backup_manifest)),
                message: None,
            }
        }
        Err(error) => RollbackOutput {
            status: "failed",
            backup_manifest: Some(path_string(backup_manifest)),
            message: Some(format!("{error:#}")),
        },
    }
}

fn force_post_update_verification_failure_for_tests() -> bool {
    std::env::var("CODEX_PROJECT_MOVER_TEST_FORCE_POST_UPDATE_VERIFICATION_FAILURE").as_deref()
        == Ok("1")
}
