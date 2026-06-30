use crate::output::ids;
use crate::output::metadata::FolderRecord;
use crate::pst::header::PstHeader;
use crate::pst::mapi::{PR_CONTENT_COUNT, PR_DISPLAY_NAME};
use crate::pst::property_context::PropertyContext;

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

pub fn folder_from_properties(
    pst_id: &str,
    folder_identity: &str,
    parent_folder_key: Option<String>,
    properties: &PropertyContext,
) -> (FolderRecord, FolderInventoryRecord) {
    let folder_key = ids::folder_key(pst_id, folder_identity);
    let folder_name = properties
        .string_value(PR_DISPLAY_NAME)
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
