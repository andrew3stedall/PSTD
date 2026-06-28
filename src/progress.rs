#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressEventType {
    RunStarted,
    PstStarted,
    ArchiveWritten,
    RunFinished,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProgressEvent {
    pub run_id: String,
    pub event_type: ProgressEventType,
    pub timestamp_utc: String,
    pub source_pst: Option<String>,
    pub messages_discovered: u64,
    pub messages_extracted: u64,
    pub messages_failed: u64,
    pub attachments_extracted: u64,
    pub bytes_written: u64,
    pub message: String,
}

impl ProgressEvent {
    pub fn new(run_id: impl Into<String>, event_type: ProgressEventType, message: impl Into<String>) -> Self {
        Self {
            run_id: run_id.into(),
            event_type,
            timestamp_utc: chrono::Utc::now().to_rfc3339(),
            source_pst: None,
            messages_discovered: 0,
            messages_extracted: 0,
            messages_failed: 0,
            attachments_extracted: 0,
            bytes_written: 0,
            message: message.into(),
        }
    }

    pub fn to_json_line(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self).map(|mut line| {
            line.push('\n');
            line
        })
    }
}
