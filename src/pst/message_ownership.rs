use std::collections::{HashMap, HashSet};

use crate::pst::message_table::{MessageNodeType, MessageTableNodeType};
use crate::pst::primitives::NodeId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageMembershipEvidence {
    pub message_node_id: NodeId,
    pub message_node_type: MessageNodeType,
    pub table_node_type: MessageTableNodeType,
    pub folder_key: String,
    pub folder_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedMessageOwner {
    pub folder_key: String,
    pub folder_path: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageOwnershipResolution {
    Resolved(ResolvedMessageOwner),
    Unresolved { status: String },
    Ambiguous { status: String },
}

pub fn resolve_message_ownership(
    message_candidates: &[(NodeId, MessageNodeType)],
    evidence: &[MessageMembershipEvidence],
) -> HashMap<NodeId, MessageOwnershipResolution> {
    let candidates: HashMap<NodeId, MessageNodeType> = message_candidates.iter().copied().collect();
    let mut grouped: HashMap<NodeId, Vec<&MessageMembershipEvidence>> = HashMap::new();

    for item in evidence {
        grouped.entry(item.message_node_id).or_default().push(item);
    }

    let mut output = HashMap::new();
    for (node_id, node_type) in message_candidates {
        let Some(items) = grouped.get(node_id) else {
            output.insert(
                *node_id,
                MessageOwnershipResolution::Unresolved {
                    status: "message_table_membership_absent".to_string(),
                },
            );
            continue;
        };

        if items
            .iter()
            .any(|item| !candidates.contains_key(&item.message_node_id))
        {
            output.insert(
                *node_id,
                MessageOwnershipResolution::Unresolved {
                    status: "message_table_member_not_in_nbt_candidates".to_string(),
                },
            );
            continue;
        }

        let physical: Vec<_> = items
            .iter()
            .copied()
            .filter(|item| {
                matches!(
                    item.table_node_type,
                    MessageTableNodeType::ContentsTable
                        | MessageTableNodeType::AssociatedContentsTable
                )
            })
            .collect();

        if physical.is_empty() {
            output.insert(
                *node_id,
                MessageOwnershipResolution::Unresolved {
                    status: "message_table_membership_search_only_or_unsupported".to_string(),
                },
            );
            continue;
        }

        if physical
            .iter()
            .any(|item| item.message_node_type != *node_type)
            || physical.iter().any(|item| {
                !matches!(
                    (item.message_node_type, item.table_node_type),
                    (
                        MessageNodeType::NormalMessage,
                        MessageTableNodeType::ContentsTable
                    ) | (
                        MessageNodeType::AssociatedMessage,
                        MessageTableNodeType::AssociatedContentsTable
                    )
                )
            })
        {
            output.insert(
                *node_id,
                MessageOwnershipResolution::Unresolved {
                    status: "message_table_membership_node_type_mismatch".to_string(),
                },
            );
            continue;
        }

        let unique_owners: HashSet<_> = physical
            .iter()
            .map(|item| (item.folder_key.as_str(), item.folder_path.as_str()))
            .collect();
        if unique_owners.len() != 1 || physical.len() != 1 {
            output.insert(
                *node_id,
                MessageOwnershipResolution::Ambiguous {
                    status: format!(
                        "message_table_membership_ambiguous; physical_rows={}; unique_owners={}",
                        physical.len(),
                        unique_owners.len()
                    ),
                },
            );
            continue;
        }

        let owner = physical[0];
        output.insert(
            *node_id,
            MessageOwnershipResolution::Resolved(ResolvedMessageOwner {
                folder_key: owner.folder_key.clone(),
                folder_path: owner.folder_path.clone(),
                status: format!(
                    "message_table_membership_exact; table_type={}",
                    owner.table_node_type.status_label()
                ),
            }),
        );
    }

    output
}

#[cfg(test)]
mod tests {
    use super::{resolve_message_ownership, MessageMembershipEvidence, MessageOwnershipResolution};
    use crate::pst::message_table::{MessageNodeType, MessageTableNodeType};
    use crate::pst::primitives::NodeId;

    #[test]
    fn resolves_one_exact_physical_owner() {
        let result = resolve_message_ownership(
            &[(NodeId(0x24), MessageNodeType::NormalMessage)],
            &[evidence(
                0x24,
                MessageNodeType::NormalMessage,
                MessageTableNodeType::ContentsTable,
                "folder-a",
            )],
        );

        let MessageOwnershipResolution::Resolved(owner) = &result[&NodeId(0x24)] else {
            panic!("expected resolved owner");
        };
        assert_eq!(owner.folder_key, "folder-a");
        assert_eq!(owner.folder_path, "/Inbox");
        assert_eq!(
            owner.status,
            "message_table_membership_exact; table_type=contents_table"
        );
    }

    #[test]
    fn rejects_duplicate_rows_even_when_owner_matches() {
        let row = evidence(
            0x24,
            MessageNodeType::NormalMessage,
            MessageTableNodeType::ContentsTable,
            "folder-a",
        );
        let result = resolve_message_ownership(
            &[(NodeId(0x24), MessageNodeType::NormalMessage)],
            &[row.clone(), row],
        );

        assert!(matches!(
            &result[&NodeId(0x24)],
            MessageOwnershipResolution::Ambiguous { status }
                if status == "message_table_membership_ambiguous; physical_rows=2; unique_owners=1"
        ));
    }

    #[test]
    fn rejects_multiple_physical_folder_owners() {
        let result = resolve_message_ownership(
            &[(NodeId(0x24), MessageNodeType::NormalMessage)],
            &[
                evidence(
                    0x24,
                    MessageNodeType::NormalMessage,
                    MessageTableNodeType::ContentsTable,
                    "folder-a",
                ),
                evidence(
                    0x24,
                    MessageNodeType::NormalMessage,
                    MessageTableNodeType::ContentsTable,
                    "folder-b",
                ),
            ],
        );

        assert!(matches!(
            &result[&NodeId(0x24)],
            MessageOwnershipResolution::Ambiguous { status }
                if status == "message_table_membership_ambiguous; physical_rows=2; unique_owners=2"
        ));
    }

    #[test]
    fn search_membership_does_not_establish_physical_ownership() {
        let result = resolve_message_ownership(
            &[(NodeId(0x24), MessageNodeType::NormalMessage)],
            &[evidence(
                0x24,
                MessageNodeType::NormalMessage,
                MessageTableNodeType::SearchContentsTable,
                "search-folder",
            )],
        );

        assert!(matches!(
            &result[&NodeId(0x24)],
            MessageOwnershipResolution::Unresolved { status }
                if status == "message_table_membership_search_only_or_unsupported"
        ));
    }

    #[test]
    fn rejects_wrong_message_and_table_type_pairing() {
        let result = resolve_message_ownership(
            &[(NodeId(0x24), MessageNodeType::NormalMessage)],
            &[evidence(
                0x24,
                MessageNodeType::NormalMessage,
                MessageTableNodeType::AssociatedContentsTable,
                "folder-a",
            )],
        );

        assert!(matches!(
            &result[&NodeId(0x24)],
            MessageOwnershipResolution::Unresolved { status }
                if status == "message_table_membership_node_type_mismatch"
        ));
    }

    #[test]
    fn missing_membership_is_explicitly_unresolved() {
        let result = resolve_message_ownership(&[(NodeId(0x24), MessageNodeType::NormalMessage)], &[]);

        assert!(matches!(
            &result[&NodeId(0x24)],
            MessageOwnershipResolution::Unresolved { status }
                if status == "message_table_membership_absent"
        ));
    }

    fn evidence(
        node_id: u64,
        message_node_type: MessageNodeType,
        table_node_type: MessageTableNodeType,
        folder_key: &str,
    ) -> MessageMembershipEvidence {
        MessageMembershipEvidence {
            message_node_id: NodeId(node_id),
            message_node_type,
            table_node_type,
            folder_key: folder_key.to_string(),
            folder_path: "/Inbox".to_string(),
        }
    }
}
