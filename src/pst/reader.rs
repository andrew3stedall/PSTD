use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::error::{PstdError, PstdResult};

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

#[derive(Debug)]
pub struct PstByteReader {
    input_path: PathBuf,
    file: File,
    file_size: u64,
}

impl PstByteReader {
    pub fn open(input_path: impl AsRef<Path>) -> PstdResult<Self> {
        let input_path = input_path.as_ref().to_path_buf();
        let file = File::open(&input_path)
            .map_err(|err| PstdError::SourceOpen(format!("{}: {err}", input_path.display())))?;
        let file_size = file.metadata()?.len();
        Ok(Self {
            input_path,
            file,
            file_size,
        })
    }

    pub fn input_path(&self) -> &Path {
        &self.input_path
    }

    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    pub fn read_at(&self, offset: u64, len: usize) -> PstdResult<Vec<u8>> {
        let len_u64 = len as u64;
        let end = offset
            .checked_add(len_u64)
            .ok_or_else(|| PstdError::pst_read(Some(offset), "offset plus length overflowed"))?;

        if end > self.file_size {
            return Err(PstdError::pst_read(
                Some(offset),
                format!(
                    "requested {} bytes ending at {}, beyond file size {}",
                    len, end, self.file_size
                ),
            ));
        }

        let mut file = self.file.try_clone()?;
        file.seek(SeekFrom::Start(offset))?;
        let mut buf = vec![0; len];
        file.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn read_prefix(&self, len: usize) -> PstdResult<Vec<u8>> {
        self.read_at(0, len.min(self.file_size as usize))
    }
}
