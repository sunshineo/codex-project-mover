use codex_project_mover::surfaces::automation_db::{scan_automation_db, update_automation_db};
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn updates_cwds_json_array_elements_equal_to_old_path() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("codex-dev.db");
    let conn = Connection::open(&db).unwrap();
    conn.execute(
        "CREATE TABLE automations (id TEXT PRIMARY KEY, cwds TEXT)",
        [],
    )
    .unwrap();
    conn.execute(
        r#"INSERT INTO automations (id, cwds) VALUES ('a', '["/old/project","/old/project/subdir"]')"#,
        [],
    )
    .unwrap();

    let matches = scan_automation_db(&db, "/old/project", "/new/project").unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].location, "automations.id=a cwds[0]");

    let changed = update_automation_db(&db, "/old/project", "/new/project").unwrap();
    assert_eq!(changed, 1);

    let updated: String = conn
        .query_row("SELECT cwds FROM automations WHERE id = 'a'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(updated, r#"["/new/project","/old/project/subdir"]"#);
}
