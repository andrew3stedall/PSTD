use std::fs::{self, File};
use std::io::Cursor;
use std::path::{Path, PathBuf};

use tar::{Builder, Header};

use crate::error::{PstdError, PstdResult};
use crate::output::paths::archive_path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShardInfo {
    pub path: PathBuf,
    pub index: u64,
    pub bytes_written_estimate: u64,
}

pub struct TarShardWriter {
    output_dir: PathBuf,
    prefix: String,
    target_size_bytes: u64,
    current_index: u64,
    current_size: u64,
    current_path: PathBuf,
    builder: Builder<File>,
    shards: Vec<ShardInfo>,
}

impl TarShardWriter {
    pub fn new(output_dir: impl AsRef<Path>, prefix: impl Into<String>, target_size_bytes: u64) -> PstdResult<Self> {
        let output_dir = output_dir.as_ref().to_path_buf();
        fs::create_dir_all(&output_dir)?;
        let prefix = prefix.into();
        let current_index = 1;
        let current_path = output_dir.join(format!("{prefix}_{current_index:06}.tar"));
        let file = File::create(&current_path)?;

        Ok(Self {
            output_dir,
            prefix,
            target_size_bytes: target_size_bytes.max(1),
            current_index,
            current_size: 0,
            current_path,
            builder: Builder::new(file),
            shards: Vec::new(),
        })
    }

    pub fn append_bytes(&mut self, path_parts: &[impl AsRef<str>], bytes: &[u8]) -> PstdResult<()> {
        if self.current_size > 0 && self.current_size.saturating_add(bytes.len() as u64) > self.target_size_bytes {
            self.rotate()?;
        }

        let path = archive_path(path_parts);
        let mut header = Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        self.builder
            .append_data(&mut header, path, Cursor::new(bytes))
            .map_err(|err| PstdError::OutputWrite(err.to_string()))?;
        self.current_size = self.current_size.saturating_add(bytes.len() as u64);
        Ok(())
    }

    pub fn finish(mut self) -> PstdResult<Vec<ShardInfo>> {
        self.finish_current()?;
        Ok(self.shards)
    }

    fn rotate(&mut self) -> PstdResult<()> {
        self.finish_current()?;
        self.current_index += 1;
        self.current_size = 0;
        self.current_path = self.output_dir.join(format!("{}_{:06}.tar", self.prefix, self.current_index));
        let file = File::create(&self.current_path)?;
        self.builder = Builder::new(file);
        Ok(())
    }

    fn finish_current(&mut self) -> PstdResult<()> {
        self.builder.finish()?;
        if !self.shards.iter().any(|shard| shard.index == self.current_index) {
            self.shards.push(ShardInfo {
                path: self.current_path.clone(),
                index: self.current_index,
                bytes_written_estimate: self.current_size,
            });
        }
        Ok(())
    }
}
