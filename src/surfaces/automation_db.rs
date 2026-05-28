use std::path::Path;

use anyhow::Result;
use rusqlite::{params, Connection};
use serde_json::Value;

use crate::model::{ReferenceMatch, SurfaceKind};

pub fn scan_automation_db(path: &Path, old: &str, new: &str) -> Result<Vec<ReferenceMatch>> {
    let conn = Connection::open(path)?;
    let mut matches = Vec::new();
    for table in tables_with_cwds(&conn)? {
        let sql = format!(
            "SELECT id, cwds FROM {} WHERE cwds IS NOT NULL",
            quote_identifier(&table)
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (id, cwds) = row?;
            collect_cwds_matches(path, &table, &id, &cwds, old, new, &mut matches);
        }
    }
    matches.sort_by(|a, b| a.location.cmp(&b.location));
    Ok(matches)
}

pub fn update_automation_db(path: &Path, old: &str, new: &str) -> Result<usize> {
    let conn = Connection::open(path)?;
    let mut changed = 0;
    for table in tables_with_cwds(&conn)? {
        let select_sql = format!(
            "SELECT id, cwds FROM {} WHERE cwds IS NOT NULL",
            quote_identifier(&table)
        );
        let rows = {
            let mut stmt = conn.prepare(&select_sql)?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            rows
        };
        for (id, cwds) in rows {
            if let Some(updated) = replace_cwds_value(&cwds, old, new) {
                let update_sql = format!(
                    "UPDATE {} SET cwds = ?1 WHERE id = ?2",
                    quote_identifier(&table)
                );
                conn.execute(&update_sql, params![updated, id])?;
                changed += 1;
            }
        }
    }
    Ok(changed)
}

fn collect_cwds_matches(
    file: &Path,
    table: &str,
    id: &str,
    cwds: &str,
    old: &str,
    new: &str,
    matches: &mut Vec<ReferenceMatch>,
) {
    if cwds == old {
        matches.push(match_for(file, table, id, "cwds", old, new));
        return;
    }

    let Ok(Value::Array(items)) = serde_json::from_str::<Value>(cwds) else {
        return;
    };
    for (index, item) in items.iter().enumerate() {
        if item.as_str() == Some(old) {
            matches.push(match_for(
                file,
                table,
                id,
                &format!("cwds[{}]", index),
                old,
                new,
            ));
        }
    }
}

fn replace_cwds_value(cwds: &str, old: &str, new: &str) -> Option<String> {
    if cwds == old {
        return Some(new.to_string());
    }
    let Ok(Value::Array(mut items)) = serde_json::from_str::<Value>(cwds) else {
        return None;
    };
    let mut changed = false;
    for item in &mut items {
        if item.as_str() == Some(old) {
            *item = Value::String(new.to_string());
            changed = true;
        }
    }
    changed.then(|| serde_json::to_string(&items).expect("serialize cwd array"))
}

fn tables_with_cwds(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type = 'table'")?;
    let names = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let mut tables = Vec::new();
    for name in names {
        let name = name?;
        let pragma = format!("PRAGMA table_info({})", quote_identifier(&name));
        let mut column_stmt = conn.prepare(&pragma)?;
        let columns = column_stmt.query_map([], |row| row.get::<_, String>(1))?;
        for column in columns {
            if column? == "cwds" {
                tables.push(name.clone());
                break;
            }
        }
    }
    Ok(tables)
}

fn match_for(
    file: &Path,
    table: &str,
    id: &str,
    field: &str,
    old: &str,
    new: &str,
) -> ReferenceMatch {
    ReferenceMatch {
        surface: SurfaceKind::AutomationDbCwds,
        file: file.to_path_buf(),
        location: format!("{}.id={} {}", table, id, field),
        old_value: old.to_string(),
        new_value: new.to_string(),
    }
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}
