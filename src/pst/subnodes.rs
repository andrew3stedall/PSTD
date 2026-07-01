use crate::pst::limits::ParserLimits;
use crate::pst::nbt::{NbtEntry, NbtIndex};
use crate::pst::primitives::{BlockId, NodeId};

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

#[cfg(test)]
mod tests {
    use super::{subnode_decode_plan, subnode_decode_plans, subnode_references_from_entries};
    use crate::pst::limits::ParserLimits;
    use crate::pst::nbt::NbtEntry;
    use crate::pst::primitives::{BlockId, NodeId};

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
}
