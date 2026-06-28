use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PstReaderConfig {
    pub input_path: PathBuf,
}

impl PstReaderConfig {
    pub fn new(input_path: impl AsRef<Path>) -> Self {
        Self {
            input_path: input_path.as_ref().to_path_buf(),
        }
    }
}
