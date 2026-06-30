use crate::error::{PstdError, PstdResult};
use crate::pst::binary::{slice_at, u16_le_at, u32_le_at, u8_at};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BthHeader {
    pub key_size: u8,
    pub value_size: u8,
    pub entry_count: u16,
    pub root_allocation: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BthEntry {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BthMap {
    pub header: BthHeader,
    pub entries: Vec<BthEntry>,
}

impl BthMap {
    pub fn parse(buf: &[u8], base_offset: u64) -> PstdResult<Self> {
        if buf.len() < 8 {
            return Err(PstdError::pst_parse(
                Some(base_offset),
                "BTH buffer too short",
            ));
        }
        let header = BthHeader {
            key_size: u8_at(buf, 0, base_offset)?,
            value_size: u8_at(buf, 1, base_offset)?,
            entry_count: u16_le_at(buf, 2, base_offset)?,
            root_allocation: u32_le_at(buf, 4, base_offset)?,
        };
        let entry_size = header.key_size as usize + header.value_size as usize;
        if entry_size == 0 {
            return Err(PstdError::pst_parse(
                Some(base_offset),
                "BTH entry size is zero",
            ));
        }

        let mut entries = Vec::new();
        let mut cursor = 8usize;
        for _ in 0..header.entry_count {
            if cursor + entry_size > buf.len() {
                break;
            }
            let key = slice_at(buf, cursor, header.key_size as usize, base_offset)?.to_vec();
            let value = slice_at(
                buf,
                cursor + header.key_size as usize,
                header.value_size as usize,
                base_offset,
            )?
            .to_vec();
            entries.push(BthEntry { key, value });
            cursor += entry_size;
        }
        Ok(Self { header, entries })
    }

    pub fn lookup(&self, key: &[u8]) -> Option<&[u8]> {
        self.entries
            .iter()
            .find(|entry| entry.key.as_slice() == key)
            .map(|entry| entry.value.as_slice())
    }
}
