#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageRecord {
    pub run_id: String,
    pub pst_id: String,
    pub folder_key: String,
    pub message_key: String,
    pub message_node_id: Option<String>,
    pub folder_path: String,
    pub item_type: String,
    pub subject: Option<String>,
    pub sender_name: Option<String>,
    pub sender_email: Option<String>,
    pub sender_raw_address: Option<String>,
    pub sender_address_type: Option<String>,
    pub sent_at: Option<String>,
    pub received_at: Option<String>,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
    pub transport_message_headers: Option<String>,
    pub internet_message_id: Option<String>,
    pub in_reply_to_id: Option<String>,
    pub conversation_index: Option<String>,
    pub conversation_topic: Option<String>,
    pub normalized_subject: Option<String>,
    pub has_text_body: bool,
    pub has_html_body: bool,
    pub has_attachments: bool,
    pub attachment_count: u64,
    pub metadata_status: String,
    pub threading_status: String,
    pub body_status: String,
    pub attachment_status: String,
    pub extraction_status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecipientRecord {
    pub message_key: String,
    pub recipient_key: String,
    pub recipient_type: String,
    pub display_name: Option<String>,
    pub raw_address: Option<String>,
    pub address_type: Option<String>,
    pub smtp_address: Option<String>,
    pub resolution_status: String,
    pub ordinal: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageReferenceRecord {
    pub message_key: String,
    pub reference_key: String,
    pub reference_type: String,
    pub referenced_internet_message_id: String,
    pub ordinal: u64,
    pub source: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BodyRecord {
    pub message_key: String,
    pub body_key: String,
    pub body_type: String,
    pub archive_path: String,
    pub encoding: Option<String>,
    pub size_bytes: u64,
    pub sha256: String,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttachmentRecord {
    pub message_key: String,
    pub attachment_key: String,
    pub filename_original: Option<String>,
    pub filename_safe: String,
    pub content_type: Option<String>,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub sha256: String,
    pub is_inline: bool,
    pub content_id: Option<String>,
    pub ordinal: u64,
    pub archive_path: String,
    pub extraction_status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FolderRecord {
    pub pst_id: String,
    pub folder_key: String,
    pub parent_folder_key: Option<String>,
    pub folder_path: String,
    pub folder_name: String,
    pub folder_node_id: Option<String>,
    pub item_count_total: Option<u64>,
    pub child_folder_count: Option<u64>,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ManifestRecord {
    pub run_id: String,
    pub pst_id: String,
    pub message_key: Option<String>,
    pub folder_key: Option<String>,
    pub artefact_type: String,
    pub archive_path: String,
    pub sha256: Option<String>,
    pub size_bytes: Option<u64>,
    pub status: String,
    pub issue_count: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SelectedMapiPropertyRecord {
    pub message_key: String,
    pub property_id: String,
    pub property_name: String,
    pub property_type: String,
    pub value_summary: Option<String>,
    pub source: String,
    pub status: String,
}
