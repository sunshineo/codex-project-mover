use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

pub fn create_codex_home(root: &Path) -> PathBuf {
    let home = root.join(".codex");
    fs::create_dir_all(home.join("sessions")).unwrap();
    fs::create_dir_all(home.join("archived_sessions")).unwrap();
    fs::create_dir_all(home.join("sqlite")).unwrap();
    home
}

pub fn create_threads_db(path: &Path, old: &str) {
    let conn = Connection::open(path).unwrap();
    conn.execute("CREATE TABLE threads (id TEXT PRIMARY KEY, cwd TEXT)", [])
        .unwrap();
    conn.execute("INSERT INTO threads (id, cwd) VALUES ('t1', ?1)", [old])
        .unwrap();
}
