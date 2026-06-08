use serde::Serialize;

use crate::app_error::{AppResult, ExitCode, ResultExitCodeExt};
use crate::backup::restore_metadata_backup;
use crate::cli::RollbackArgs;
use crate::output::{path_string, print_json, OutputMode};
use crate::process_guard::assert_no_codex_processes;
use crate::trash::move_to_trash;

#[derive(Debug, Serialize)]
struct RollbackCommandOutput {
    command: &'static str,
    status: &'static str,
    backup_manifest: String,
    removed_created_new_project_path: Option<String>,
}

pub fn run(args: RollbackArgs, output_mode: OutputMode) -> AppResult<()> {
    assert_no_codex_processes(args.allow_running_codex).exit_code(ExitCode::ProcessGuard)?;
    let backup_manifest = if args.backup.is_dir() {
        args.backup.join("manifest.json")
    } else {
        args.backup
    };
    let manifest = restore_metadata_backup(&backup_manifest).exit_code(ExitCode::Rollback)?;
    let mut removed_created_new_project_path = None;
    if let Some(created_new_project_path) = manifest.created_new_project_path {
        if created_new_project_path.exists() {
            move_to_trash(&created_new_project_path).exit_code(ExitCode::Rollback)?;
            removed_created_new_project_path = Some(path_string(&created_new_project_path));
            if !output_mode.is_json() {
                println!(
                    "removed created new project folder: {}",
                    created_new_project_path.display()
                );
            }
        }
    }
    if output_mode.is_json() {
        let output = RollbackCommandOutput {
            command: "rollback",
            status: "ok",
            backup_manifest: path_string(&backup_manifest),
            removed_created_new_project_path,
        };
        return print_json(&output);
    }
    println!("metadata rollback complete");
    println!("old project folder was not restored from Trash");
    Ok(())
}
