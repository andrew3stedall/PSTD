use std::collections::{HashMap, HashSet};

use crate::output::ids;
use crate::output::metadata::{
    EnvelopeRecordKind, FolderRecord, ItemEnvelope, ItemEnvelopeSource, ItemKind, ItemVisibility,
    MessageRecord,
};
use crate::pst::message_ownership::MessageOwnershipResolution;
use crate::pst::message_table::{
    message_node_type, message_table_node_type, node_identity, MessageNodeType,
};
use crate::pst::nbt::NbtEntry;
use crate::pst::primitives::NodeId;

pub fn build_item_envelopes(
    pst_id: &str,
    folders: &[FolderRecord],
    nbt_entries: &[NbtEntry],
    message_candidates: &[NbtEntry],
    ownership: &HashMap<NodeId, MessageOwnershipResolution>,
    messages: &[MessageRecord],
) -> Vec<ItemEnvelope> {
    let mut envelopes = Vec::new();
    let mut seen_keys = HashSet::new();
    let messages_by_node: HashMap<_, _> = messages
        .iter()
        .filter_map(|message| {
            message
                .message_node_id
                .as_ref()
                .map(|node_id| (node_id.as_str(), message))
        })
        .collect();

    for (ordinal, folder) in folders.iter().enumerate() {
        let node_identity = folder.folder_node_id.clone();
        let envelope = ItemEnvelope {
            envelope_key: folder.folder_key.clone(),
            record_kind: EnvelopeRecordKind::Folder,
            source: ItemEnvelopeSource {
                pst_id: pst_id.to_string(),
                descriptor_id: node_identity.clone(),
                node_id: node_identity,
                folder_id: Some(folder.folder_key.clone()),
                ordinal: ordinal as u64,
            },
            parent_envelope_key: folder.parent_folder_key.clone(),
            child_envelope_keys: Vec::new(),
            folder_path: folder.folder_path.clone(),
            visibility: ItemVisibility::Visible,
            item_kind: None,
            message_class: None,
            classification_confidence: "folder_source_record".to_string(),
            provenance_status: folder.status.clone(),
            extraction_status: "folder_discovered".to_string(),
            raw_evidence_refs: vec![format!("folder_record:{}", folder.folder_key)],
        };
        insert_envelope(&mut envelopes, &mut seen_keys, envelope);
    }

    for (ordinal, entry) in message_candidates.iter().enumerate() {
        let source_node_id = node_identity(entry.node_id);
        let envelope_key = ids::message_key(pst_id, &source_node_id);
        let (folder_key, folder_path, provenance_status, extraction_status) =
            match ownership.get(&entry.node_id) {
                Some(MessageOwnershipResolution::Resolved(owner)) => {
                    let extraction_status = messages_by_node
                        .get(source_node_id.as_str())
                        .map(|message| message.extraction_status.clone())
                        .unwrap_or_else(|| "item_metadata_unavailable".to_string());
                    (
                        Some(owner.folder_key.clone()),
                        owner.folder_path.clone(),
                        owner.status.clone(),
                        extraction_status,
                    )
                }
                Some(MessageOwnershipResolution::Unresolved { status }) => (
                    None,
                    String::new(),
                    status.clone(),
                    "unavailable_unresolved_owner".to_string(),
                ),
                Some(MessageOwnershipResolution::Ambiguous { status }) => (
                    None,
                    String::new(),
                    status.clone(),
                    "unavailable_ambiguous_owner".to_string(),
                ),
                None => (
                    None,
                    String::new(),
                    "message_table_membership_absent".to_string(),
                    "unavailable_unresolved_owner".to_string(),
                ),
            };
        let (visibility, item_kind, confidence) = match message_node_type(entry.node_id) {
            Some(MessageNodeType::AssociatedMessage) => (
                ItemVisibility::Associated,
                ItemKind::Note,
                "node_type_only_associated",
            ),
            Some(MessageNodeType::NormalMessage) => {
                (ItemVisibility::Visible, ItemKind::Note, "node_type_only")
            }
            None => (
                ItemVisibility::Unknown,
                ItemKind::Other,
                "node_type_unavailable",
            ),
        };
        let envelope = ItemEnvelope {
            envelope_key,
            record_kind: EnvelopeRecordKind::Item,
            source: ItemEnvelopeSource {
                pst_id: pst_id.to_string(),
                descriptor_id: Some(source_node_id.clone()),
                node_id: Some(source_node_id.clone()),
                folder_id: folder_key.clone(),
                ordinal: ordinal as u64,
            },
            parent_envelope_key: folder_key,
            child_envelope_keys: Vec::new(),
            folder_path,
            visibility,
            item_kind: Some(item_kind),
            message_class: None,
            classification_confidence: confidence.to_string(),
            provenance_status,
            extraction_status,
            raw_evidence_refs: vec![
                format!("nbt:{}", source_node_id),
                format!("message_candidate:{}", source_node_id),
            ],
        };
        insert_envelope(&mut envelopes, &mut seen_keys, envelope);
    }

    let message_node_ids: HashSet<_> = message_candidates
        .iter()
        .map(|entry| entry.node_id)
        .collect();
    for (ordinal, entry) in nbt_entries.iter().enumerate() {
        if message_node_ids.contains(&entry.node_id)
            || crate::pst::folder_tree::is_folder_candidate(entry)
            || message_table_node_type(entry.node_id).is_some()
        {
            continue;
        }
        let source_node_id = node_identity(entry.node_id);
        let envelope = ItemEnvelope {
            envelope_key: ids::stable_id("item", &[pst_id, &source_node_id]),
            record_kind: EnvelopeRecordKind::Item,
            source: ItemEnvelopeSource {
                pst_id: pst_id.to_string(),
                descriptor_id: Some(source_node_id.clone()),
                node_id: Some(source_node_id.clone()),
                folder_id: None,
                ordinal: ordinal as u64,
            },
            parent_envelope_key: None,
            child_envelope_keys: Vec::new(),
            folder_path: String::new(),
            visibility: ItemVisibility::Unknown,
            item_kind: Some(ItemKind::Other),
            message_class: None,
            classification_confidence: "node_type_unclassified".to_string(),
            provenance_status: "source_nbt_entry_not_message_or_folder".to_string(),
            extraction_status: "skipped_unclassified_source_entry".to_string(),
            raw_evidence_refs: vec![format!("nbt:{}", source_node_id)],
        };
        insert_envelope(&mut envelopes, &mut seen_keys, envelope);
    }

    for message in messages {
        let Some(source_node_id) = message.message_node_id.as_deref() else {
            continue;
        };
        let envelope_key = message.message_key.clone();
        if seen_keys.contains(&envelope_key) {
            continue;
        }
        let item_kind = if message.item_type == "embedded_message_metadata" {
            ItemKind::Note
        } else {
            ItemKind::Note
        };
        let envelope = ItemEnvelope {
            envelope_key,
            record_kind: EnvelopeRecordKind::Item,
            source: ItemEnvelopeSource {
                pst_id: pst_id.to_string(),
                descriptor_id: Some(source_node_id.to_string()),
                node_id: Some(source_node_id.to_string()),
                folder_id: (!message.folder_key.is_empty()).then(|| message.folder_key.clone()),
                ordinal: messages.len() as u64,
            },
            parent_envelope_key: (!message.folder_key.is_empty())
                .then(|| message.folder_key.clone()),
            child_envelope_keys: Vec::new(),
            folder_path: message.folder_path.clone(),
            visibility: ItemVisibility::Visible,
            item_kind: Some(item_kind),
            message_class: Some(message.item_type.clone()),
            classification_confidence: "message_record_fallback".to_string(),
            provenance_status: "message_record_source".to_string(),
            extraction_status: message.extraction_status.clone(),
            raw_evidence_refs: vec![format!("message_record:{}", message.message_key)],
        };
        insert_envelope(&mut envelopes, &mut seen_keys, envelope);
    }

    reconcile_folder_relationships(&mut envelopes);
    envelopes.sort_by(|left, right| left.envelope_key.cmp(&right.envelope_key));
    envelopes
}

fn insert_envelope(
    envelopes: &mut Vec<ItemEnvelope>,
    seen_keys: &mut HashSet<String>,
    envelope: ItemEnvelope,
) {
    if seen_keys.insert(envelope.envelope_key.clone()) {
        envelopes.push(envelope);
        return;
    }

    let duplicate_key = ids::stable_id(
        "item_duplicate",
        &[
            &envelope.envelope_key,
            &envelope.source.ordinal.to_string(),
            envelope.source.node_id.as_deref().unwrap_or("none"),
        ],
    );
    let mut duplicate = envelope;
    duplicate.envelope_key = duplicate_key;
    duplicate.provenance_status = "duplicate_source_identity".to_string();
    duplicate.extraction_status = "failed_duplicate_source_identity".to_string();
    envelopes.push(duplicate);
}

fn reconcile_folder_relationships(envelopes: &mut [ItemEnvelope]) {
    let mut children_by_parent: HashMap<String, Vec<String>> = HashMap::new();
    for envelope in envelopes.iter() {
        if envelope.record_kind == EnvelopeRecordKind::Folder {
            if let Some(parent) = &envelope.parent_envelope_key {
                children_by_parent
                    .entry(parent.clone())
                    .or_default()
                    .push(envelope.envelope_key.clone());
            }
        }
    }
    for children in children_by_parent.values_mut() {
        children.sort();
    }

    let mut paths: HashMap<(Option<String>, String), Vec<usize>> = HashMap::new();
    for (index, envelope) in envelopes.iter().enumerate() {
        if envelope.record_kind == EnvelopeRecordKind::Folder {
            paths
                .entry((
                    envelope.parent_envelope_key.clone(),
                    envelope.folder_path.clone(),
                ))
                .or_default()
                .push(index);
        }
    }

    for (index, envelope) in envelopes.iter_mut().enumerate() {
        if envelope.record_kind == EnvelopeRecordKind::Folder {
            envelope.child_envelope_keys = children_by_parent
                .get(&envelope.envelope_key)
                .cloned()
                .unwrap_or_default();
            if paths
                .values()
                .any(|indices| indices.len() > 1 && indices.contains(&index))
            {
                envelope
                    .provenance_status
                    .push_str("; folder_path_collision");
                envelope.extraction_status = "failed_folder_path_collision".to_string();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::build_item_envelopes;
    use crate::output::metadata::{EnvelopeRecordKind, FolderRecord, ItemKind, ItemVisibility};
    use crate::pst::message_ownership::MessageOwnershipResolution;
    use crate::pst::nbt::NbtEntry;
    use crate::pst::primitives::{BlockId, NodeId};

    fn folder(name: &str, key: &str, node: Option<&str>) -> FolderRecord {
        FolderRecord {
            pst_id: "pst-test".to_string(),
            folder_key: key.to_string(),
            parent_folder_key: None,
            folder_path: format!("/{name}"),
            folder_name: name.to_string(),
            folder_node_id: node.map(str::to_string),
            item_count_total: None,
            child_folder_count: None,
            status: "folder_test".to_string(),
        }
    }

    fn entry(node_id: u64) -> NbtEntry {
        NbtEntry {
            node_id: NodeId(node_id),
            data_block_id: BlockId(0),
            subnode_block_id: None,
        }
    }

    #[test]
    fn preserves_source_identity_for_duplicate_display_names() {
        let folders = vec![
            folder("same", "folder-a", Some("node_22")),
            folder("same", "folder-b", Some("node_42")),
        ];
        let candidates = vec![entry(0x24), entry(0x44)];
        let mut ownership = HashMap::new();
        for node_id in [0x24, 0x44] {
            ownership.insert(
                NodeId(node_id),
                MessageOwnershipResolution::Unresolved {
                    status: "message_table_membership_absent".to_string(),
                },
            );
        }
        let envelopes = build_item_envelopes(
            "pst-test",
            &folders,
            &candidates,
            &candidates,
            &ownership,
            &[],
        );
        let keys: Vec<_> = envelopes
            .iter()
            .filter(|envelope| envelope.record_kind == EnvelopeRecordKind::Item)
            .map(|envelope| envelope.envelope_key.clone())
            .collect();
        assert_eq!(keys.len(), 2);
        assert_ne!(keys[0], keys[1]);
        assert!(envelopes
            .iter()
            .filter(|envelope| envelope.record_kind == EnvelopeRecordKind::Item)
            .all(|envelope| {
                envelope.item_kind == Some(ItemKind::Note)
                    && envelope.visibility == ItemVisibility::Visible
            }));
    }

    #[test]
    fn preserves_ambiguous_ownership_as_unavailable() {
        let candidates = vec![entry(0x24)];
        let mut ownership = HashMap::new();
        ownership.insert(
            NodeId(0x24),
            MessageOwnershipResolution::Ambiguous {
                status: "message_table_membership_ambiguous".to_string(),
            },
        );
        let envelopes =
            build_item_envelopes("pst-test", &[], &candidates, &candidates, &ownership, &[]);
        let envelope = envelopes
            .iter()
            .find(|envelope| envelope.source.node_id.as_deref() == Some("node_24"))
            .unwrap();
        assert_eq!(envelope.extraction_status, "unavailable_ambiguous_owner");
        assert_eq!(envelope.parent_envelope_key, None);
    }

    #[test]
    fn classifies_unmapped_source_entries_as_explicit_other() {
        let entries = vec![entry(0x21)];
        let envelopes = build_item_envelopes("pst-test", &[], &entries, &[], &HashMap::new(), &[]);
        let envelope = &envelopes[0];
        assert_eq!(envelope.item_kind, Some(ItemKind::Other));
        assert_eq!(envelope.visibility, ItemVisibility::Unknown);
        assert_eq!(
            envelope.extraction_status,
            "skipped_unclassified_source_entry"
        );
    }
}
