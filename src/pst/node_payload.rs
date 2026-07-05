use crate::error::PstdResult;
use crate::pst::bbt::BbtIndex;
use crate::pst::bth::BthMap;
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
    let bth = BthMap::parse(&payload.bytes, payload.block_ref.offset.0)?;
    let property_report = PropertyContext::from_bth_with_report(&bth)?;
    let properties = property_report.context.clone();
    let report = NodePayloadReport {
        node_id: entry.node_id.0,
        data_block_id: entry.data_block_id.0,
        payload_size_bytes: payload.bytes.len() as u64,
        property_count: property_report.parsed_property_count,
        status: "node_property_context_loaded".to_string(),
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
    fn loads_node_property_context_from_data_block() {
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
        assert_eq!(loaded.report.status, "node_property_context_loaded");
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

    fn utf16le_fixed(value: &str, len: usize) -> Vec<u8> {
        let mut bytes = Vec::new();
        for unit in value.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes.extend_from_slice(&0u16.to_le_bytes());
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
            page_diagnostics: Vec::new(),
            status: "test".to_string(),
        }
    }
}
