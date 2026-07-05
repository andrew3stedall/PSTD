use std::collections::{HashMap, HashSet};

use crate::pst::nbt::{NbtEntry, NbtIndex};
use crate::pst::primitives::NodeId;

const NID_TYPE_MASK: u64 = 0x1f;
const NID_TYPE_NORMAL_MESSAGE: u64 = 0x04;
const NID_TYPE_ASSOC_MESSAGE: u64 = 0x08;
const NID_TYPE_CONTENTS_TABLE: u64 = 0x12;
const NID_TYPE_ASSOC_CONTENTS_TABLE: u64 = 0x13;
const NID_TYPE_SEARCH_CONTENTS_TABLE: u64 = 0x0d;
const NID_TYPE_HIERARCHY_TABLE: u64 = 0x11;
const NID_TYPE_NORMAL_FOLDER: u64 = 0x02;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageNodeType {
    NormalMessage,
    AssociatedMessage,
}

impl MessageNodeType {
    pub fn status_label(self) -> &'static str {
        match self {
            Self::NormalMessage => "normal_message",
            Self::AssociatedMessage => "associated_message",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageTableNodeType {
    ContentsTable,
    AssociatedContentsTable,
    SearchContentsTable,
    HierarchyTable,
}

impl MessageTableNodeType {
    pub fn status_label(self) -> &'static str {
        match self {
            Self::ContentsTable => "contents_table",
            Self::AssociatedContentsTable => "associated_contents_table",
            Self::SearchContentsTable => "search_contents_table",
            Self::HierarchyTable => "hierarchy_table",
        }
    }

    pub fn describes_message_membership(self) -> bool {
        matches!(
            self,
            Self::ContentsTable | Self::AssociatedContentsTable | Self::SearchContentsTable
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageCandidate {
    pub node_identity: String,
    pub node_type: MessageNodeType,
    pub membership_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageTableCandidate {
    pub node_identity: String,
    pub table_type: MessageTableNodeType,
    pub owner_folder_node_identity: String,
    pub linked_folder_key: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct MessageTableDiscovery {
    pub message_candidates: Vec<NbtEntry>,
    pub table_candidates: Vec<MessageTableCandidate>,
    pub linked_table_count: usize,
    pub unlinked_table_count: usize,
    pub status: String,
}

impl MessageTableDiscovery {
    pub fn message_candidate_count(&self) -> usize {
        self.message_candidates.len()
    }

    pub fn message_membership_table_count(&self) -> usize {
        self.table_candidates
            .iter()
            .filter(|candidate| candidate.table_type.describes_message_membership())
            .count()
    }

    pub fn message_membership_status(&self) -> String {
        if self.message_membership_table_count() == 0 {
            "message_table_membership_not_found".to_string()
        } else if self.linked_table_count == 0 {
            format!(
                "message_table_candidates_unlinked; table_candidates={}",
                self.message_membership_table_count()
            )
        } else {
            format!(
                "message_table_candidates_linked; linked_tables={}; unlinked_tables={}",
                self.linked_table_count, self.unlinked_table_count
            )
        }
    }
}

pub fn discover_message_tables(
    nbt: &NbtIndex,
    folder_key_by_node_identity: &HashMap<String, String>,
) -> MessageTableDiscovery {
    let mut message_candidates = Vec::new();
    let mut table_candidates = Vec::new();
    let mut linked_table_count = 0usize;
    let mut unlinked_table_count = 0usize;
    let mut seen_messages = HashSet::new();
    let mut seen_tables = HashSet::new();

    for entry in &nbt.entries {
        if is_message_candidate(entry) && seen_messages.insert(entry.node_id) {
            message_candidates.push(entry.clone());
        }

        if let Some(table_type) = message_table_node_type(entry.node_id) {
            let node_identity = node_identity(entry.node_id);
            if !seen_tables.insert(node_identity.clone()) {
                continue;
            }
            let owner_folder_node_identity = owner_folder_node_identity(entry.node_id);
            let linked_folder_key = folder_key_by_node_identity
                .get(&owner_folder_node_identity)
                .cloned();
            let link_status = if linked_folder_key.is_some() {
                linked_table_count += 1;
                "owner_folder_linked"
            } else {
                unlinked_table_count += 1;
                "owner_folder_unmatched"
            };
            table_candidates.push(MessageTableCandidate {
                node_identity,
                table_type,
                owner_folder_node_identity,
                linked_folder_key,
                status: format!(
                    "message_table_candidate_from_nbt; table_type={}; {link_status}",
                    table_type.status_label()
                ),
            });
        }
    }

    let status = format!(
        "message_table_discovery; message_candidates={}; table_candidates={}; linked_tables={}; unlinked_tables={}",
        message_candidates.len(),
        table_candidates.len(),
        linked_table_count,
        unlinked_table_count
    );

    MessageTableDiscovery {
        message_candidates,
        table_candidates,
        linked_table_count,
        unlinked_table_count,
        status,
    }
}

pub fn classify_message_candidate(entry: &NbtEntry) -> Option<MessageCandidate> {
    message_node_type(entry.node_id).map(|node_type| MessageCandidate {
        node_identity: node_identity(entry.node_id),
        node_type,
        membership_status: format!(
            "message_candidate_from_nbt; node_type={}; message_table_membership_pending",
            node_type.status_label()
        ),
    })
}

pub fn is_message_candidate(entry: &NbtEntry) -> bool {
    classify_message_candidate(entry).is_some()
}

pub fn message_node_type(node_id: NodeId) -> Option<MessageNodeType> {
    match node_type_code(node_id) {
        NID_TYPE_NORMAL_MESSAGE => Some(MessageNodeType::NormalMessage),
        NID_TYPE_ASSOC_MESSAGE => Some(MessageNodeType::AssociatedMessage),
        _ => None,
    }
}

pub fn message_table_node_type(node_id: NodeId) -> Option<MessageTableNodeType> {
    match node_type_code(node_id) {
        NID_TYPE_CONTENTS_TABLE => Some(MessageTableNodeType::ContentsTable),
        NID_TYPE_ASSOC_CONTENTS_TABLE => Some(MessageTableNodeType::AssociatedContentsTable),
        NID_TYPE_SEARCH_CONTENTS_TABLE => Some(MessageTableNodeType::SearchContentsTable),
        NID_TYPE_HIERARCHY_TABLE => Some(MessageTableNodeType::HierarchyTable),
        _ => None,
    }
}

pub fn node_type_code(node_id: NodeId) -> u64 {
    node_id.0 & NID_TYPE_MASK
}

pub fn node_identity(node_id: NodeId) -> String {
    format!("node_{:x}", node_id.0)
}

pub fn owner_folder_node_identity(table_node_id: NodeId) -> String {
    let folder_node_id = (table_node_id.0 & !NID_TYPE_MASK) | NID_TYPE_NORMAL_FOLDER;
    format!("node_{folder_node_id:x}")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        classify_message_candidate, discover_message_tables, message_node_type,
        message_table_node_type, owner_folder_node_identity, MessageNodeType, MessageTableNodeType,
    };
    use crate::pst::nbt::{NbtEntry, NbtIndex};
    use crate::pst::primitives::{BlockId, NodeId};

    #[test]
    fn classifies_message_like_nbt_entries_by_decoded_node_type() {
        let normal_message = entry(0x24);
        let assoc_message = entry(0x28);
        let folder = entry(0x22);

        assert_eq!(
            message_node_type(normal_message.node_id),
            Some(MessageNodeType::NormalMessage)
        );
        assert_eq!(
            message_node_type(assoc_message.node_id),
            Some(MessageNodeType::AssociatedMessage)
        );
        assert_eq!(message_node_type(folder.node_id), None);
        assert_eq!(
            classify_message_candidate(&normal_message)
                .unwrap()
                .membership_status,
            "message_candidate_from_nbt; node_type=normal_message; message_table_membership_pending"
        );
        assert!(classify_message_candidate(&folder).is_none());
    }

    #[test]
    fn classifies_message_table_entries_and_owner_folder_identity() {
        let contents_table = entry(0x32);
        let assoc_contents_table = entry(0x33);
        let hierarchy_table = entry(0x31);

        assert_eq!(
            message_table_node_type(contents_table.node_id),
            Some(MessageTableNodeType::ContentsTable)
        );
        assert_eq!(
            message_table_node_type(assoc_contents_table.node_id),
            Some(MessageTableNodeType::AssociatedContentsTable)
        );
        assert_eq!(
            message_table_node_type(hierarchy_table.node_id),
            Some(MessageTableNodeType::HierarchyTable)
        );
        assert_eq!(owner_folder_node_identity(contents_table.node_id), "node_22");
    }

    #[test]
    fn discovers_message_and_table_candidates_with_folder_links() {
        let nbt = nbt_index(vec![entry(0x22), entry(0x24), entry(0x32), entry(0x52)]);
        let mut folder_keys = HashMap::new();
        folder_keys.insert("node_22".to_string(), "folder-key-22".to_string());

        let discovery = discover_message_tables(&nbt, &folder_keys);

        assert_eq!(discovery.message_candidate_count(), 1);
        assert_eq!(discovery.table_candidates.len(), 2);
        assert_eq!(discovery.linked_table_count, 1);
        assert_eq!(discovery.unlinked_table_count, 1);
        assert_eq!(
            discovery.table_candidates[0].linked_folder_key.as_deref(),
            Some("folder-key-22")
        );
        assert_eq!(
            discovery.message_membership_status(),
            "message_table_candidates_linked; linked_tables=1; unlinked_tables=1"
        );
    }

    fn entry(node_id: u64) -> NbtEntry {
        NbtEntry {
            node_id: NodeId(node_id),
            data_block_id: BlockId(0x1000),
            subnode_block_id: None,
        }
    }

    fn nbt_index(entries: Vec<NbtEntry>) -> NbtIndex {
        NbtIndex {
            root: None,
            entries,
            parsed_pages: 1,
            discovered_child_pages: 0,
            traversal_error_count: 0,
            duplicate_entry_count: 0,
            truncated_entry_count: 0,
            page_diagnostics: Vec::new(),
            status: "test".to_string(),
        }
    }
}
