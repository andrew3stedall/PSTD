use crate::output::ids;
use crate::output::metadata::MessageRecord;
use crate::pst::mapi::{
    PR_HASATTACH, PR_MESSAGE_DELIVERY_TIME, PR_SENDER_EMAIL_ADDRESS, PR_SENDER_NAME, PR_SUBJECT,
};
use crate::pst::primitives::NodeId;
use crate::pst::property_context::PropertyContext;

pub fn message_from_properties(
    run_id: &str,
    pst_id: &str,
    folder_key: &str,
    folder_path: &str,
    node_id: NodeId,
    properties: &PropertyContext,
) -> MessageRecord {
    let message_identity = format!("node_{:x}", node_id.0);
    let subject = properties.string_value(PR_SUBJECT);
    let has_attachments = properties
        .string_value(PR_HASATTACH)
        .map(|value| value == "true" || value == "1")
        .unwrap_or(false);

    MessageRecord {
        run_id: run_id.to_string(),
        pst_id: pst_id.to_string(),
        folder_key: folder_key.to_string(),
        message_key: ids::message_key(pst_id, &message_identity),
        message_node_id: Some(message_identity),
        folder_path: folder_path.to_string(),
        item_type: "message_metadata".to_string(),
        subject: subject.clone(),
        sender_name: properties.string_value(PR_SENDER_NAME),
        sender_email: properties.string_value(PR_SENDER_EMAIL_ADDRESS),
        sender_raw_address: None,
        sender_address_type: None,
        sent_at: None,
        received_at: properties.string_value(PR_MESSAGE_DELIVERY_TIME),
        created_at: None,
        modified_at: None,
        internet_message_id: None,
        in_reply_to_id: None,
        conversation_index: None,
        conversation_topic: None,
        normalized_subject: subject.map(|value| value.trim().to_lowercase()),
        has_text_body: false,
        has_html_body: false,
        has_attachments,
        attachment_count: 0,
        metadata_status: "partial".to_string(),
        threading_status: "deferred_to_m4".to_string(),
        body_status: "deferred_to_m5".to_string(),
        attachment_status: "deferred_to_m5".to_string(),
        extraction_status: "metadata_only".to_string(),
    }
}

pub fn status_row(
    run_id: &str,
    pst_id: &str,
    folder_key: &str,
    folder_path: &str,
    status: &str,
) -> MessageRecord {
    MessageRecord {
        run_id: run_id.to_string(),
        pst_id: pst_id.to_string(),
        folder_key: folder_key.to_string(),
        message_key: ids::message_key(pst_id, status),
        message_node_id: None,
        folder_path: folder_path.to_string(),
        item_type: "metadata_status".to_string(),
        subject: Some("PSTD metadata extraction status".to_string()),
        sender_name: None,
        sender_email: None,
        sender_raw_address: None,
        sender_address_type: None,
        sent_at: None,
        received_at: None,
        created_at: None,
        modified_at: None,
        internet_message_id: None,
        in_reply_to_id: None,
        conversation_index: None,
        conversation_topic: None,
        normalized_subject: Some("pstd metadata extraction status".to_string()),
        has_text_body: false,
        has_html_body: false,
        has_attachments: false,
        attachment_count: 0,
        metadata_status: status.to_string(),
        threading_status: "deferred_to_m4".to_string(),
        body_status: "deferred_to_m5".to_string(),
        attachment_status: "deferred_to_m5".to_string(),
        extraction_status: "metadata_only_status".to_string(),
    }
}
