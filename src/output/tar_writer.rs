use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use tar::{Builder, Header};

use crate::error::{PstdError, PstdResult};
use crate::output::paths::{archive_path, archive_path_preserve_hidden, validate_archive_path};

/// Archive entries are a 512-byte header, the data and padding; buffering turns those
/// three or more writes per entry into one write per buffer.
const TAR_WRITE_BUFFER_BYTES: usize = 256 * 1024;

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
    current_temp_path: PathBuf,
    builder: Option<Builder<BufWriter<File>>>,
    shards: Vec<ShardInfo>,
}

impl TarShardWriter {
    pub fn new(
        output_dir: impl AsRef<Path>,
        prefix: impl Into<String>,
        target_size_bytes: u64,
    ) -> PstdResult<Self> {
        let output_dir = output_dir.as_ref().to_path_buf();
        fs::create_dir_all(&output_dir)?;
        let prefix = prefix.into();
        let current_index = 1;
        let current_path = output_dir.join(format!("{prefix}_{current_index:06}.tar"));
        let current_temp_path = output_dir.join(format!("{prefix}_{current_index:06}.tar.part"));
        let file = File::create(&current_temp_path)?;

        Ok(Self {
            output_dir,
            prefix,
            target_size_bytes: target_size_bytes.max(1),
            current_index,
            current_size: 0,
            current_path,
            current_temp_path,
            builder: Some(Builder::new(BufWriter::with_capacity(
                TAR_WRITE_BUFFER_BYTES,
                file,
            ))),
            shards: Vec::new(),
        })
    }

    pub fn append_bytes(&mut self, path_parts: &[impl AsRef<str>], bytes: &[u8]) -> PstdResult<()> {
        self.append_path(archive_path(path_parts), bytes)
    }

    pub fn append_bytes_preserve_hidden(
        &mut self,
        path_parts: &[impl AsRef<str>],
        bytes: &[u8],
    ) -> PstdResult<()> {
        self.append_path(archive_path_preserve_hidden(path_parts), bytes)
    }

    fn append_path(&mut self, path: PathBuf, bytes: &[u8]) -> PstdResult<()> {
        validate_archive_path(&path)?;
        if self.current_size > 0
            && self.current_size.saturating_add(bytes.len() as u64) > self.target_size_bytes
        {
            self.rotate()?;
        }

        let mut header = Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        self.builder
            .as_mut()
            .expect("tar builder is active")
            .append_data(&mut header, path, bytes)
            .map_err(|err| PstdError::OutputWrite(err.to_string()))?;
        self.current_size = self.current_size.saturating_add(bytes.len() as u64);
        Ok(())
    }

    pub fn finish(mut self) -> PstdResult<Vec<ShardInfo>> {
        self.finish_current()?;
        Ok(std::mem::take(&mut self.shards))
    }

    fn rotate(&mut self) -> PstdResult<()> {
        self.finish_current()?;
        self.current_index += 1;
        self.current_size = 0;
        self.current_path = self
            .output_dir
            .join(format!("{}_{:06}.tar", self.prefix, self.current_index));
        self.current_temp_path = self.output_dir.join(format!(
            "{}_{:06}.tar.part",
            self.prefix, self.current_index
        ));
        let file = File::create(&self.current_temp_path)?;
        self.builder = Some(Builder::new(BufWriter::with_capacity(
            TAR_WRITE_BUFFER_BYTES,
            file,
        )));
        Ok(())
    }

    fn finish_current(&mut self) -> PstdResult<()> {
        if let Some(builder) = self.builder.take() {
            // `into_inner` writes the end-of-archive blocks; the explicit flush surfaces
            // a write error instead of losing it when the buffer is dropped.
            let written = builder.into_inner().and_then(|mut file| file.flush());
            if let Err(err) = written {
                // Buffered writes can fail only here, after the builder is gone, so the
                // partial shard is removed here rather than in `Drop`.
                let _ = fs::remove_file(&self.current_temp_path);
                return Err(err.into());
            }
            fs::rename(&self.current_temp_path, &self.current_path)?;
        }
        if !self
            .shards
            .iter()
            .any(|shard| shard.index == self.current_index)
        {
            self.shards.push(ShardInfo {
                path: self.current_path.clone(),
                index: self.current_index,
                bytes_written_estimate: self.current_size,
            });
        }
        Ok(())
    }
}

impl Drop for TarShardWriter {
    fn drop(&mut self) {
        if self.builder.is_some() {
            let _ = fs::remove_file(&self.current_temp_path);
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::TarShardWriter;

    #[test]
    fn deferred_write_failure_removes_the_partial_shard() {
        let full = std::path::Path::new("/dev/full");
        if !full.exists() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let part = dir.path().join("pst_000001.tar.part");
        // The shard is created through this link, so every flushed write fails.
        std::os::unix::fs::symlink(full, &part).unwrap();

        let mut writer = TarShardWriter::new(dir.path(), "pst", 1024).unwrap();
        writer.append_bytes(&["data", "a.txt"], b"data").unwrap();
        assert!(writer.finish().is_err());

        assert!(std::fs::symlink_metadata(&part).is_err());
        assert!(!dir.path().join("pst_000001.tar").exists());
    }
}
