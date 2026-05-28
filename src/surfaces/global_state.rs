use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::{Map, Value};

use crate::model::{ReferenceMatch, SurfaceKind};

pub fn scan_global_state(path: &Path, old: &str, new: &str) -> Result<Vec<ReferenceMatch>> {
    let value = read_json(path)?;
    let mut matches = Vec::new();
    collect_exact_string_and_key_matches(&value, old, new, path, String::new(), &mut matches);
    matches.sort_by(|a, b| a.location.cmp(&b.location));
    Ok(matches)
}

pub fn update_global_state(path: &Path, old: &str, new: &str) -> Result<usize> {
    let mut value = read_json(path)?;
    let changed = replace_exact_string_values_and_keys(&mut value, old, new);
    fs::write(path, serde_json::to_string_pretty(&value)? + "\n")
        .with_context(|| format!("write global state JSON {}", path.display()))?;
    Ok(changed)
}

fn read_json(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("read global state JSON {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("parse global state JSON {}", path.display()))
}

fn collect_exact_string_and_key_matches(
    value: &Value,
    old: &str,
    new: &str,
    file: &Path,
    pointer: String,
    matches: &mut Vec<ReferenceMatch>,
) {
    match value {
        Value::String(current) if current == old => matches.push(ReferenceMatch {
            surface: SurfaceKind::GlobalStateJson,
            file: file.to_path_buf(),
            location: if pointer.is_empty() {
                "/".to_string()
            } else {
                pointer
            },
            old_value: old.to_string(),
            new_value: new.to_string(),
        }),
        Value::Object(map) => {
            for (key, child) in map {
                let child_pointer = format!("{}/{}", pointer, escape_pointer(key));
                if key == old {
                    matches.push(ReferenceMatch {
                        surface: SurfaceKind::GlobalStateJson,
                        file: file.to_path_buf(),
                        location: child_pointer.clone(),
                        old_value: old.to_string(),
                        new_value: new.to_string(),
                    });
                }
                collect_exact_string_and_key_matches(child, old, new, file, child_pointer, matches);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect_exact_string_and_key_matches(
                    child,
                    old,
                    new,
                    file,
                    format!("{}/{}", pointer, index),
                    matches,
                );
            }
        }
        _ => {}
    }
}

fn replace_exact_string_values_and_keys(value: &mut Value, old: &str, new: &str) -> usize {
    match value {
        Value::String(current) if current == old => {
            *current = new.to_string();
            1
        }
        Value::Object(map) => replace_object_values_and_keys(map, old, new),
        Value::Array(items) => items
            .iter_mut()
            .map(|child| replace_exact_string_values_and_keys(child, old, new))
            .sum(),
        _ => 0,
    }
}

fn replace_object_values_and_keys(map: &mut Map<String, Value>, old: &str, new: &str) -> usize {
    let mut changed = map
        .values_mut()
        .map(|child| replace_exact_string_values_and_keys(child, old, new))
        .sum::<usize>();
    if let Some(item) = map.remove(old) {
        map.insert(new.to_string(), item);
        changed += 1;
    }
    changed
}

fn escape_pointer(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}
