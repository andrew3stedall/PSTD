use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ExtractConfig {
    pub input: PathBuf,
    pub output: PathBuf,
    pub continue_on_error: bool,
    pub overwrite: bool,
    pub manifest_only: bool,
    pub archive_format: String,
    pub data_format: String,
    pub tar_shard_size_mb: u64,
    pub progress: String,
    pub log_level: String,
    pub profile: String,
}

impl ExtractConfig {
    pub fn tar_shard_size_bytes(&self) -> u64 {
        self.tar_shard_size_mb.saturating_mul(1024).saturating_mul(1024)
    }
}
