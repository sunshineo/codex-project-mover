use std::path::Path;

use anyhow::Result;

use crate::discovery::discover_state;
use crate::model::ScanReport;
use crate::surfaces::{
    automation_db::scan_automation_db, config_toml::scan_config_toml,
    global_state::scan_global_state, jsonl::scan_jsonl_file, sqlite_threads::scan_threads_db,
};

pub fn scan_codex_home(codex_home: &Path, old: &str, new: &str) -> Result<ScanReport> {
    let state = discover_state(codex_home)?;
    let mut report = ScanReport::default();

    for file in state.jsonl_files {
        report.matches.extend(scan_jsonl_file(&file, old, new)?);
    }

    for db in state.sqlite_state_dbs {
        report.matches.extend(scan_threads_db(&db, old, new)?);
    }

    if let Some(file) = state.global_state_json {
        report.matches.extend(scan_global_state(&file, old, new)?);
    }

    if let Some(file) = state.config_toml {
        report.matches.extend(scan_config_toml(&file, old, new)?);
    }

    if let Some(db) = state.automation_db {
        report.matches.extend(scan_automation_db(&db, old, new)?);
    }

    report.matches.sort();
    Ok(report)
}
