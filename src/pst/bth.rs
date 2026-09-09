use std::collections::HashSet;

use crate::error::{PstdError, PstdResult};
use crate::pst::binary::{slice_at, u16_le_at, u32_le_at, u8_at};
use crate::pst::heap::HeapOnNode;
use crate::pst::tcinfo::{classify_hnid, HnidKind};

const BTH_HEADER_TYPE: u8 = 0xb5;
const MAX_BTH_ENTRIES: usize = 4096;
const MAX_BTH_INDEX_LEVELS: u8 = 8;
const PTYP_OBJECT: u16 = 0x000d;

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

/// Storage resolution before semantic MAPI decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PropertyStorageStatus {
    Inline,
    Heap,
    Subnode,
    DataTree,
    NodeUnresolved,
    HeapUnresolved,
    Null,
    ObjectReference,
}

/// The original PC leaf identity and value/reference, independent of flattened bytes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PropertySource {
    pub prop_id: u16,
    pub prop_type: u16,
    pub value_hnid: u32,
    /// None for inline scalars: their bits must never be interpreted as an HNID.
    pub hnid_kind: Option<HnidKind>,
    pub status: PropertyStorageStatus,
    #[serde(default)]
    pub owner_node_id: Option<u64>,
    #[serde(default)]
    pub source_block_ids: Vec<u64>,
    #[serde(default)]
    pub resolution_detail: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BthPropertyEntry {
    pub entry: BthEntry,
    pub source: Option<PropertySource>,
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
        let (header, properties) =
            Self::parse_property_context_with_sources(heap, buf, base_offset)?;
        Ok(Self {
            header,
            entries: properties
                .into_iter()
                .map(|property| property.entry)
                .collect(),
        })
    }

    /// Parse PC leaves without losing reference provenance. Legacy flattened callers
    /// can continue using parse_property_context_from_heap.
    pub fn parse_property_context_with_sources(
        heap: &HeapOnNode,
        buf: &[u8],
        base_offset: u64,
    ) -> PstdResult<(BthHeader, Vec<BthPropertyEntry>)> {
        let bth_header = heap.allocation_by_hid(buf, heap.header.user_root, base_offset)?;
        if bth_header.len() < 8 {
            return Err(PstdError::pst_parse(
                Some(base_offset),
                "heap BTH header too short",
            ));
        }
        let bth_type = bth_header[0];
        if bth_type != BTH_HEADER_TYPE {
            return Err(PstdError::pst_parse(
                Some(base_offset),
                format!("unexpected heap BTH type 0x{bth_type:02x}"),
            ));
        }
        let key_size = bth_header[1];
        let value_size = bth_header[2];
        if key_size != 2 || value_size != 6 {
            return Err(PstdError::pst_parse(Some(base_offset), "invalid Property Context BTH record widths"));
        }
        let index_levels = bth_header[3];
        if index_levels > MAX_BTH_INDEX_LEVELS {
            return Err(PstdError::pst_parse(
                Some(base_offset),
                format!("heap BTH index levels exceed limit: {index_levels}"),
            ));
        }
        let root_allocation =
            u32::from_le_bytes([bth_header[4], bth_header[5], bth_header[6], bth_header[7]]);

        let root = heap.allocation_by_hid(buf, root_allocation, base_offset)?;
        let mut walker = HeapBthWalker {
            heap, heap_buf: buf, key_size, value_size, base_offset,
            seen: HashSet::from([root_allocation]), entry_count: 0,
        };
        let entries = walker.entries(root, index_levels)?;
        let mut tags = HashSet::new();
        if entries.iter().any(|property| !tags.insert(property.entry.key.clone())) {
            return Err(PstdError::pst_parse(Some(base_offset), "duplicate Property Context tag"));
        }

        Ok((
            BthHeader {
                key_size: 4,
                value_size,
                entry_count: entries.len() as u16,
                root_allocation,
            },
            entries,
        ))
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

struct HeapBthWalker<'a> {
    heap: &'a HeapOnNode,
    heap_buf: &'a [u8],
    key_size: u8,
    value_size: u8,
    base_offset: u64,
    seen: HashSet<u32>,
    entry_count: usize,
}

impl HeapBthWalker<'_> {
    fn entries(&mut self, allocation: &[u8], index_levels: u8) -> PstdResult<Vec<BthPropertyEntry>> {
        if index_levels == 0 {
            let entries = parse_heap_leaf_entries(
                self.heap, self.heap_buf, allocation, self.key_size, self.value_size, self.base_offset,
            )?;
            self.entry_count += entries.len();
            if self.entry_count > MAX_BTH_ENTRIES {
                return Err(PstdError::pst_parse(Some(self.base_offset), "BTH entry resource limit exceeded"));
            }
            return Ok(entries);
        }
        let entry_size = self.key_size as usize + 4;
        if allocation.len() % entry_size != 0 {
            return Err(PstdError::pst_parse(Some(self.base_offset), "truncated BTH index entry"));
        }
        let mut entries = Vec::new();
        for item in allocation.chunks_exact(entry_size) {
            let child_hid = u32::from_le_bytes(item[self.key_size as usize..].try_into().unwrap());
            if !self.seen.insert(child_hid) {
                return Err(PstdError::pst_parse(Some(self.base_offset), "BTH cyclic or duplicate child HID"));
            }
            if self.seen.len() > MAX_BTH_ENTRIES {
                return Err(PstdError::pst_parse(Some(self.base_offset), "BTH index resource limit exceeded"));
            }
            if child_hid == 0 || child_hid & 0x1f != 0 || child_hid >> 16 != 0 {
                return Err(PstdError::pst_parse(Some(self.base_offset), "invalid BTH child HID"));
            }
            let child = self.heap.allocation_by_hid(self.heap_buf, child_hid, self.base_offset)?;
            entries.extend(self.entries(child, index_levels - 1)?);
        }
        Ok(entries)
    }
}

fn parse_heap_leaf_entries(
    heap: &HeapOnNode,
    heap_buf: &[u8],
    allocation: &[u8],
    key_size: u8,
    value_size: u8,
    base_offset: u64,
) -> PstdResult<Vec<BthPropertyEntry>> {
    let entry_size = key_size as usize + value_size as usize;
    if entry_size == 0 {
        return Err(PstdError::pst_parse(
            Some(base_offset),
            "heap BTH entry size is zero",
        ));
    }

    let mut entries = Vec::new();
    if allocation.len() % entry_size != 0 {
        return Err(PstdError::pst_parse(Some(base_offset), "truncated BTH leaf entry"));
    }
    let entry_count = allocation.len() / entry_size;
    if entry_count > MAX_BTH_ENTRIES {
        return Err(PstdError::pst_parse(Some(base_offset), "BTH entry resource limit exceeded"));
    }
    for idx in 0..entry_count {
        let cursor = idx * entry_size;
        let raw_key = slice_at(allocation, cursor, key_size as usize, base_offset)?;
        let raw_value = slice_at(
            allocation,
            cursor + key_size as usize,
            value_size as usize,
            base_offset,
        )?;
        entries.push(property_context_entry(
            heap,
            heap_buf,
            raw_key,
            raw_value,
            base_offset,
        ));
    }
    Ok(entries)
}

fn property_context_entry(
    heap: &HeapOnNode,
    buf: &[u8],
    raw_key: &[u8],
    raw_value: &[u8],
    base_offset: u64,
) -> BthPropertyEntry {
    if raw_key.len() == 2 && raw_value.len() >= 6 {
        let prop_id = u16::from_le_bytes([raw_key[0], raw_key[1]]);
        let prop_type = u16::from_le_bytes([raw_value[0], raw_value[1]]);
        let value_hnid =
            u32::from_le_bytes([raw_value[2], raw_value[3], raw_value[4], raw_value[5]]);
        let tag = ((prop_id as u32) << 16) | prop_type as u32;
        // MS-PST PC inline types occupy at most the four-byte value slot.
        // Fixed-width types larger than four bytes still require HNID resolution.
        let inline = matches!(prop_type, 0x0002 | 0x0003 | 0x0004 | 0x000a | 0x000b);
        let hnid_kind = (!inline).then(|| classify_hnid(value_hnid));
        let (status, resolved) = if inline {
            (PropertyStorageStatus::Inline, None)
        } else if prop_type == PTYP_OBJECT {
            (PropertyStorageStatus::ObjectReference, None)
        } else {
            match classify_hnid(value_hnid) {
                HnidKind::Null => (PropertyStorageStatus::Null, Some(Vec::new())),
                HnidKind::NodeId => (PropertyStorageStatus::NodeUnresolved, None),
                HnidKind::HeapId => {
                    // This heap represents one page. Do not let high block-index
                    // bits alias an allocation on this page.
                    let bytes = if value_hnid >> 16 == 0 {
                        heap.try_allocation_by_hnid(buf, value_hnid, base_offset)
                    } else {
                        None
                    };
                    match bytes {
                        Some(bytes) => (PropertyStorageStatus::Heap, Some(bytes.to_vec())),
                        None => (PropertyStorageStatus::HeapUnresolved, None),
                    }
                }
            }
        };
        return BthPropertyEntry {
            entry: BthEntry {
                key: tag.to_le_bytes().to_vec(),
                value: resolved.unwrap_or_else(|| value_hnid.to_le_bytes().to_vec()),
            },
            source: Some(PropertySource {
                prop_id,
                prop_type,
                value_hnid,
                hnid_kind,
                status,
                owner_node_id: None,
                source_block_ids: Vec::new(),
                resolution_detail: None,
            }),
        };
    }

    BthPropertyEntry {
        entry: BthEntry {
            key: raw_key.to_vec(),
            value: raw_value.to_vec(),
        },
        source: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{property_context_entry, BthMap, PropertyStorageStatus};
    use crate::pst::heap::{HeapAllocation, HeapHeader, HeapOnNode};
    use crate::pst::mapi::{PR_ATTACH_DATA_OBJ, PR_SUBJECT};
    use crate::pst::tcinfo::HnidKind;

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

    #[test]
    fn parses_indexed_heap_property_context_entries() {
        let heap_bytes = indexed_property_context_heap();
        let heap = HeapOnNode::parse(&heap_bytes, 0).unwrap();
        let bth = BthMap::parse_property_context_from_heap(&heap, &heap_bytes, 0).unwrap();

        assert_eq!(bth.entries.len(), 1);
        assert_eq!(bth.entries[0].key, PR_SUBJECT.to_le_bytes());
        assert_eq!(bth.entries[0].value, utf16le("Indexed heap subject"));
    }

    #[test]
    fn preserves_object_nid_even_when_it_aliases_a_heap_allocation() {
        let heap = HeapOnNode {
            header: HeapHeader {
                page_map_offset: 0,
                signature: 0xec,
                client_signature: 0xbc,
                user_root: 0,
                allocation_count: 52,
                free_allocation_count: 0,
            },
            allocations: vec![HeapAllocation {
                id: 52,
                offset: 0,
                size: 7,
            }],
        };
        let mut raw_value = 0x000du16.to_le_bytes().to_vec();
        raw_value.extend_from_slice(&0x684u32.to_le_bytes());

        let entry =
            property_context_entry(&heap, b"aliased", &0x3701u16.to_le_bytes(), &raw_value, 0);

        assert_eq!(entry.entry.key, PR_ATTACH_DATA_OBJ.to_le_bytes());
        assert_eq!(entry.entry.value, 0x684u32.to_le_bytes());
    }

    #[test]
    fn preserves_inline_bits_even_when_they_alias_an_allocation() {
        let bytes = property_context_heap();
        let heap = HeapOnNode::parse(&bytes, 0).unwrap();
        for prop_type in [0x0002u16, 0x0003, 0x0004, 0x000a, 0x000b] {
            let mut raw = prop_type.to_le_bytes().to_vec();
            raw.extend_from_slice(&0x60u32.to_le_bytes());
            let property = property_context_entry(&heap, &bytes, &[1, 0], &raw, 0);
            assert_eq!(property.entry.value, 0x60u32.to_le_bytes());
            let source = property.source.unwrap();
            assert_eq!(source.status, PropertyStorageStatus::Inline);
            assert_eq!(source.hnid_kind, None);
        }
    }

    #[test]
    fn preserves_heap_payload_and_source_through_public_parser() {
        let bytes = indexed_property_context_heap();
        let heap = HeapOnNode::parse(&bytes, 0).unwrap();
        let (_, properties) =
            BthMap::parse_property_context_with_sources(&heap, &bytes, 0).unwrap();
        assert_eq!(properties[0].entry.value, utf16le("Indexed heap subject"));
        let source = properties[0].source.as_ref().unwrap();
        assert_eq!(source.prop_id, 0x0037);
        assert_eq!(source.prop_type, 0x001f);
        assert_eq!(source.value_hnid, 0x80);
        assert_eq!(source.hnid_kind, Some(HnidKind::HeapId));
        assert_eq!(source.status, PropertyStorageStatus::Heap);
    }

    #[test]
    fn unresolved_references_never_alias_heap_payloads() {
        let bytes = property_context_heap();
        let heap = HeapOnNode::parse(&bytes, 0).unwrap();
        for (hnid, kind, status) in [
            (
                0x64u32,
                HnidKind::NodeId,
                PropertyStorageStatus::NodeUnresolved,
            ),
            (
                0x80,
                HnidKind::HeapId,
                PropertyStorageStatus::HeapUnresolved,
            ),
            (
                0x10060,
                HnidKind::HeapId,
                PropertyStorageStatus::HeapUnresolved,
            ),
        ] {
            let mut raw = 0x0102u16.to_le_bytes().to_vec();
            raw.extend_from_slice(&hnid.to_le_bytes());
            let property = property_context_entry(&heap, &bytes, &[0x13, 0x10], &raw, 0);
            assert_eq!(property.entry.value, hnid.to_le_bytes());
            let source = property.source.unwrap();
            assert_eq!(source.value_hnid, hnid);
            assert_eq!(source.hnid_kind, Some(kind));
            assert_eq!(source.status, status);
        }
    }

    #[test]
    fn rejects_cyclic_and_truncated_heap_bth() {
        let mut bytes = indexed_property_context_heap();
        bytes[26..30].copy_from_slice(&0x40u32.to_le_bytes());
        let heap = HeapOnNode::parse(&bytes, 0).unwrap();
        assert!(BthMap::parse_property_context_from_heap(&heap, &bytes, 0)
            .unwrap_err().to_string().contains("cyclic or duplicate"));
        let mut bytes = property_context_heap();
        bytes[152..154].copy_from_slice(&31u16.to_le_bytes());
        let heap = HeapOnNode::parse(&bytes, 0).unwrap();
        assert!(BthMap::parse_property_context_from_heap(&heap, &bytes, 0)
            .unwrap_err().to_string().contains("truncated BTH leaf"));
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

    fn indexed_property_context_heap() -> Vec<u8> {
        let subject = utf16le("Indexed heap subject");
        let subject_start = 38u16;
        let subject_end = subject_start + subject.len() as u16;
        let page_map_offset = 176u16;
        let mut buf = vec![0; 192];
        buf[0..2].copy_from_slice(&page_map_offset.to_le_bytes());
        buf[2] = 0xec;
        buf[3] = 0xbc;
        buf[4..8].copy_from_slice(&0x20u32.to_le_bytes());

        buf[16] = 0xb5;
        buf[17] = 2;
        buf[18] = 6;
        buf[19] = 1;
        buf[20..24].copy_from_slice(&0x40u32.to_le_bytes());

        buf[24..26].copy_from_slice(&0x0037u16.to_le_bytes());
        buf[26..30].copy_from_slice(&0x60u32.to_le_bytes());

        buf[30..32].copy_from_slice(&0x0037u16.to_le_bytes());
        buf[32..34].copy_from_slice(&0x001fu16.to_le_bytes());
        buf[34..38].copy_from_slice(&0x80u32.to_le_bytes());

        buf[subject_start as usize..subject_end as usize].copy_from_slice(&subject);

        buf[176..178].copy_from_slice(&4u16.to_le_bytes());
        buf[178..180].copy_from_slice(&0u16.to_le_bytes());
        buf[180..182].copy_from_slice(&16u16.to_le_bytes());
        buf[182..184].copy_from_slice(&24u16.to_le_bytes());
        buf[184..186].copy_from_slice(&30u16.to_le_bytes());
        buf[186..188].copy_from_slice(&38u16.to_le_bytes());
        buf[188..190].copy_from_slice(&subject_end.to_le_bytes());
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
