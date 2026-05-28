use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SurfaceKind {
    JsonlCwd,
    SqliteThreadsCwd,
    GlobalStateJson,
    ConfigToml,
    AutomationDbCwds,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ReferenceMatch {
    pub surface: SurfaceKind,
    pub file: PathBuf,
    pub location: String,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ScanReport {
    pub matches: Vec<ReferenceMatch>,
}

impl ScanReport {
    pub fn old_reference_count(&self) -> usize {
        self.matches.len()
    }
}
