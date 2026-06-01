use anyhow::Result;

use crate::backup::restore_metadata_backup;
use crate::cli::RollbackArgs;
use crate::process_guard::assert_no_codex_processes;
use crate::trash::move_to_trash;

pub fn run(args: RollbackArgs) -> Result<()> {
    assert_no_codex_processes(args.allow_running_codex)?;
    let backup_manifest = if args.backup.is_dir() {
        args.backup.join("manifest.json")
    } else {
        args.backup
    };
    let manifest = restore_metadata_backup(&backup_manifest)?;
    if let Some(created_new_project_path) = manifest.created_new_project_path {
        if created_new_project_path.exists() {
            move_to_trash(&created_new_project_path)?;
            println!(
                "removed created new project folder: {}",
                created_new_project_path.display()
            );
        }
    }
    println!("metadata rollback complete");
    println!("old project folder was not restored from Trash");
    Ok(())
}
