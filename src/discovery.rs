use std::path::{Path, PathBuf};

use anyhow::Result;
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CodexStateFiles {
    pub sqlite_state_dbs: Vec<PathBuf>,
    pub jsonl_files: Vec<PathBuf>,
    pub global_state_json: Option<PathBuf>,
    pub config_toml: Option<PathBuf>,
    pub automation_db: Option<PathBuf>,
}

pub fn discover_state(codex_home: &Path) -> Result<CodexStateFiles> {
    let mut files = CodexStateFiles::default();

    if codex_home.exists() {
        for entry in std::fs::read_dir(codex_home)? {
            let entry = entry?;
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if name.starts_with("state_") && name.ends_with(".sqlite") {
                files.sqlite_state_dbs.push(path);
            }
        }
    }

    for folder in ["sessions", "archived_sessions"] {
        let root = codex_home.join(folder);
        if root.exists() {
            for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
                    files.jsonl_files.push(path.to_path_buf());
                }
            }
        }
    }

    let global_state = codex_home.join(".codex-global-state.json");
    if global_state.exists() {
        files.global_state_json = Some(global_state);
    }

    let config = codex_home.join("config.toml");
    if config.exists() {
        files.config_toml = Some(config);
    }

    let automation_db = codex_home.join("sqlite/codex-dev.db");
    if automation_db.exists() {
        files.automation_db = Some(automation_db);
    }

    files.sqlite_state_dbs.sort();
    files.jsonl_files.sort();
    Ok(files)
}
