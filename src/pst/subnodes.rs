use crate::pst::bbt::BbtIndex;
use crate::pst::limits::ParserLimits;
use crate::pst::nbt::{NbtEntry, NbtIndex};
use crate::pst::payload::{load_payload_block, PayloadBlock};
use crate::pst::primitives::{BlockId, NodeId};
use crate::pst::reader::PstByteReader;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubnodeReference {
    pub node_id: NodeId,
    pub subnode_block_id: BlockId,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubnodeReferenceReport {
    pub node_count: usize,
    pub subnode_reference_count: usize,
    pub references: Vec<SubnodeReference>,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubnodeDecodePlan {
    pub root_node_id: NodeId,
    pub root_subnode_block_id: BlockId,
    pub requested_depth: usize,
    pub max_depth: usize,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubnodeDecodeReport {
    pub root_node_id: NodeId,
    pub root_subnode_block_id: BlockId,
    pub requested_depth: usize,
    pub max_depth: usize,
    pub decoded_block_count: usize,
    pub failed_block_count: usize,
    pub decoded_bytes: u64,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct LoadedSubnodeBlocks {
    pub payloads: Vec<PayloadBlock>,
    pub report: SubnodeDecodeReport,
}

pub fn subnode_references_from_index(index: &NbtIndex) -> SubnodeReferenceReport {
    subnode_references_from_entries(&index.entries)
}

pub fn subnode_references_from_entries(entries: &[NbtEntry]) -> SubnodeReferenceReport {
    let references = entries
        .iter()
        .filter_map(|entry| {
            entry
                .subnode_block_id
                .map(|subnode_block_id| SubnodeReference {
                    node_id: entry.node_id,
                    subnode_block_id,
                    status: "subnode_reference_discovered".to_string(),
                })
        })
        .collect::<Vec<_>>();

    let status = if references.is_empty() {
        "no_subnode_references".to_string()
    } else {
        "subnode_references_discovered".to_string()
    };

    SubnodeReferenceReport {
        node_count: entries.len(),
        subnode_reference_count: references.len(),
        references,
        status,
    }
}

pub fn subnode_decode_plans(
    references: &[SubnodeReference],
    limits: ParserLimits,
) -> Vec<SubnodeDecodePlan> {
    references
        .iter()
        .map(|reference| subnode_decode_plan(reference, 1, limits))
        .collect()
}

pub fn subnode_decode_plan(
    reference: &SubnodeReference,
    requested_depth: usize,
    limits: ParserLimits,
) -> SubnodeDecodePlan {
    let status = if requested_depth > limits.max_subnode_depth {
        "subnode_depth_limit_exceeded"
    } else {
        "subnode_decode_planned"
    };

    SubnodeDecodePlan {
        root_node_id: reference.node_id,
        root_subnode_block_id: reference.subnode_block_id,
        requested_depth,
        max_depth: limits.max_subnode_depth,
        status: status.to_string(),
    }
}

pub fn load_bounded_subnode_blocks(
    reader: &PstByteReader,
    bbt: &BbtIndex,
    reference: &SubnodeReference,
    requested_depth: usize,
    limits: ParserLimits,
) -> LoadedSubnodeBlocks {
    if requested_depth > limits.max_subnode_depth {
        return LoadedSubnodeBlocks {
            payloads: Vec::new(),
            report: SubnodeDecodeReport {
                root_node_id: reference.node_id,
                root_subnode_block_id: reference.subnode_block_id,
                requested_depth,
                max_depth: limits.max_subnode_depth,
                decoded_block_count: 0,
                failed_block_count: 0,
                decoded_bytes: 0,
                status: "subnode_depth_limit_exceeded".to_string(),
            },
        };
    }

    match load_payload_block(reader, bbt, reference.subnode_block_id, limits) {
        Ok(payload) => LoadedSubnodeBlocks {
            report: SubnodeDecodeReport {
                root_node_id: reference.node_id,
                root_subnode_block_id: reference.subnode_block_id,
                requested_depth,
                max_depth: limits.max_subnode_depth,
                decoded_block_count: 1,
                failed_block_count: 0,
                decoded_bytes: payload.bytes.len() as u64,
                status: "subnode_root_block_loaded".to_string(),
            },
            payloads: vec![payload],
        },
        Err(reason) => LoadedSubnodeBlocks {
            payloads: Vec::new(),
            report: SubnodeDecodeReport {
                root_node_id: reference.node_id,
                root_subnode_block_id: reference.subnode_block_id,
                requested_depth,
                max_depth: limits.max_subnode_depth,
                decoded_block_count: 0,
                failed_block_count: 1,
                decoded_bytes: 0,
                status: format!("subnode_root_block_unavailable; reason={reason}"),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use super::{
        load_bounded_subnode_blocks, subnode_decode_plan, subnode_decode_plans,
        subnode_references_from_entries,
    };
    use crate::pst::bbt::{BbtEntry, BbtIndex};
    use crate::pst::limits::ParserLimits;
    use crate::pst::nbt::NbtEntry;
    use crate::pst::primitives::{BlockId, ByteOffset, NodeId};
    use crate::pst::reader::PstByteReader;

    #[test]
    fn reports_subnode_references() {
        let entries = vec![
            NbtEntry {
                node_id: NodeId(1),
                data_block_id: BlockId(10),
                subnode_block_id: Some(BlockId(100)),
            },
            NbtEntry {
                node_id: NodeId(2),
                data_block_id: BlockId(20),
                subnode_block_id: None,
            },
        ];

        let report = subnode_references_from_entries(&entries);
        assert_eq!(report.node_count, 2);
        assert_eq!(report.subnode_reference_count, 1);
        assert_eq!(report.references[0].node_id.0, 1);
        assert_eq!(report.references[0].subnode_block_id.0, 100);
        assert_eq!(report.status, "subnode_references_discovered");
    }

    #[test]
    fn reports_empty_subnode_reference_set() {
        let report = subnode_references_from_entries(&[]);
        assert_eq!(report.node_count, 0);
        assert_eq!(report.subnode_reference_count, 0);
        assert_eq!(report.status, "no_subnode_references");
    }

    #[test]
    fn plans_subnode_decoding_with_depth_limit() {
        let entries = vec![NbtEntry {
            node_id: NodeId(1),
            data_block_id: BlockId(10),
            subnode_block_id: Some(BlockId(100)),
        }];
        let report = subnode_references_from_entries(&entries);
        let plans = subnode_decode_plans(&report.references, ParserLimits::default());
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].status, "subnode_decode_planned");
    }

    #[test]
    fn marks_subnode_depth_limit_exceeded() {
        let reference = subnode_references_from_entries(&[NbtEntry {
            node_id: NodeId(1),
            data_block_id: BlockId(10),
            subnode_block_id: Some(BlockId(100)),
        }])
        .references
        .remove(0);
        let limits = ParserLimits {
            max_subnode_depth: 1,
            ..ParserLimits::default()
        };

        let plan = subnode_decode_plan(&reference, 2, limits);
        assert_eq!(plan.status, "subnode_depth_limit_exceeded");
        assert_eq!(plan.max_depth, 1);
    }

    #[test]
    fn loads_bounded_subnode_root_block() {
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), b"0123456789subnode").unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = index_with_entry(BlockId(100), 10, 7);
        let reference = subnode_references_from_entries(&[NbtEntry {
            node_id: NodeId(1),
            data_block_id: BlockId(10),
            subnode_block_id: Some(BlockId(100)),
        }])
        .references
        .remove(0);

        let loaded = load_bounded_subnode_blocks(&reader, &bbt, &reference, 1, ParserLimits::default());
        assert_eq!(loaded.payloads.len(), 1);
        assert_eq!(loaded.payloads[0].bytes, b"subnode");
        assert_eq!(loaded.report.decoded_block_count, 1);
        assert_eq!(loaded.report.status, "subnode_root_block_loaded");
    }

    #[test]
    fn refuses_subnode_decode_over_depth_limit() {
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), b"0123456789subnode").unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = index_with_entry(BlockId(100), 10, 7);
        let reference = subnode_references_from_entries(&[NbtEntry {
            node_id: NodeId(1),
            data_block_id: BlockId(10),
            subnode_block_id: Some(BlockId(100)),
        }])
        .references
        .remove(0);
        let limits = ParserLimits {
            max_subnode_depth: 0,
            ..ParserLimits::default()
        };

        let loaded = load_bounded_subnode_blocks(&reader, &bbt, &reference, 1, limits);
        assert!(loaded.payloads.is_empty());
        assert_eq!(loaded.report.status, "subnode_depth_limit_exceeded");
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
