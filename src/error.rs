use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PstdError {
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("source could not be opened: {0}")]
    SourceOpen(String),

    #[error("output could not be written: {0}")]
    OutputWrite(String),

    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("json serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type PstdResult<T> = Result<T, PstdError>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusSeverity {
    Info,
    Warning,
    Recoverable,
    Fatal,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusScope {
    Run,
    Pst,
    Folder,
    Message,
    Attachment,
    Body,
    Metadata,
    Output,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StatusRecord {
    pub run_id: String,
    pub source_pst: Option<String>,
    pub scope: StatusScope,
    pub severity: StatusSeverity,
    pub code: String,
    pub message: String,
    pub folder_key: Option<String>,
    pub message_key: Option<String>,
    pub attachment_key: Option<String>,
    pub timestamp_utc: String,
    pub recoverable: bool,
    pub raw_offset: Option<u64>,
}

impl StatusRecord {
    pub fn info(run_id: impl Into<String>, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            run_id: run_id.into(),
            source_pst: None,
            scope: StatusScope::Run,
            severity: StatusSeverity::Info,
            code: code.into(),
            message: message.into(),
            folder_key: None,
            message_key: None,
            attachment_key: None,
            timestamp_utc: chrono::Utc::now().to_rfc3339(),
            recoverable: true,
            raw_offset: None,
        }
    }
}
