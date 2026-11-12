use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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
    crypt_method: OnceLock<u8>,
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
            crypt_method: OnceLock::new(),
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

        let mut buf = vec![0; len];
        read_exact_at(&self.file, &mut buf, offset)?;
        Ok(buf)
    }

    /// Returns the header crypt method, reading it with `load` only on first use.
    ///
    /// The value is a property of the opened file's header, so it is read once per
    /// reader instead of once per payload block. A failed `load` is not cached.
    /// This assumes the source file is not rewritten while the reader is open, as the
    /// already loaded NBT/BBT indexes also do.
    pub(crate) fn crypt_method_or_load(
        &self,
        load: impl FnOnce(&Self) -> PstdResult<u8>,
    ) -> PstdResult<u8> {
        if let Some(method) = self.crypt_method.get() {
            return Ok(*method);
        }
        let method = load(self)?;
        Ok(*self.crypt_method.get_or_init(|| method))
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

/// Positional read that neither duplicates the descriptor nor moves a shared cursor.
#[cfg(unix)]
fn read_exact_at(file: &File, buf: &mut [u8], offset: u64) -> std::io::Result<()> {
    std::os::unix::fs::FileExt::read_exact_at(file, buf, offset)
}

#[cfg(windows)]
fn read_exact_at(file: &File, mut buf: &mut [u8], mut offset: u64) -> std::io::Result<()> {
    use std::os::windows::fs::FileExt;
    while !buf.is_empty() {
        match file.seek_read(buf, offset) {
            Ok(0) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "failed to fill whole buffer",
                ))
            }
            Ok(read) => {
                buf = &mut buf[read..];
                offset += read as u64;
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn read_exact_at(file: &File, buf: &mut [u8], offset: u64) -> std::io::Result<()> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = file.try_clone()?;
    file.seek(SeekFrom::Start(offset))?;
    file.read_exact(buf)
}

#[cfg(test)]
mod tests {
    use super::PstByteReader;

    #[test]
    fn positional_reads_are_independent_and_bounded() {
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), b"0123456789").unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();

        assert_eq!(reader.read_at(6, 4).unwrap(), b"6789");
        assert_eq!(reader.read_at(0, 3).unwrap(), b"012");
        assert_eq!(reader.read_at(6, 4).unwrap(), b"6789");
        assert_eq!(reader.read_at(10, 0).unwrap(), b"");
        assert!(reader.read_at(7, 4).is_err());
        assert!(reader.read_at(u64::MAX, 1).is_err());
    }
}
