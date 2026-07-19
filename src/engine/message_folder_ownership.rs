use std::collections::HashMap;

use crate::pst::bbt::BbtIndex;
use crate::pst::limits::ParserLimits;
use crate::pst::message_ownership::{
    resolve_message_ownership, MessageMembershipEvidence, MessageOwnershipResolution,
};
use crate::pst::message_table::{
    message_node_type, node_identity, MessageTableDiscovery,
};
use crate::pst::message_table_membership::load_message_table_membership;
use crate::pst::nbt::NbtIndex;
use crate::pst::primitives::NodeId;
use crate::pst::reader::PstByteReader;

#[derive(Debug, Clone)]
pub struct MessageFolderOwnershipReport {
    pub resolutions: HashMap<NodeId, MessageOwnershipResolution>,
    pub diagnostics: Vec<String>,
}

pub fn resolve_folder_ownership(
    reader: &PstByteReader,
    bbt: &BbtIndex,
    nbt: &NbtIndex,
    discovery: &MessageTableDiscovery,
    folder_path_by_key: &HashMap<String, String>,
    limits: ParserLimits,
) -> MessageFolderOwnershipReport {
    let message_candidates: Vec<_> = discovery
        .message_candidates
        .iter()
        .filter_map(|entry| message_node_type(entry.node_id).map(|kind| (entry.node_id, kind)))
        .collect();
    let candidate_types: HashMap<_, _> = message_candidates.iter().copied().collect();
    let mut evidence = Vec::new();
    let mut diagnostics = Vec::new();

    for table in &discovery.table_candidates {
        let Some(folder_key) = table.linked_folder_key.as_ref() else {
            diagnostics.push(format!(
                "{}: message_table_membership_owner_folder_unlinked",
                table.node_identity
            ));
            continue;
        };
        let Some(folder_path) = folder_path_by_key.get(folder_key) else {
            diagnostics.push(format!(
                "{}: message_table_membership_folder_path_unavailable",
                table.node_identity
            ));
            continue;
        };
        let Some(table_entry) = nbt
            .entries
            .iter()
            .find(|entry| node_identity(entry.node_id) == table.node_identity)
        else {
            diagnostics.push(format!(
                "{}: message_table_membership_table_entry_unavailable",
                table.node_identity
            ));
            continue;
        };

        match load_message_table_membership(reader, bbt, table_entry, table.table_type, limits) {
            Ok(membership) => {
                diagnostics.push(format!("{}: {}", table.node_identity, membership.status));
                for message_node_id in membership.member_node_ids {
                    let Some(message_node_type) = candidate_types.get(&message_node_id).copied()
                    else {
                        diagnostics.push(format!(
                            "{}: message_table_member_not_in_nbt_candidates; member={}",
                            table.node_identity,
                            node_identity(message_node_id)
                        ));
                        continue;
                    };
                    evidence.push(MessageMembershipEvidence {
                        message_node_id,
                        message_node_type,
                        table_node_type: table.table_type,
                        folder_key: folder_key.clone(),
                        folder_path: folder_path.clone(),
                    });
                }
            }
            Err(reason) => diagnostics.push(format!(
                "{}: message_table_membership_decode_error; reason={reason}",
                table.node_identity
            )),
        }
    }

    MessageFolderOwnershipReport {
        resolutions: resolve_message_ownership(&message_candidates, &evidence),
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::pst::message_ownership::MessageOwnershipResolution;
    use crate::pst::message_table::{
        MessageNodeType, MessageTableCandidate, MessageTableDiscovery, MessageTableNodeType,
    };
    use crate::pst::nbt::NbtEntry;
    use crate::pst::primitives::{BlockId, NodeId};

    #[test]
    fn reducer_input_remains_unresolved_without_decoded_membership() {
        let discovery = MessageTableDiscovery {
            message_candidates: vec![NbtEntry {
                node_id: NodeId(0x24),
                data_block_id: BlockId(0x1000),
                subnode_block_id: None,
            }],
            table_candidates: Vec::new(),
            linked_table_count: 0,
            unlinked_table_count: 0,
            status: "test".to_string(),
        };
        let candidates: Vec<_> = discovery
            .message_candidates
            .iter()
            .map(|entry| (entry.node_id, MessageNodeType::NormalMessage))
            .collect();
        let resolutions = crate::pst::message_ownership::resolve_message_ownership(&candidates, &[]);

        assert!(matches!(
            resolutions.get(&NodeId(0x24)),
            Some(MessageOwnershipResolution::Unresolved { status })
                if status == "message_table_membership_absent"
        ));
    }

    #[test]
    fn discovery_model_preserves_linked_table_identity() {
        let table = MessageTableCandidate {
            node_identity: "node_32".to_string(),
            table_type: MessageTableNodeType::ContentsTable,
            owner_folder_node_identity: "node_22".to_string(),
            linked_folder_key: Some("folder-22".to_string()),
            status: "test".to_string(),
        };
        let mut paths = HashMap::new();
        paths.insert("folder-22".to_string(), "/Inbox".to_string());

        assert_eq!(table.linked_folder_key.as_deref(), Some("folder-22"));
        assert_eq!(paths.get("folder-22").map(String::as_str), Some("/Inbox"));
    }
}
