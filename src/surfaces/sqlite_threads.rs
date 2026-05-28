use std::path::Path;

use anyhow::Result;
use rusqlite::{params, Connection};

use crate::model::{ReferenceMatch, SurfaceKind};

pub fn scan_threads_db(path: &Path, old: &str, new: &str) -> Result<Vec<ReferenceMatch>> {
    let conn = Connection::open(path)?;
    if !has_table_column(&conn, "threads", "cwd")? {
        return Ok(Vec::new());
    }

    let mut stmt = conn.prepare("SELECT id, cwd FROM threads WHERE cwd = ?1")?;
    let rows = stmt.query_map(params![old], |row| {
        let id: String = row.get(0)?;
        Ok(ReferenceMatch {
            surface: SurfaceKind::SqliteThreadsCwd,
            file: path.to_path_buf(),
            location: format!("threads.id={} cwd", id),
            old_value: old.to_string(),
            new_value: new.to_string(),
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn update_threads_db(path: &Path, old: &str, new: &str) -> Result<usize> {
    let conn = Connection::open(path)?;
    if !has_table_column(&conn, "threads", "cwd")? {
        return Ok(0);
    }

    let changed = conn.execute(
        "UPDATE threads SET cwd = ?1 WHERE cwd = ?2",
        params![new, old],
    )?;
    Ok(changed)
}

fn has_table_column(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let table_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        params![table],
        |row| row.get(0),
    )?;
    if table_count == 0 {
        return Ok(false);
    }

    let pragma = format!("PRAGMA table_info({})", quote_identifier(table));
    let mut stmt = conn.prepare(&pragma)?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for name in columns {
        if name? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}
