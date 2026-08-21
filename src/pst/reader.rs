use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::error::{PstdError, PstdResult};
use crate::pst::limits::InputLimits;

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
    limits: InputLimits,
}

impl PstByteReader {
    pub fn open(input_path: impl AsRef<Path>) -> PstdResult<Self> {
        Self::open_with_limits(input_path, &InputLimits::default())
    }

    pub fn open_with_limits(
        input_path: impl AsRef<Path>,
        limits: &InputLimits,
    ) -> PstdResult<Self> {
        let input_path = input_path.as_ref().to_path_buf();
        let file = File::open(&input_path)
            .map_err(|err| PstdError::SourceOpen(format!("{}: {err}", input_path.display())))?;
        let file_size = file.metadata()?.len();
        if file_size > limits.max_file_bytes {
            return Err(PstdError::pst_read(
                Some(0),
                format!(
                    "input file size {} exceeds max_file_bytes {}",
                    file_size, limits.max_file_bytes
                ),
            ));
        }
        Ok(Self {
            input_path,
            file,
            file_size,
            limits: *limits,
        })
    }

    pub fn input_path(&self) -> &Path {
        &self.input_path
    }

    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    pub fn limits(&self) -> InputLimits {
        self.limits
    }

    pub fn read_at(&self, offset: u64, len: usize) -> PstdResult<Vec<u8>> {
        let len_u64 = len as u64;
        if len_u64 > self.limits.max_single_read_bytes as u64 {
            return Err(PstdError::pst_read(
                Some(offset),
                format!(
                    "requested read of {} bytes exceeds max_single_read_bytes {}",
                    len, self.limits.max_single_read_bytes
                ),
            ));
        }
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
        let bounded_by_file = self.file_size.min(len as u64);
        let bounded_by_limit = bounded_by_file.min(self.limits.max_single_read_bytes as u64);
        let bounded_len = usize::try_from(bounded_by_limit).map_err(|_| {
            PstdError::pst_read(Some(0), "prefix length does not fit in platform usize")
        })?;
        self.read_at(0, bounded_len)
    }
}
