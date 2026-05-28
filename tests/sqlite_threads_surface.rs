use codex_project_mover::surfaces::sqlite_threads::{scan_threads_db, update_threads_db};
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn scans_and_updates_only_threads_cwd_equal_to_old_path() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("state_test.sqlite");
    let conn = Connection::open(&db).unwrap();
    conn.execute("CREATE TABLE threads (id TEXT PRIMARY KEY, cwd TEXT)", [])
        .unwrap();
    conn.execute(
        "INSERT INTO threads (id, cwd) VALUES ('a', '/old/project')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO threads (id, cwd) VALUES ('b', '/old/project/subdir')",
        [],
    )
    .unwrap();

    let matches = scan_threads_db(&db, "/old/project", "/new/project").unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].location, "threads.id=a cwd");

    let changed = update_threads_db(&db, "/old/project", "/new/project").unwrap();
    assert_eq!(changed, 1);

    let updated: String = conn
        .query_row("SELECT cwd FROM threads WHERE id = 'a'", [], |row| {
            row.get(0)
        })
        .unwrap();
    let untouched: String = conn
        .query_row("SELECT cwd FROM threads WHERE id = 'b'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(updated, "/new/project");
    assert_eq!(untouched, "/old/project/subdir");
}
