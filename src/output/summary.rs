#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractionSummary {
    pub run_id: String,
    pub pst_id: String,
    pub source_pst_path: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub duration_seconds: Option<f64>,
    pub folders_discovered: u64,
    pub messages_discovered: u64,
    pub messages_extracted: u64,
    pub messages_not_extracted: u64,
    pub attachments_extracted: u64,
    pub attachments_not_extracted: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub tar_shards_written: u64,
    pub status: String,
}
