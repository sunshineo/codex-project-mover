use std::fmt;

use anyhow::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitCode {
    Success = 0,
    Unexpected = 1,
    InvalidArguments = 2,
    ProcessGuard = 3,
    PathValidation = 4,
    Backup = 5,
    MoveOperation = 6,
    MetadataUpdate = 7,
    Verification = 8,
    Rollback = 9,
}

impl ExitCode {
    pub fn code(self) -> i32 {
        self as i32
    }
}

#[derive(Debug)]
pub struct AppError {
    exit_code: ExitCode,
    source: Error,
    details: Option<Value>,
}

impl AppError {
    pub fn new(exit_code: ExitCode, source: Error) -> Self {
        Self {
            exit_code,
            source,
            details: None,
        }
    }

    pub fn message(exit_code: ExitCode, message: impl Into<String>) -> Self {
        Self::new(exit_code, anyhow::anyhow!(message.into()))
    }

    pub fn exit_code(&self) -> ExitCode {
        self.exit_code
    }

    pub fn source_error(&self) -> &Error {
        &self.source
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn details(&self) -> Option<&Value> {
        self.details.as_ref()
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:#}", self.source)
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.source()
    }
}

pub trait ResultExitCodeExt<T> {
    fn exit_code(self, exit_code: ExitCode) -> AppResult<T>;
}

impl<T> ResultExitCodeExt<T> for anyhow::Result<T> {
    fn exit_code(self, exit_code: ExitCode) -> AppResult<T> {
        self.map_err(|error| AppError::new(exit_code, error))
    }
}
