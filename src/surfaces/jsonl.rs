use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::model::{ReferenceMatch, SurfaceKind};

pub fn scan_jsonl_file(path: &Path, old: &str, new: &str) -> Result<Vec<ReferenceMatch>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("read JSONL session file {}", path.display()))?;
    let mut matches = Vec::new();

    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line)
            .with_context(|| format!("parse JSONL line {} in {}", index + 1, path.display()))?;
        collect_cwd_matches(
            &value,
            old,
            new,
            path,
            format!("line {}", index + 1),
            String::new(),
            &mut matches,
        );
    }

    Ok(matches)
}

pub fn update_jsonl_file(path: &Path, old: &str, new: &str) -> Result<usize> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("read JSONL session file {}", path.display()))?;
    let mut changed_count = 0;
    let mut output = String::new();

    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            output.push('\n');
            continue;
        }
        let mut value: Value = serde_json::from_str(line)
            .with_context(|| format!("parse JSONL line {} in {}", index + 1, path.display()))?;
        changed_count += replace_cwd_values(&mut value, old, new);
        output.push_str(&serde_json::to_string(&value)?);
        output.push('\n');
    }

    fs::write(path, output)
        .with_context(|| format!("write JSONL session file {}", path.display()))?;
    Ok(changed_count)
}

fn collect_cwd_matches(
    value: &Value,
    old: &str,
    new: &str,
    file: &Path,
    line_label: String,
    pointer: String,
    matches: &mut Vec<ReferenceMatch>,
) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_pointer = format!("{}/{}", pointer, escape_pointer(key));
                if key == "cwd" && child.as_str() == Some(old) {
                    matches.push(ReferenceMatch {
                        surface: SurfaceKind::JsonlCwd,
                        file: file.to_path_buf(),
                        location: format!("{} {}", line_label, child_pointer),
                        old_value: old.to_string(),
                        new_value: new.to_string(),
                    });
                }
                collect_cwd_matches(
                    child,
                    old,
                    new,
                    file,
                    line_label.clone(),
                    child_pointer,
                    matches,
                );
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect_cwd_matches(
                    child,
                    old,
                    new,
                    file,
                    line_label.clone(),
                    format!("{}/{}", pointer, index),
                    matches,
                );
            }
        }
        _ => {}
    }
}

fn replace_cwd_values(value: &mut Value, old: &str, new: &str) -> usize {
    match value {
        Value::Object(map) => map
            .iter_mut()
            .map(|(key, child)| {
                let direct = if key == "cwd" && child.as_str() == Some(old) {
                    *child = Value::String(new.to_string());
                    1
                } else {
                    0
                };
                direct + replace_cwd_values(child, old, new)
            })
            .sum(),
        Value::Array(items) => items
            .iter_mut()
            .map(|child| replace_cwd_values(child, old, new))
            .sum(),
        _ => 0,
    }
}

fn escape_pointer(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}
