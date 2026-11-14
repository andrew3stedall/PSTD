use std::collections::HashMap;

use crate::output::ids;
use crate::output::metadata::FolderRecord;
use crate::pst::header::PstHeader;
use crate::pst::mapi::{PR_CONTENT_COUNT, PR_DISPLAY_NAME, PR_DISPLAY_NAME_A};
use crate::pst::nbt::NbtEntry;
use crate::pst::primitives::NodeId;
use crate::pst::property_context::PropertyContext;

const NID_TYPE_MASK: u64 = 0x1f;
const NID_TYPE_NORMAL_FOLDER: u64 = 0x02;
const NID_TYPE_SEARCH_FOLDER: u64 = 0x03;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderNodeType {
    NormalFolder,
    SearchFolder,
}

impl FolderNodeType {
    pub fn status_label(self) -> &'static str {
        match self {
            Self::NormalFolder => "normal_folder",
            Self::SearchFolder => "search_folder",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderCandidate {
    pub node_identity: String,
    pub node_type: FolderNodeType,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FolderInventoryRecord {
    pub pst_id: String,
    pub folder_key: String,
    pub folder_path: String,
    pub item_count_total: Option<u64>,
    pub item_count_email: Option<u64>,
    pub item_count_unknown: Option<u64>,
    pub child_folder_count: Option<u64>,
    pub inventory_status: String,
}

pub fn root_folder_from_header(
    pst_id: &str,
    header: &PstHeader,
) -> (FolderRecord, FolderInventoryRecord) {
    let folder_key = ids::folder_key(pst_id, "root");
    let status = format!("metadata_root_from_{}", header.summary.parser_status);
    let folder = FolderRecord {
        pst_id: pst_id.to_string(),
        folder_key: folder_key.clone(),
        parent_folder_key: None,
        folder_path: "/".to_string(),
        folder_name: "root".to_string(),
        folder_node_id: None,
        item_count_total: None,
        child_folder_count: None,
        status: status.clone(),
    };
    let inventory = FolderInventoryRecord {
        pst_id: pst_id.to_string(),
        folder_key,
        folder_path: "/".to_string(),
        item_count_total: None,
        item_count_email: None,
        item_count_unknown: None,
        child_folder_count: None,
        inventory_status: status,
    };
    (folder, inventory)
}

pub fn classify_folder_candidate(entry: &NbtEntry) -> Option<FolderCandidate> {
    folder_node_type(entry.node_id).map(|node_type| {
        let node_identity = folder_identity(entry);
        FolderCandidate {
            node_identity,
            node_type,
            status: format!(
                "folder_candidate_from_nbt; node_type={}",
                node_type.status_label()
            ),
        }
    })
}

pub fn folder_node_type(node_id: NodeId) -> Option<FolderNodeType> {
    match node_id.0 & NID_TYPE_MASK {
        NID_TYPE_NORMAL_FOLDER => Some(FolderNodeType::NormalFolder),
        NID_TYPE_SEARCH_FOLDER => Some(FolderNodeType::SearchFolder),
        _ => None,
    }
}

pub fn is_folder_candidate(entry: &NbtEntry) -> bool {
    classify_folder_candidate(entry).is_some()
}

pub fn folder_identity(entry: &NbtEntry) -> String {
    format!("node_{:x}", entry.node_id.0)
}

pub fn folder_from_nbt_candidate(
    pst_id: &str,
    entry: &NbtEntry,
    parent_folder_key: Option<String>,
    properties: Option<&PropertyContext>,
) -> (FolderRecord, FolderInventoryRecord) {
    let candidate = classify_folder_candidate(entry).unwrap_or_else(|| FolderCandidate {
        node_identity: folder_identity(entry),
        node_type: FolderNodeType::NormalFolder,
        status: "folder_candidate_from_nbt; node_type=unknown".to_string(),
    });
    let folder_key = ids::folder_key(pst_id, &candidate.node_identity);
    let folder_name = properties
        .and_then(|values| {
            values
                .string_value(PR_DISPLAY_NAME)
                .or_else(|| values.string_value(PR_DISPLAY_NAME_A))
        })
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| candidate.node_identity.clone());
    let item_count = properties
        .and_then(|values| values.string_value(PR_CONTENT_COUNT))
        .and_then(|value| value.parse::<u64>().ok());
    let folder_path = format!("/{}", safe_folder_path_segment(&folder_name));
    let property_status = if properties.is_some() {
        "property_context_loaded"
    } else {
        "property_context_unavailable"
    };
    let status = format!("{}; {property_status}", candidate.status);
    let folder = FolderRecord {
        pst_id: pst_id.to_string(),
        folder_key: folder_key.clone(),
        parent_folder_key,
        folder_path: folder_path.clone(),
        folder_name,
        folder_node_id: Some(candidate.node_identity),
        item_count_total: item_count,
        child_folder_count: None,
        status: status.clone(),
    };
    let inventory = FolderInventoryRecord {
        pst_id: pst_id.to_string(),
        folder_key,
        folder_path,
        item_count_total: item_count,
        item_count_email: item_count,
        item_count_unknown: None,
        child_folder_count: None,
        inventory_status: status,
    };
    (folder, inventory)
}

pub fn folder_from_properties(
    pst_id: &str,
    folder_identity: &str,
    parent_folder_key: Option<String>,
    properties: &PropertyContext,
) -> (FolderRecord, FolderInventoryRecord) {
    let folder_key = ids::folder_key(pst_id, folder_identity);
    let folder_name = properties
        .string_value(PR_DISPLAY_NAME)
        .or_else(|| properties.string_value(PR_DISPLAY_NAME_A))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| folder_identity.to_string());
    let item_count = properties
        .string_value(PR_CONTENT_COUNT)
        .and_then(|value| value.parse::<u64>().ok());
    let folder_path = format!("/{}", folder_name.trim_matches('/'));
    let folder = FolderRecord {
        pst_id: pst_id.to_string(),
        folder_key: folder_key.clone(),
        parent_folder_key,
        folder_path: folder_path.clone(),
        folder_name,
        folder_node_id: Some(folder_identity.to_string()),
        item_count_total: item_count,
        child_folder_count: None,
        status: "metadata_from_properties".to_string(),
    };
    let inventory = FolderInventoryRecord {
        pst_id: pst_id.to_string(),
        folder_key,
        folder_path,
        item_count_total: item_count,
        item_count_email: item_count,
        item_count_unknown: None,
        child_folder_count: None,
        inventory_status: "metadata_from_properties".to_string(),
    };
    (folder, inventory)
}

/// Rebuild folder nesting from each folder node's NBT parent.
///
/// `parent_by_identity` maps a folder's node identity to its parent's. A folder
/// whose parent is another discovered folder is placed under it; a folder that is
/// its own parent is the mailbox root and adds no path segment. A folder whose
/// parent is unknown, or whose parent chain loops, keeps its flat path and parent.
pub fn apply_folder_nesting(
    folders: &mut [FolderRecord],
    inventory: &mut [FolderInventoryRecord],
    parent_by_identity: &HashMap<String, String>,
) {
    let index_by_identity: HashMap<&str, usize> = folders
        .iter()
        .enumerate()
        .filter_map(|(index, folder)| Some((folder.folder_node_id.as_deref()?, index)))
        .collect();
    let parent_index = |index: usize| -> Option<usize> {
        let identity = folders[index].folder_node_id.as_deref()?;
        let parent = *index_by_identity.get(parent_by_identity.get(identity)?.as_str())?;
        (parent != index).then_some(parent)
    };
    let is_mailbox_root = |index: usize| {
        folders[index]
            .folder_node_id
            .as_ref()
            .is_some_and(|identity| parent_by_identity.get(identity) == Some(identity))
    };

    let mut updates = Vec::new();
    for index in 0..folders.len() {
        let Some(parent) = parent_index(index) else {
            continue;
        };
        let mut segments = vec![folders[index].folder_path.trim_start_matches('/')];
        let mut current = Some(parent);
        let mut steps = 0;
        while let Some(ancestor) = current {
            steps += 1;
            if steps > folders.len() {
                break;
            }
            if !is_mailbox_root(ancestor) {
                segments.push(folders[ancestor].folder_path.trim_start_matches('/'));
            }
            current = parent_index(ancestor);
        }
        if steps > folders.len() {
            continue;
        }
        segments.reverse();
        updates.push((
            index,
            format!("/{}", segments.join("/")),
            folders[parent].folder_key.clone(),
        ));
    }

    let path_by_key: HashMap<String, String> = updates
        .iter()
        .map(|(index, path, _)| (folders[*index].folder_key.clone(), path.clone()))
        .collect();
    for (index, path, parent_key) in updates {
        folders[index].folder_path = path;
        folders[index].parent_folder_key = Some(parent_key);
    }
    for record in inventory {
        if let Some(path) = path_by_key.get(&record.folder_key) {
            record.folder_path = path.clone();
        }
    }
}

fn safe_folder_path_segment(value: &str) -> String {
    let cleaned = value
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
        .trim()
        .to_string();
    if cleaned.is_empty() {
        "unnamed-folder".to_string()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        apply_folder_nesting, classify_folder_candidate, folder_from_nbt_candidate,
        folder_node_type, FolderInventoryRecord, FolderNodeType,
    };
    use crate::pst::mapi::{MapiValue, PR_CONTENT_COUNT, PR_DISPLAY_NAME, PR_DISPLAY_NAME_A};
    use crate::pst::nbt::NbtEntry;
    use crate::pst::primitives::{BlockId, NodeId};
    use crate::pst::property_context::{PropertyContext, PropertyValue};

    #[test]
    fn classifies_folder_like_nbt_entries_by_decoded_node_type() {
        let normal_folder = entry(0x22);
        let search_folder = entry(0x43);
        let message = entry(0x24);

        assert_eq!(
            folder_node_type(normal_folder.node_id),
            Some(FolderNodeType::NormalFolder)
        );
        assert_eq!(
            folder_node_type(search_folder.node_id),
            Some(FolderNodeType::SearchFolder)
        );
        assert_eq!(folder_node_type(message.node_id), None);
        assert_eq!(
            classify_folder_candidate(&normal_folder).unwrap().status,
            "folder_candidate_from_nbt; node_type=normal_folder"
        );
        assert!(classify_folder_candidate(&message).is_none());
    }

    #[test]
    fn emits_folder_rows_from_loaded_candidate_properties() {
        let candidate = entry(0x22);
        let properties = property_context("Inbox", 7);
        let (folder, inventory) = folder_from_nbt_candidate(
            "pst-a",
            &candidate,
            Some("root-key".to_string()),
            Some(&properties),
        );

        assert_eq!(folder.folder_name, "Inbox");
        assert_eq!(folder.folder_path, "/Inbox");
        assert_eq!(folder.item_count_total, Some(7));
        assert_eq!(folder.parent_folder_key.as_deref(), Some("root-key"));
        assert!(folder.status.contains("folder_candidate_from_nbt"));
        assert!(folder.status.contains("property_context_loaded"));
        assert_eq!(inventory.item_count_email, Some(7));
    }

    #[test]
    fn emits_folder_rows_from_ansi_display_name_properties() {
        let candidate = entry(0x22);
        let mut properties = property_context("", 1);
        properties.values.remove(&PR_DISPLAY_NAME);
        properties.values.insert(
            PR_DISPLAY_NAME_A,
            PropertyValue {
                tag: PR_DISPLAY_NAME_A,
                name: "display_name".to_string(),
                raw: b"Synthetic Mail\0".to_vec(),
                decoded: Some(MapiValue::String("Synthetic Mail".to_string())),
                status: "selected".to_string(),
            },
        );

        let (folder, _) = folder_from_nbt_candidate("pst-a", &candidate, None, Some(&properties));

        assert_eq!(folder.folder_name, "Synthetic Mail");
        assert_eq!(folder.folder_path, "/Synthetic Mail");
    }

    #[test]
    fn emits_deterministic_fallback_when_candidate_properties_are_unavailable() {
        let candidate = entry(0x22);
        let (folder, inventory) = folder_from_nbt_candidate("pst-a", &candidate, None, None);

        assert_eq!(folder.folder_name, "node_22");
        assert_eq!(folder.folder_path, "/node_22");
        assert_eq!(folder.folder_node_id.as_deref(), Some("node_22"));
        assert!(folder.status.contains("property_context_unavailable"));
        assert_eq!(inventory.inventory_status, folder.status);
    }

    #[test]
    fn nests_folders_under_their_nbt_parents_and_keeps_unknown_parents_flat() {
        let record = |identity: &str, name: &str| {
            let entry =
                entry(u64::from_str_radix(identity.trim_start_matches("node_"), 16).unwrap());
            let (mut folder, inventory) =
                folder_from_nbt_candidate("pst", &entry, Some("root".to_string()), None);
            folder.folder_path = format!("/{name}");
            (
                folder,
                FolderInventoryRecord {
                    folder_path: format!("/{name}"),
                    ..inventory
                },
            )
        };
        let (mut folders, mut inventory): (Vec<_>, Vec<_>) = [
            record("node_122", "node_122"),
            record("node_8022", "Top"),
            record("node_8042", "Archive"),
            record("node_8062", "2023"),
            record("node_8082", "Orphan"),
            record("node_80a2", "LoopA"),
            record("node_80c2", "LoopB"),
        ]
        .into_iter()
        .unzip();
        let parents: HashMap<String, String> = [
            ("node_122", "node_122"),
            ("node_8022", "node_122"),
            ("node_8042", "node_8022"),
            ("node_8062", "node_8042"),
            ("node_8082", "node_9999"),
            ("node_80a2", "node_80c2"),
            ("node_80c2", "node_80a2"),
        ]
        .into_iter()
        .map(|(child, parent)| (child.to_string(), parent.to_string()))
        .collect();

        apply_folder_nesting(&mut folders, &mut inventory, &parents);

        let paths: Vec<_> = folders.iter().map(|f| f.folder_path.as_str()).collect();
        assert_eq!(
            paths,
            vec![
                "/node_122",
                "/Top",
                "/Top/Archive",
                "/Top/Archive/2023",
                "/Orphan",
                "/LoopA",
                "/LoopB"
            ]
        );
        assert_eq!(inventory[3].folder_path, "/Top/Archive/2023");
        assert_eq!(
            folders[3].parent_folder_key,
            Some(folders[2].folder_key.clone())
        );
        assert_eq!(
            folders[1].parent_folder_key,
            Some(folders[0].folder_key.clone())
        );
        assert_eq!(folders[4].parent_folder_key.as_deref(), Some("root"));
        assert_eq!(folders[5].parent_folder_key.as_deref(), Some("root"));
    }

    fn entry(node_id: u64) -> NbtEntry {
        NbtEntry {
            node_id: NodeId(node_id),
            data_block_id: BlockId(0x1000),
            subnode_block_id: None,
            parent_node_id: None,
        }
    }

    fn property_context(display_name: &str, content_count: i32) -> PropertyContext {
        let mut values = HashMap::new();
        values.insert(
            PR_DISPLAY_NAME,
            PropertyValue {
                tag: PR_DISPLAY_NAME,
                name: "display_name".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String(display_name.to_string())),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_CONTENT_COUNT,
            PropertyValue {
                tag: PR_CONTENT_COUNT,
                name: "content_count".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::Integer32(content_count)),
                status: "selected".to_string(),
            },
        );
        PropertyContext { values }
    }
}
