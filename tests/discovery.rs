use std::fs;

use codex_project_mover::discovery::discover_state;
use tempfile::tempdir;

#[test]
fn discovers_supported_codex_state_files() {
    let temp = tempdir().unwrap();
    let codex_home = temp.path();
    fs::create_dir_all(codex_home.join("sessions/2026")).unwrap();
    fs::create_dir_all(codex_home.join("archived_sessions/2025")).unwrap();
    fs::create_dir_all(codex_home.join("sqlite")).unwrap();
    fs::write(codex_home.join("state_main.sqlite"), "").unwrap();
    fs::write(codex_home.join("sessions/2026/thread.jsonl"), "").unwrap();
    fs::write(codex_home.join("archived_sessions/2025/thread.jsonl"), "").unwrap();
    fs::write(codex_home.join(".codex-global-state.json"), "{}").unwrap();
    fs::write(codex_home.join("config.toml"), "").unwrap();
    fs::write(codex_home.join("sqlite/codex-dev.db"), "").unwrap();

    let state = discover_state(codex_home).unwrap();

    assert_eq!(state.sqlite_state_dbs.len(), 1);
    assert_eq!(state.jsonl_files.len(), 2);
    assert!(state.global_state_json.is_some());
    assert!(state.config_toml.is_some());
    assert!(state.automation_db.is_some());
}
