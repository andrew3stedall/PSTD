use crate::error::{PstdError, PstdResult};
use crate::pst::binary::{u16_le_at, u64_le_at};
use crate::pst::primitives::{ByteOffset, PageRef, PstVariant, RootPointers};
use crate::pst::reader::PstByteReader;

pub const PST_HEADER_MIN_BYTES: usize = 64;
pub const PST_MAGIC: [u8; 4] = [0x21, 0x42, 0x44, 0x4e];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PstHeaderSummary {
    pub format: String,
    pub parser_status: String,
    pub file_size: u64,
    pub magic: String,
    pub magic_client: Option<String>,
    pub version: Option<u16>,
    pub variant: String,
    pub bbt_root_offset: Option<u64>,
    pub nbt_root_offset: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct PstHeader {
    pub summary: PstHeaderSummary,
    pub variant: PstVariant,
    pub roots: RootPointers,
}

impl PstHeader {
    pub fn parse(reader: &PstByteReader) -> PstdResult<Self> {
        let header_bytes = reader.read_prefix(PST_HEADER_MIN_BYTES)?;
        Self::parse_bytes(&header_bytes, reader.file_size())
    }

    pub fn parse_bytes(buf: &[u8], file_size: u64) -> PstdResult<Self> {
        if buf.len() < PST_HEADER_MIN_BYTES {
            return Err(PstdError::pst_parse(
                Some(0),
                format!("PST header too short: {} bytes", buf.len()),
            ));
        }

        if buf[0..4] != PST_MAGIC {
            return Err(PstdError::pst_parse(Some(0), "missing PST magic !BDN"));
        }

        let magic_client = String::from_utf8_lossy(&buf[8..10]).to_string();
        let version = u16_le_at(buf, 10, 0)?;
        let variant = match version {
            23 | 36 => PstVariant::Unicode,
            14 | 15 => PstVariant::Ansi,
            _ => PstVariant::Unknown,
        };

        let bbt_root_offset = read_optional_offset(buf, 56)?;
        let nbt_root_offset = read_optional_offset(buf, 48)?;
        let roots = RootPointers {
            bbt_root: bbt_root_offset.map(|offset| PageRef {
                offset: ByteOffset(offset),
            }),
            nbt_root: nbt_root_offset.map(|offset| PageRef {
                offset: ByteOffset(offset),
            }),
        };

        let summary = PstHeaderSummary {
            format: "pst".to_string(),
            parser_status: match variant {
                PstVariant::Unicode => "supported_unicode_header".to_string(),
                PstVariant::Ansi => "detected_ansi_header_unsupported_for_extraction".to_string(),
                PstVariant::Unknown => "detected_unknown_version".to_string(),
            },
            file_size,
            magic: "!BDN".to_string(),
            magic_client: Some(magic_client),
            version: Some(version),
            variant: format!("{:?}", variant).to_lowercase(),
            bbt_root_offset,
            nbt_root_offset,
        };

        Ok(Self {
            summary,
            variant,
            roots,
        })
    }
}

fn read_optional_offset(buf: &[u8], start: usize) -> PstdResult<Option<u64>> {
    if buf.len() < start + 8 {
        return Ok(None);
    }
    let value = u64_le_at(buf, start, 0)?;
    if value == 0 {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

pub fn validate_pst_magic(buf: &[u8]) -> PstdResult<()> {
    if buf.len() < 4 {
        return Err(PstdError::pst_parse(
            Some(0),
            "file too short to contain PST magic",
        ));
    }
    if buf[0..4] != PST_MAGIC {
        return Err(PstdError::pst_parse(Some(0), "missing PST magic !BDN"));
    }
    Ok(())
}

pub fn summarize_version(version: Option<u16>) -> String {
    match version {
        Some(23) | Some(36) => "unicode".to_string(),
        Some(14) | Some(15) => "ansi".to_string(),
        Some(value) => format!("unknown_version_{value}"),
        None => "unknown".to_string(),
    }
}
