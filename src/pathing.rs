use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};

pub fn normalize_project_path(path: PathBuf) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .context("read current directory")?
            .join(path)
    };

    Ok(clean_path(&absolute))
}

pub fn codex_home_from_arg(path: Option<PathBuf>) -> Result<PathBuf> {
    match path {
        Some(path) => Ok(clean_path(&path)),
        None => dirs::home_dir()
            .map(|home| clean_path(&home.join(".codex")))
            .context("could not determine home directory for default ~/.codex"),
    }
}

fn clean_path(path: &Path) -> PathBuf {
    let mut cleaned = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                cleaned.pop();
            }
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
                cleaned.push(component.as_os_str());
            }
        }
    }
    cleaned
}
