use crate::error::{PstdError, PstdResult};
use crate::pst::binary::{slice_at, u16_le_at, u32_le_at, u8_at};
use crate::pst::heap::HeapOnNode;

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
        let entries = parse_flat_entries(buf, 8, &header, base_offset)?;
        Ok(Self { header, entries })
    }

    pub fn parse_property_context_from_heap(
        heap: &HeapOnNode,
        buf: &[u8],
        base_offset: u64,
    ) -> PstdResult<Self> {
        let bth_header = heap.allocation_by_hid(buf, heap.header.user_root, base_offset)?;
        if bth_header.len() < 8 {
            return Err(PstdError::pst_parse(
                Some(base_offset),
                "heap BTH header too short",
            ));
        }
        let bth_type = bth_header[0];
        if bth_type != 0xb5 {
            return Err(PstdError::pst_parse(
                Some(base_offset),
                format!("unexpected heap BTH type 0x{bth_type:02x}"),
            ));
        }
        let key_size = bth_header[1];
        let value_size = bth_header[2];
        let index_levels = bth_header[3];
        let root_allocation = u32::from_le_bytes([
            bth_header[4],
            bth_header[5],
            bth_header[6],
            bth_header[7],
        ]);
        if index_levels != 0 {
            return Err(PstdError::pst_parse(
                Some(base_offset),
                format!("heap BTH index levels not yet supported: {index_levels}"),
            ));
        }
        let root = heap.allocation_by_hid(buf, root_allocation, base_offset)?;
        let entry_size = key_size as usize + value_size as usize;
        if entry_size == 0 {
            return Err(PstdError::pst_parse(
                Some(base_offset),
                "heap BTH entry size is zero",
            ));
        }

        let mut entries = Vec::new();
        let entry_count = root.len() / entry_size;
        for idx in 0..entry_count {
            let cursor = idx * entry_size;
            let raw_key = slice_at(root, cursor, key_size as usize, base_offset)?;
            let raw_value = slice_at(
                root,
                cursor + key_size as usize,
                value_size as usize,
                base_offset,
            )?;
            entries.push(property_context_entry(
                heap,
                buf,
                raw_key,
                raw_value,
                base_offset,
            ));
        }

        Ok(Self {
            header: BthHeader {
                key_size: 4,
                value_size,
                entry_count: entries.len() as u16,
                root_allocation,
            },
            entries,
        })
    }

    pub fn lookup(&self, key: &[u8]) -> Option<&[u8]> {
        self.entries
            .iter()
            .find(|entry| entry.key.as_slice() == key)
            .map(|entry| entry.value.as_slice())
    }
}

fn parse_flat_entries(
    buf: &[u8],
    start: usize,
    header: &BthHeader,
    base_offset: u64,
) -> PstdResult<Vec<BthEntry>> {
    let entry_size = header.key_size as usize + header.value_size as usize;
    if entry_size == 0 {
        return Err(PstdError::pst_parse(
            Some(base_offset),
            "BTH entry size is zero",
        ));
    }

    let mut entries = Vec::new();
    let mut cursor = start;
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
    Ok(entries)
}

fn property_context_entry(
    heap: &HeapOnNode,
    buf: &[u8],
    raw_key: &[u8],
    raw_value: &[u8],
    base_offset: u64,
) -> BthEntry {
    if raw_key.len() == 2 && raw_value.len() >= 6 {
        let prop_id = u16::from_le_bytes([raw_key[0], raw_key[1]]);
        let prop_type = u16::from_le_bytes([raw_value[0], raw_value[1]]);
        let value_hnid =
            u32::from_le_bytes([raw_value[2], raw_value[3], raw_value[4], raw_value[5]]);
        let tag = ((prop_id as u32) << 16) | prop_type as u32;
        let value = heap
            .try_allocation_by_hnid(buf, value_hnid, base_offset)
            .map(|bytes| bytes.to_vec())
            .unwrap_or_else(|| value_hnid.to_le_bytes().to_vec());
        return BthEntry {
            key: tag.to_le_bytes().to_vec(),
            value,
        };
    }

    BthEntry {
        key: raw_key.to_vec(),
        value: raw_value.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::BthMap;
    use crate::pst::heap::HeapOnNode;
    use crate::pst::mapi::PR_SUBJECT;

    #[test]
    fn parses_legacy_flat_bth_entries() {
        let mut body = Vec::new();
        body.push(4);
        body.push(4);
        body.extend_from_slice(&1u16.to_le_bytes());
        body.extend_from_slice(&0u32.to_le_bytes());
        body.extend_from_slice(&PR_SUBJECT.to_le_bytes());
        body.extend_from_slice(&123u32.to_le_bytes());

        let bth = BthMap::parse(&body, 0).unwrap();
        assert_eq!(bth.header.key_size, 4);
        assert_eq!(bth.header.value_size, 4);
        assert_eq!(bth.entries.len(), 1);
        assert_eq!(bth.entries[0].key, PR_SUBJECT.to_le_bytes());
        assert_eq!(bth.entries[0].value, 123u32.to_le_bytes());
    }

    #[test]
    fn parses_heap_property_context_entries() {
        let heap_bytes = property_context_heap();
        let heap = HeapOnNode::parse(&heap_bytes, 0).unwrap();
        let bth = BthMap::parse_property_context_from_heap(&heap, &heap_bytes, 0).unwrap();

        assert_eq!(bth.entries.len(), 1);
        assert_eq!(bth.entries[0].key, PR_SUBJECT.to_le_bytes());
        assert_eq!(bth.entries[0].value, utf16le("Heap subject"));
    }

    fn property_context_heap() -> Vec<u8> {
        let subject = utf16le("Heap subject");
        let subject_end = 32u16 + subject.len() as u16;
        let mut buf = vec![0; 160];
        buf[0..2].copy_from_slice(&144u16.to_le_bytes());
        buf[2] = 0xec;
        buf[3] = 0xbc;
        buf[4..8].copy_from_slice(&0x20u32.to_le_bytes());

        buf[16] = 0xb5;
        buf[17] = 2;
        buf[18] = 6;
        buf[19] = 0;
        buf[20..24].copy_from_slice(&0x40u32.to_le_bytes());

        buf[24..26].copy_from_slice(&0x0037u16.to_le_bytes());
        buf[26..28].copy_from_slice(&0x001fu16.to_le_bytes());
        buf[28..32].copy_from_slice(&0x60u32.to_le_bytes());

        buf[32..subject_end as usize].copy_from_slice(&subject);

        buf[144..146].copy_from_slice(&3u16.to_le_bytes());
        buf[146..148].copy_from_slice(&0u16.to_le_bytes());
        buf[148..150].copy_from_slice(&16u16.to_le_bytes());
        buf[150..152].copy_from_slice(&24u16.to_le_bytes());
        buf[152..154].copy_from_slice(&32u16.to_le_bytes());
        buf[154..156].copy_from_slice(&subject_end.to_le_bytes());
        buf
    }

    fn utf16le(value: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        for unit in value.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes
    }
}
