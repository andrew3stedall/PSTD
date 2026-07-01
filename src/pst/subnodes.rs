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

pub fn subnode_references_from_index(index: &NbtIndex) -> SubnodeReferenceReport {
    subnode_references_from_entries(&index.entries)
}

pub fn subnode_references_from_entries(entries: &[NbtEntry]) -> SubnodeReferenceReport {
    let references = entries
        .iter()
        .filter_map(|entry| {
            entry.subnode_block_id.map(|subnode_block_id| SubnodeReference {
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

#[cfg(test)]
mod tests {
    use super::subnode_references_from_entries;
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
}
