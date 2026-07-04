use crate::output::ids;
use crate::output::metadata::MessageRecord;
use crate::pst::mapi::{
    PR_CONVERSATION_INDEX, PR_CONVERSATION_TOPIC, PR_HASATTACH, PR_INTERNET_MESSAGE_ID,
    PR_IN_REPLY_TO_ID, PR_MESSAGE_DELIVERY_TIME, PR_SENDER_ADDRTYPE, PR_SENDER_EMAIL_ADDRESS,
    PR_SENDER_NAME, PR_SUBJECT, PR_TRANSPORT_MESSAGE_HEADERS,
};
use crate::pst::primitives::NodeId;
use crate::pst::property_context::PropertyContext;
use crate::pst::threading::{normalize_subject, threading_status};

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
    let internet_message_id = properties.string_value(PR_INTERNET_MESSAGE_ID);
    let in_reply_to_id = properties.string_value(PR_IN_REPLY_TO_ID);
    let conversation_index = properties.string_value(PR_CONVERSATION_INDEX);
    let conversation_topic = properties.string_value(PR_CONVERSATION_TOPIC);
    let has_attachments = properties
        .string_value(PR_HASATTACH)
        .map(|value| value == "true" || value == "1")
        .unwrap_or(false);
    let references = Vec::new();
    let threading_status = threading_status(
        internet_message_id.as_deref(),
        in_reply_to_id.as_deref(),
        &references,
        conversation_index.as_deref(),
    );

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
        sender_raw_address: properties.string_value(PR_SENDER_EMAIL_ADDRESS),
        sender_address_type: properties.string_value(PR_SENDER_ADDRTYPE),
        sent_at: None,
        received_at: properties.string_value(PR_MESSAGE_DELIVERY_TIME),
        created_at: None,
        modified_at: None,
        transport_message_headers: properties.string_value(PR_TRANSPORT_MESSAGE_HEADERS),
        internet_message_id,
        in_reply_to_id,
        conversation_index,
        conversation_topic,
        normalized_subject: subject.map(|value| normalize_subject(&value)),
        has_text_body: false,
        has_html_body: false,
        has_attachments,
        attachment_count: 0,
        metadata_status: "partial".to_string(),
        threading_status,
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
        transport_message_headers: None,
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
        threading_status: "threading_metadata_absent".to_string(),
        body_status: "deferred_to_m5".to_string(),
        attachment_status: "deferred_to_m5".to_string(),
        extraction_status: "metadata_only_status".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{message_from_properties, status_row};
    use crate::pst::mapi::{MapiValue, PR_SUBJECT, PR_TRANSPORT_MESSAGE_HEADERS};
    use crate::pst::primitives::NodeId;
    use crate::pst::property_context::{PropertyContext, PropertyValue};

    #[test]
    fn surfaces_transport_message_headers_when_present() {
        let mut values = HashMap::new();
        values.insert(
            PR_SUBJECT,
            PropertyValue {
                tag: PR_SUBJECT,
                name: "subject".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String("Re: Hello".to_string())),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_TRANSPORT_MESSAGE_HEADERS,
            PropertyValue {
                tag: PR_TRANSPORT_MESSAGE_HEADERS,
                name: "transport_message_headers".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String(
                    "Message-ID: <abc@example.com>\r\nFrom: sender@example.com".to_string(),
                )),
                status: "selected".to_string(),
            },
        );
        let properties = PropertyContext { values };

        let message = message_from_properties(
            "run_123",
            "pst_123",
            "folder_123",
            "/Inbox",
            NodeId(42),
            &properties,
        );

        assert_eq!(
            message.transport_message_headers.as_deref(),
            Some("Message-ID: <abc@example.com>\r\nFrom: sender@example.com")
        );
        assert_eq!(message.normalized_subject.as_deref(), Some("hello"));
    }

    #[test]
    fn leaves_transport_headers_absent_for_status_rows() {
        let message = status_row(
            "run_123",
            "pst_123",
            "folder_123",
            "/Inbox",
            "metadata_root_only",
        );

        assert_eq!(message.transport_message_headers, None);
        assert_eq!(message.item_type, "metadata_status");
    }
}
