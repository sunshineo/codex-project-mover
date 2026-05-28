use std::path::Path;

use anyhow::Result;

use crate::discovery::discover_state;
use crate::surfaces::{
    automation_db::update_automation_db, config_toml::update_config_toml,
    global_state::update_global_state, jsonl::update_jsonl_file, sqlite_threads::update_threads_db,
};

pub fn update_codex_home(codex_home: &Path, old: &str, new: &str) -> Result<usize> {
    let state = discover_state(codex_home)?;
    let mut changed = 0;

    for file in state.jsonl_files {
        changed += update_jsonl_file(&file, old, new)?;
    }

    for db in state.sqlite_state_dbs {
        changed += update_threads_db(&db, old, new)?;
    }

    if let Some(file) = state.global_state_json {
        changed += update_global_state(&file, old, new)?;
    }

    if let Some(file) = state.config_toml {
        changed += update_config_toml(&file, old, new)?;
    }

    if let Some(db) = state.automation_db {
        changed += update_automation_db(&db, old, new)?;
    }

    Ok(changed)
}
