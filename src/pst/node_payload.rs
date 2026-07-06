use crate::error::PstdResult;
use crate::pst::bbt::BbtIndex;
use crate::pst::bth::BthMap;
use crate::pst::heap::HeapOnNode;
use crate::pst::limits::ParserLimits;
use crate::pst::nbt::NbtEntry;
use crate::pst::payload::{load_payload_block, PayloadBlock};
use crate::pst::property_context::{PropertyContext, PropertyContextParseReport};
use crate::pst::reader::PstByteReader;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodePayloadReport {
    pub node_id: u64,
    pub data_block_id: u64,
    pub payload_size_bytes: u64,
    pub property_count: usize,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct LoadedNodePayload {
    pub payload: PayloadBlock,
    pub properties: PropertyContext,
    pub property_report: PropertyContextParseReport,
    pub report: NodePayloadReport,
}

pub fn load_node_property_context(
    reader: &PstByteReader,
    bbt: &BbtIndex,
    entry: &NbtEntry,
    limits: ParserLimits,
) -> PstdResult<LoadedNodePayload> {
    let payload = load_payload_block(reader, bbt, entry.data_block_id, limits)?;
    let payload_base_offset = payload.block_ref.offset.0;
    let (bth, traversal_status) = match HeapOnNode::parse(&payload.bytes, payload_base_offset)
        .and_then(|heap| {
            BthMap::parse_property_context_from_heap(&heap, &payload.bytes, payload_base_offset)
        }) {
        Ok(bth) => (bth, "heap_bth_property_context"),
        Err(_) => (
            BthMap::parse(&payload.bytes, payload_base_offset)?,
            "legacy_flat_bth_property_context",
        ),
    };
    let property_report = PropertyContext::from_bth_with_report(&bth)?;
    let properties = property_report
        .context
        .clone()
        .with_pq10_traversal_status(traversal_status);
    let report = NodePayloadReport {
        node_id: entry.node_id.0,
        data_block_id: entry.data_block_id.0,
        payload_size_bytes: payload.bytes.len() as u64,
        property_count: property_report.parsed_property_count,
        status: format!("node_property_context_loaded; traversal={traversal_status}"),
    };

    Ok(LoadedNodePayload {
        payload,
        properties,
        property_report,
        report,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use super::load_node_property_context;
    use crate::pst::bbt::{BbtEntry, BbtIndex};
    use crate::pst::limits::ParserLimits;
    use crate::pst::mapi::PR_SUBJECT;
    use crate::pst::nbt::NbtEntry;
    use crate::pst::primitives::{BlockId, ByteOffset, NodeId};
    use crate::pst::reader::PstByteReader;

    #[test]
    fn loads_node_property_context_from_legacy_flat_data_block() {
        let mut bytes = bth_with_subject("Hello from node");
        let offset = 16usize;
        let mut file_bytes = vec![0; offset];
        file_bytes.append(&mut bytes);

        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), file_bytes).unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = index_with_entry(BlockId(100), offset as u64, 44);
        let entry = NbtEntry {
            node_id: NodeId(200),
            data_block_id: BlockId(100),
            subnode_block_id: None,
        };

        let loaded =
            load_node_property_context(&reader, &bbt, &entry, ParserLimits::default()).unwrap();
        assert_eq!(loaded.report.node_id, 200);
        assert_eq!(loaded.report.data_block_id, 100);
        assert_eq!(loaded.report.property_count, 1);
        assert_eq!(
            loaded.properties.string_value(PR_SUBJECT).as_deref(),
            Some("Hello from node")
        );
        assert_eq!(
            loaded.properties.pq10_status(),
            "pq10_traversal=legacy_flat_bth_property_context"
        );
        assert_eq!(
            loaded.report.status,
            "node_property_context_loaded; traversal=legacy_flat_bth_property_context"
        );
    }

    #[test]
    fn loads_node_property_context_from_heap_bth_data_block() {
        let mut bytes = heap_bth_with_subject("Hello from heap");
        let offset = 32usize;
        let size = bytes.len() as u64;
        let mut file_bytes = vec![0; offset];
        file_bytes.append(&mut bytes);

        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), file_bytes).unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = index_with_entry(BlockId(100), offset as u64, size);
        let entry = NbtEntry {
            node_id: NodeId(200),
            data_block_id: BlockId(100),
            subnode_block_id: None,
        };

        let loaded =
            load_node_property_context(&reader, &bbt, &entry, ParserLimits::default()).unwrap();
        assert_eq!(loaded.report.property_count, 1);
        assert_eq!(
            loaded.properties.string_value(PR_SUBJECT).as_deref(),
            Some("Hello from heap")
        );
        assert_eq!(
            loaded.properties.pq10_status(),
            "pq10_traversal=heap_bth_property_context"
        );
        assert_eq!(
            loaded.report.status,
            "node_property_context_loaded; traversal=heap_bth_property_context"
        );
    }

    fn bth_with_subject(value: &str) -> Vec<u8> {
        let mut body = Vec::new();
        let mut value_bytes = utf16le_fixed(value, 32);
        body.push(4);
        body.push(32);
        body.extend_from_slice(&1u16.to_le_bytes());
        body.extend_from_slice(&0u32.to_le_bytes());
        body.extend_from_slice(&PR_SUBJECT.to_le_bytes());
        body.append(&mut value_bytes);
        body
    }

    fn heap_bth_with_subject(value: &str) -> Vec<u8> {
        let subject = utf16le(value);
        let subject_end = 32u16 + subject.len() as u16;
        let page_map_offset = 144u16;
        let mut body = vec![0; page_map_offset as usize + 16];
        body[0..2].copy_from_slice(&page_map_offset.to_le_bytes());
        body[2] = 0xec;
        body[3] = 0xbc;
        body[4..8].copy_from_slice(&0x20u32.to_le_bytes());

        body[16] = 0xb5;
        body[17] = 2;
        body[18] = 6;
        body[19] = 0;
        body[20..24].copy_from_slice(&0x40u32.to_le_bytes());

        body[24..26].copy_from_slice(&0x0037u16.to_le_bytes());
        body[26..28].copy_from_slice(&0x001fu16.to_le_bytes());
        body[28..32].copy_from_slice(&0x60u32.to_le_bytes());

        body[32..subject_end as usize].copy_from_slice(&subject);

        body[144..146].copy_from_slice(&3u16.to_le_bytes());
        body[146..148].copy_from_slice(&0u16.to_le_bytes());
        body[148..150].copy_from_slice(&16u16.to_le_bytes());
        body[150..152].copy_from_slice(&24u16.to_le_bytes());
        body[152..154].copy_from_slice(&32u16.to_le_bytes());
        body[154..156].copy_from_slice(&subject_end.to_le_bytes());
        body
    }

    fn utf16le(value: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        for unit in value.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes
    }

    fn utf16le_fixed(value: &str, len: usize) -> Vec<u8> {
        let mut bytes = utf16le(value);
        bytes.resize(len, 0);
        bytes
    }

    fn index_with_entry(block_id: BlockId, offset: u64, size: u64) -> BbtIndex {
        BbtIndex {
            root: None,
            entries: vec![BbtEntry {
                block_id,
                offset: ByteOffset(offset),
                size,
            }],
            parsed_pages: 0,
            discovered_child_pages: 0,
            traversal_error_count: 0,
            duplicate_entry_count: 0,
            truncated_entry_count: 0,
            status: "test".to_string(),
        }
    }
}
