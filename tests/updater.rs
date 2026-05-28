use std::fs;

use codex_project_mover::scanner::scan_codex_home;
use codex_project_mover::updater::update_codex_home;
use tempfile::tempdir;

#[test]
fn updates_all_supported_metadata_files() {
    let temp = tempdir().unwrap();
    let home = temp.path().join(".codex");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::write(
        home.join("sessions/thread.jsonl"),
        r#"{"cwd":"/old/project"}"#,
    )
    .unwrap();
    fs::write(
        home.join(".codex-global-state.json"),
        r#"{"roots":["/old/project"]}"#,
    )
    .unwrap();
    fs::write(
        home.join("config.toml"),
        "[desktop]\nopen_target = \"/old/project\"\n",
    )
    .unwrap();

    let changed = update_codex_home(&home, "/old/project", "/new/project").unwrap();
    let remaining = scan_codex_home(&home, "/old/project", "/new/project").unwrap();

    assert_eq!(changed, 3);
    assert_eq!(remaining.old_reference_count(), 0);
}
