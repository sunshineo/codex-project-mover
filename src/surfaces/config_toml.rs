use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use toml_edit::{DocumentMut, Item, Table, Value};

use crate::model::{ReferenceMatch, SurfaceKind};

pub fn scan_config_toml(path: &Path, old: &str, new: &str) -> Result<Vec<ReferenceMatch>> {
    let document = read_document(path)?;
    let mut matches = Vec::new();
    collect_toml_matches(
        document.as_item(),
        old,
        new,
        path,
        String::new(),
        &mut matches,
    );
    collect_project_table_key_match(&document, old, new, path, &mut matches);
    collect_per_path_key_match(&document, old, new, path, &mut matches);
    matches.sort_by(|a, b| a.location.cmp(&b.location));
    Ok(matches)
}

pub fn update_config_toml(path: &Path, old: &str, new: &str) -> Result<usize> {
    let mut document = read_document(path)?;
    let mut changed = replace_toml_values(document.as_item_mut(), old, new);
    changed += rename_project_table_key(&mut document, old, new);
    changed += rename_per_path_key(&mut document, old, new);
    fs::write(path, document.to_string())
        .with_context(|| format!("write config TOML {}", path.display()))?;
    Ok(changed)
}

fn read_document(path: &Path) -> Result<DocumentMut> {
    let content =
        fs::read_to_string(path).with_context(|| format!("read config TOML {}", path.display()))?;
    content
        .parse::<DocumentMut>()
        .with_context(|| format!("parse config TOML {}", path.display()))
}

fn collect_toml_matches(
    item: &Item,
    old: &str,
    new: &str,
    file: &Path,
    location: String,
    matches: &mut Vec<ReferenceMatch>,
) {
    match item {
        Item::Value(value) if value.as_str() == Some(old) => matches.push(ReferenceMatch {
            surface: SurfaceKind::ConfigToml,
            file: file.to_path_buf(),
            location,
            old_value: old.to_string(),
            new_value: new.to_string(),
        }),
        Item::Table(table) => {
            for (key, child) in table.iter() {
                collect_toml_matches(
                    child,
                    old,
                    new,
                    file,
                    format!("{}/{}", location, key),
                    matches,
                );
            }
        }
        _ => {}
    }
}

fn replace_toml_values(item: &mut Item, old: &str, new: &str) -> usize {
    match item {
        Item::Value(value) if value.as_str() == Some(old) => {
            *value = Value::from(new);
            1
        }
        Item::Table(table) => table
            .iter_mut()
            .map(|(_, child)| replace_toml_values(child, old, new))
            .sum(),
        _ => 0,
    }
}

fn collect_project_table_key_match(
    document: &DocumentMut,
    old: &str,
    new: &str,
    file: &Path,
    matches: &mut Vec<ReferenceMatch>,
) {
    if table_at_path(document.as_item(), &["projects"])
        .and_then(|projects| projects.get(old))
        .is_some()
    {
        matches.push(ReferenceMatch {
            surface: SurfaceKind::ConfigToml,
            file: file.to_path_buf(),
            location: format!("/projects/{}", old),
            old_value: old.to_string(),
            new_value: new.to_string(),
        });
    }
}

fn collect_per_path_key_match(
    document: &DocumentMut,
    old: &str,
    new: &str,
    file: &Path,
    matches: &mut Vec<ReferenceMatch>,
) {
    if table_at_path(
        document.as_item(),
        &["desktop", "open-in-target-preferences", "perPath"],
    )
    .and_then(|per_path| per_path.get(old))
    .is_some()
    {
        matches.push(ReferenceMatch {
            surface: SurfaceKind::ConfigToml,
            file: file.to_path_buf(),
            location: format!("/desktop/open-in-target-preferences/perPath/{}", old),
            old_value: old.to_string(),
            new_value: new.to_string(),
        });
    }
}

fn rename_project_table_key(document: &mut DocumentMut, old: &str, new: &str) -> usize {
    let Some(projects) = table_at_path_mut(document.as_item_mut(), &["projects"]) else {
        return 0;
    };
    let Some(old_item) = projects.remove(old) else {
        return 0;
    };
    projects.insert(new, old_item);
    1
}

fn rename_per_path_key(document: &mut DocumentMut, old: &str, new: &str) -> usize {
    let Some(per_path) = table_at_path_mut(
        document.as_item_mut(),
        &["desktop", "open-in-target-preferences", "perPath"],
    ) else {
        return 0;
    };
    let Some(old_item) = per_path.remove(old) else {
        return 0;
    };
    per_path.insert(new, old_item);
    1
}

fn table_at_path<'a>(item: &'a Item, path: &[&str]) -> Option<&'a Table> {
    let mut table = item.as_table()?;
    for segment in path {
        table = table.get(segment)?.as_table()?;
    }
    Some(table)
}

fn table_at_path_mut<'a>(item: &'a mut Item, path: &[&str]) -> Option<&'a mut Table> {
    let mut table = item.as_table_mut()?;
    for segment in path {
        table = table.get_mut(segment)?.as_table_mut()?;
    }
    Some(table)
}
