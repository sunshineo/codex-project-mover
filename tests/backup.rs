use std::fs;

use codex_project_mover::backup::{create_metadata_backup, restore_metadata_backup};
use tempfile::tempdir;

#[test]
fn backs_up_and_restores_metadata_files() {
    let temp = tempdir().unwrap();
    let codex_home = temp.path().join(".codex");
    let backups_root = codex_home.join("codex-project-mover-backups");
    fs::create_dir_all(&codex_home).unwrap();
    let metadata = codex_home.join("config.toml");
    fs::write(&metadata, "before").unwrap();

    let backup = create_metadata_backup(
        &backups_root,
        "/old/project",
        "/new/project",
        None,
        std::slice::from_ref(&metadata),
    )
    .unwrap();

    fs::write(&metadata, "after").unwrap();
    restore_metadata_backup(&backup.manifest_path).unwrap();

    assert_eq!(fs::read_to_string(&metadata).unwrap(), "before");
    assert!(backup.manifest_path.exists());
}
