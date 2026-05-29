use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::backup::create_metadata_backup;
use crate::cli::ApplyArgs;
use crate::git_worktree::{
    build_plan_for_existing_project, move_linked_worktree, repair_main_worktree_after_copy,
    verify_git_from_new_path, GitWorktreeKind,
};
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

    let git_plan = if args.relink_only {
        None
    } else {
        Some(build_plan_for_existing_project(&old, &new)?)
    };

    if !args.relink_only {
        match git_plan.as_ref() {
            Some(plan) if plan.kind == GitWorktreeKind::LinkedWorktree => {
                move_linked_worktree(plan)?;
                verify_git_from_new_path(&old, &new).with_context(|| {
                    "Git worktree verification failed after move; metadata was not changed"
                })?;
                println!("Git worktree move complete");
            }
            Some(plan) if plan.kind == GitWorktreeKind::MainWorktree => {
                copy_project_tree(&old, &new)?;
                verify_project_tree(&old, &new)
                    .with_context(|| "copied project tree failed verification; metadata was not changed and old folder was not moved to Trash")?;
                repair_main_worktree_after_copy(plan)?;
                verify_git_from_new_path(&old, &new).with_context(|| {
                    "Git worktree verification failed after repair; metadata was not changed and old folder was not moved to Trash"
                })?;
                println!("Git worktree repair complete");
            }
            Some(_) | None => {
                copy_project_tree(&old, &new)?;
                verify_project_tree(&old, &new)
                    .with_context(|| "copied project tree failed verification; metadata was not changed and old folder was not moved to Trash")?;
            }
        }
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
    // Scan for new_str by passing it as the "old" argument — scan_codex_home finds exact
    // matches of its second argument, so swapping old/new here finds occurrences of new_str.
    let new_references = scan_codex_home(&codex_home, &new_str, &old_str)?;
    if changed > 0 && new_references.old_reference_count() < changed {
        bail!(
            "metadata verification failed: expected at least {} new-path reference(s), found {}. Restore metadata with: codex-project-mover rollback --backup {}",
            changed,
            new_references.old_reference_count(),
            backup.manifest_path.display()
        );
    }

    if !args.relink_only
        && !matches!(
            git_plan.as_ref().map(|plan| &plan.kind),
            Some(GitWorktreeKind::LinkedWorktree)
        )
    {
        move_to_trash(&old)?;
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

fn validate_normal_move(old: &std::path::Path, new: &std::path::Path) -> Result<()> {
    if !old.is_dir() {
        bail!(
            "normal apply requires old path to exist as a directory: {}",
            old.display()
        );
    }
    if new.exists() {
        bail!(
            "normal apply requires new path to not exist: {}",
            new.display()
        );
    }
    if new.starts_with(old) {
        bail!(
            "normal apply requires new path to not be inside old path: {}",
            new.display()
        );
    }
    Ok(())
}

fn validate_relink_only(old: &std::path::Path, new: &std::path::Path) -> Result<()> {
    if old.exists() {
        bail!(
            "relink-only requires old path to not exist: {}",
            old.display()
        );
    }
    if !new.is_dir() {
        bail!(
            "relink-only requires new path to exist as a directory: {}",
            new.display()
        );
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
