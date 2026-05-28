use std::fs;

use codex_project_mover::scanner::scan_codex_home;
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn scanner_combines_matches_from_supported_surfaces() {
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

    let db = home.join("state_main.sqlite");
    let conn = Connection::open(&db).unwrap();
    conn.execute("CREATE TABLE threads (id TEXT PRIMARY KEY, cwd TEXT)", [])
        .unwrap();
    conn.execute(
        "INSERT INTO threads (id, cwd) VALUES ('a', '/old/project')",
        [],
    )
    .unwrap();

    let report = scan_codex_home(&home, "/old/project", "/new/project").unwrap();

    assert_eq!(report.old_reference_count(), 4);
}
