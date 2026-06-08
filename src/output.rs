use std::path::Path;

use serde::Serialize;

use crate::app_error::{AppError, AppResult, ExitCode};
use crate::model::ReferenceMatch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Human,
    Json,
}

impl OutputMode {
    pub fn from_json_flag(json: bool) -> Self {
        if json {
            Self::Json
        } else {
            Self::Human
        }
    }

    pub fn is_json(self) -> bool {
        self == Self::Json
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ReferenceOutput {
    pub surface: crate::model::SurfaceKind,
    pub file: String,
    pub location: String,
    pub old_value: String,
    pub new_value: String,
}

impl From<&ReferenceMatch> for ReferenceOutput {
    fn from(reference: &ReferenceMatch) -> Self {
        Self {
            surface: reference.surface.clone(),
            file: path_string(&reference.file),
            location: reference.location.clone(),
            old_value: reference.old_value.clone(),
            new_value: reference.new_value.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RollbackOutput {
    pub status: &'static str,
    pub backup_manifest: Option<String>,
    pub message: Option<String>,
}

impl RollbackOutput {
    pub fn not_needed() -> Self {
        Self {
            status: "not_needed",
            backup_manifest: None,
            message: None,
        }
    }
}

pub fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub fn print_json<T: Serialize>(value: &T) -> AppResult<()> {
    println!(
        "{}",
        serde_json::to_string(value)
            .map_err(|error| AppError::new(ExitCode::Unexpected, error.into()))?
    );
    Ok(())
}
