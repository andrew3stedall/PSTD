use crate::error::{PstdError, PstdResult};
use crate::pst::binary::{u16_le_at, u32_le_at, u8_at};

const TCINFO_TYPE: u8 = 0x7c;
const TCINFO_HEADER_BYTES: usize = 22;
const TCOLDESC_BYTES: usize = 8;
const MAX_TC_COLUMNS: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HnidKind {
    HeapId,
    NodeId,
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TcColumnDescriptor {
    pub property_tag: u32,
    pub data_offset: u16,
    pub data_size: u8,
    pub bitmap_bit: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TcInfo {
    pub column_count: u8,
    pub data_region_boundaries: [u16; 4],
    pub row_index_hid: u32,
    pub rows_hnid: u32,
    pub rows_hnid_kind: HnidKind,
    pub index_hid: u32,
    pub columns: Vec<TcColumnDescriptor>,
}

impl TcInfo {
    pub fn parse(buf: &[u8], base_offset: u64) -> PstdResult<Self> {
        if buf.len() < TCINFO_HEADER_BYTES {
            return Err(PstdError::pst_parse(
                Some(base_offset),
                "TCINFO allocation shorter than 22-byte header",
            ));
        }

        let table_type = u8_at(buf, 0, base_offset)?;
        if table_type != TCINFO_TYPE {
            return Err(PstdError::pst_parse(
                Some(base_offset),
                format!("TCINFO type mismatch: 0x{table_type:02x}"),
            ));
        }

        let column_count = u8_at(buf, 1, base_offset)?;
        if column_count as usize > MAX_TC_COLUMNS {
            return Err(PstdError::pst_parse(
                Some(base_offset + 1),
                format!("TCINFO column count exceeds limit: {column_count}"),
            ));
        }

        let required_len = TCINFO_HEADER_BYTES
            .checked_add(column_count as usize * TCOLDESC_BYTES)
            .ok_or_else(|| PstdError::pst_parse(Some(base_offset), "TCINFO size overflow"))?;
        if required_len > buf.len() {
            return Err(PstdError::pst_parse(
                Some(base_offset + TCINFO_HEADER_BYTES as u64),
                format!(
                    "TCINFO column descriptors truncated: need {required_len} bytes, have {}",
                    buf.len()
                ),
            ));
        }

        let data_region_boundaries = [
            u16_le_at(buf, 2, base_offset)?,
            u16_le_at(buf, 4, base_offset)?,
            u16_le_at(buf, 6, base_offset)?,
            u16_le_at(buf, 8, base_offset)?,
        ];
        if data_region_boundaries
            .windows(2)
            .any(|pair| pair[0] > pair[1])
        {
            return Err(PstdError::pst_parse(
                Some(base_offset + 2),
                "TCINFO data-region boundaries are not monotonic",
            ));
        }

        let row_index_hid = u32_le_at(buf, 10, base_offset)?;
        let rows_hnid = u32_le_at(buf, 14, base_offset)?;
        let index_hid = u32_le_at(buf, 18, base_offset)?;

        let mut columns = Vec::with_capacity(column_count as usize);
        for index in 0..column_count as usize {
            let start = TCINFO_HEADER_BYTES + index * TCOLDESC_BYTES;
            let descriptor = TcColumnDescriptor {
                property_tag: u32_le_at(buf, start, base_offset)?,
                data_offset: u16_le_at(buf, start + 4, base_offset)?,
                data_size: u8_at(buf, start + 6, base_offset)?,
                bitmap_bit: u8_at(buf, start + 7, base_offset)?,
            };
            if descriptor.data_size == 0 {
                return Err(PstdError::pst_parse(
                    Some(base_offset + start as u64 + 6),
                    format!("TCOLDESC {index} has zero data size"),
                ));
            }
            if descriptor.bitmap_bit as usize >= column_count as usize {
                return Err(PstdError::pst_parse(
                    Some(base_offset + start as u64 + 7),
                    format!(
                        "TCOLDESC {index} bitmap bit {} exceeds column count {column_count}",
                        descriptor.bitmap_bit
                    ),
                ));
            }
            columns.push(descriptor);
        }

        Ok(Self {
            column_count,
            data_region_boundaries,
            row_index_hid,
            rows_hnid,
            rows_hnid_kind: classify_hnid(rows_hnid),
            index_hid,
            columns,
        })
    }
}

pub fn classify_hnid(hnid: u32) -> HnidKind {
    if hnid == 0 {
        HnidKind::Null
    } else if hnid & 0x1f == 0 {
        HnidKind::HeapId
    } else {
        HnidKind::NodeId
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_hnid, HnidKind, TcInfo};

    #[test]
    fn parses_bounded_tcinfo_and_column_descriptors() {
        let mut bytes = vec![0; 38];
        bytes[0] = 0x7c;
        bytes[1] = 2;
        bytes[2..4].copy_from_slice(&4u16.to_le_bytes());
        bytes[4..6].copy_from_slice(&8u16.to_le_bytes());
        bytes[6..8].copy_from_slice(&10u16.to_le_bytes());
        bytes[8..10].copy_from_slice(&12u16.to_le_bytes());
        bytes[10..14].copy_from_slice(&0x60u32.to_le_bytes());
        bytes[14..18].copy_from_slice(&0x74u32.to_le_bytes());
        bytes[18..22].copy_from_slice(&0x80u32.to_le_bytes());
        bytes[22..26].copy_from_slice(&0x001a0037u32.to_le_bytes());
        bytes[26..28].copy_from_slice(&0u16.to_le_bytes());
        bytes[28] = 4;
        bytes[29] = 0;
        bytes[30..34].copy_from_slice(&0x001f3001u32.to_le_bytes());
        bytes[34..36].copy_from_slice(&4u16.to_le_bytes());
        bytes[36] = 4;
        bytes[37] = 1;

        let info = TcInfo::parse(&bytes, 0).unwrap();
        assert_eq!(info.column_count, 2);
        assert_eq!(info.row_index_hid, 0x60);
        assert_eq!(info.rows_hnid, 0x74);
        assert_eq!(info.rows_hnid_kind, HnidKind::NodeId);
        assert_eq!(info.columns.len(), 2);
        assert_eq!(info.columns[1].property_tag, 0x001f3001);
    }

    #[test]
    fn classifies_hnid_union_without_resolving_it() {
        assert_eq!(classify_hnid(0), HnidKind::Null);
        assert_eq!(classify_hnid(0x60), HnidKind::HeapId);
        assert_eq!(classify_hnid(0x74), HnidKind::NodeId);
    }

    #[test]
    fn rejects_truncated_column_array() {
        let mut bytes = vec![0; 22];
        bytes[0] = 0x7c;
        bytes[1] = 1;
        let error = TcInfo::parse(&bytes, 0).unwrap_err();
        assert!(error.to_string().contains("column descriptors truncated"));
    }

    #[test]
    fn rejects_non_monotonic_region_boundaries() {
        let mut bytes = vec![0; 22];
        bytes[0] = 0x7c;
        bytes[2..4].copy_from_slice(&8u16.to_le_bytes());
        bytes[4..6].copy_from_slice(&4u16.to_le_bytes());
        let error = TcInfo::parse(&bytes, 0).unwrap_err();
        assert!(error.to_string().contains("not monotonic"));
    }
}
