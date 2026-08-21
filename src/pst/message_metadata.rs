use crate::output::ids;
use crate::output::metadata::MessageRecord;
use crate::pst::mapi::{
    PR_CLIENT_SUBMIT_TIME, PR_CONVERSATION_INDEX, PR_CONVERSATION_TOPIC, PR_CONVERSATION_TOPIC_A,
    PR_CREATION_TIME, PR_DELETE_AFTER_SUBMIT, PR_HASATTACH, PR_IMPORTANCE, PR_INTERNET_MESSAGE_ID,
    PR_INTERNET_MESSAGE_ID_A, PR_IN_REPLY_TO_ID, PR_IN_REPLY_TO_ID_A, PR_LAST_MODIFICATION_TIME,
    PR_MESSAGE_CLASS, PR_MESSAGE_CLASS_A, PR_MESSAGE_DELIVERY_TIME, PR_MESSAGE_FLAGS,
    PR_ORIGINATOR_DELIVERY_REPORT_REQUESTED, PR_PRIORITY, PR_READ_RECEIPT_REQUESTED,
    PR_RECEIVED_BY_ADDRTYPE, PR_RECEIVED_BY_ADDRTYPE_A, PR_RECEIVED_BY_EMAIL_ADDRESS,
    PR_RECEIVED_BY_EMAIL_ADDRESS_A, PR_RECEIVED_REPRESENTING_ADDRTYPE,
    PR_RECEIVED_REPRESENTING_ADDRTYPE_A, PR_RECEIVED_REPRESENTING_EMAIL_ADDRESS,
    PR_RECEIVED_REPRESENTING_EMAIL_ADDRESS_A, PR_REPLY_REQUESTED, PR_SENDER_ADDRTYPE,
    PR_SENDER_ADDRTYPE_A, PR_SENDER_EMAIL_ADDRESS, PR_SENDER_EMAIL_ADDRESS_A, PR_SENDER_NAME,
    PR_SENDER_NAME_A, PR_SENSITIVITY, PR_SENT_REPRESENTING_ADDRTYPE,
    PR_SENT_REPRESENTING_ADDRTYPE_A, PR_SENT_REPRESENTING_EMAIL_ADDRESS,
    PR_SENT_REPRESENTING_EMAIL_ADDRESS_A, PR_SUBJECT, PR_SUBJECT_A, PR_TRANSPORT_MESSAGE_HEADERS,
    PR_TRANSPORT_MESSAGE_HEADERS_A,
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
    let subject = properties.first_string_value(&[PR_SUBJECT, PR_SUBJECT_A]);
    let message_class = properties.first_string_value(&[PR_MESSAGE_CLASS, PR_MESSAGE_CLASS_A]);
    let internet_message_id =
        properties.first_string_value(&[PR_INTERNET_MESSAGE_ID, PR_INTERNET_MESSAGE_ID_A]);
    let in_reply_to_id = properties.first_string_value(&[PR_IN_REPLY_TO_ID, PR_IN_REPLY_TO_ID_A]);
    let conversation_index = properties.string_value(PR_CONVERSATION_INDEX);
    let conversation_topic =
        properties.first_string_value(&[PR_CONVERSATION_TOPIC, PR_CONVERSATION_TOPIC_A]);
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

    let sender_email =
        properties.first_string_value(&[PR_SENDER_EMAIL_ADDRESS, PR_SENDER_EMAIL_ADDRESS_A]);
    let report_controls = [
        ("read_receipt", PR_READ_RECEIPT_REQUESTED),
        ("reply_requested", PR_REPLY_REQUESTED),
        ("delivery_report", PR_ORIGINATOR_DELIVERY_REPORT_REQUESTED),
        ("delete_after_submit", PR_DELETE_AFTER_SUBMIT),
    ]
    .into_iter()
    .filter_map(|(name, tag)| {
        properties
            .string_value(tag)
            .map(|value| format!("{name}={value}"))
    })
    .collect::<Vec<_>>();

    MessageRecord {
        run_id: run_id.to_string(),
        pst_id: pst_id.to_string(),
        folder_key: folder_key.to_string(),
        message_key: ids::message_key(pst_id, &message_identity),
        message_node_id: Some(message_identity),
        folder_path: folder_path.to_string(),
        item_type: "message_metadata".to_string(),
        message_class,
        subject: subject.clone(),
        sender_name: properties.first_string_value(&[PR_SENDER_NAME, PR_SENDER_NAME_A]),
        sender_email: sender_email.clone(),
        sender_raw_address: sender_email,
        sender_address_type: properties
            .first_string_value(&[PR_SENDER_ADDRTYPE, PR_SENDER_ADDRTYPE_A]),
        sent_representing_email: properties.first_string_value(&[
            PR_SENT_REPRESENTING_EMAIL_ADDRESS,
            PR_SENT_REPRESENTING_EMAIL_ADDRESS_A,
        ]),
        sent_representing_address_type: properties.first_string_value(&[
            PR_SENT_REPRESENTING_ADDRTYPE,
            PR_SENT_REPRESENTING_ADDRTYPE_A,
        ]),
        received_by_email: properties
            .first_string_value(&[PR_RECEIVED_BY_EMAIL_ADDRESS, PR_RECEIVED_BY_EMAIL_ADDRESS_A]),
        received_by_address_type: properties
            .first_string_value(&[PR_RECEIVED_BY_ADDRTYPE, PR_RECEIVED_BY_ADDRTYPE_A]),
        received_representing_email: properties.first_string_value(&[
            PR_RECEIVED_REPRESENTING_EMAIL_ADDRESS,
            PR_RECEIVED_REPRESENTING_EMAIL_ADDRESS_A,
        ]),
        received_representing_address_type: properties.first_string_value(&[
            PR_RECEIVED_REPRESENTING_ADDRTYPE,
            PR_RECEIVED_REPRESENTING_ADDRTYPE_A,
        ]),
        sent_at: properties.string_value(PR_CLIENT_SUBMIT_TIME),
        received_at: properties.string_value(PR_MESSAGE_DELIVERY_TIME),
        created_at: properties.string_value(PR_CREATION_TIME),
        modified_at: properties.string_value(PR_LAST_MODIFICATION_TIME),
        importance: properties.string_value(PR_IMPORTANCE),
        message_flags: properties.string_value(PR_MESSAGE_FLAGS),
        priority: properties.string_value(PR_PRIORITY),
        sensitivity: properties.string_value(PR_SENSITIVITY),
        read_receipt_requested: properties.string_value(PR_READ_RECEIPT_REQUESTED),
        reply_requested: properties.string_value(PR_REPLY_REQUESTED),
        delivery_report_requested: properties.string_value(PR_ORIGINATOR_DELIVERY_REPORT_REQUESTED),
        delete_after_submit: properties.string_value(PR_DELETE_AFTER_SUBMIT),
        transport_message_headers: properties
            .first_string_value(&[PR_TRANSPORT_MESSAGE_HEADERS, PR_TRANSPORT_MESSAGE_HEADERS_A]),
        internet_message_id,
        in_reply_to_id,
        conversation_index,
        conversation_topic,
        normalized_subject: subject.map(|value| normalize_subject(&value)),
        has_text_body: false,
        has_html_body: false,
        has_attachments,
        attachment_count: 0,
        metadata_status: if report_controls.is_empty() {
            "metadata_projected_without_report_controls".to_string()
        } else {
            format!(
                "metadata_projected; report_controls={}",
                report_controls.join(",")
            )
        },
        threading_status,
        body_status: "deferred_to_m5".to_string(),
        attachment_status: "deferred_to_m5".to_string(),
        extraction_status: format!(
            "metadata_only; {}; {}",
            properties.pq9_status(),
            properties.pq10_status()
        ),
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
        message_class: None,
        subject: Some("PSTD metadata extraction status".to_string()),
        sender_name: None,
        sender_email: None,
        sender_raw_address: None,
        sender_address_type: None,
        sent_representing_email: None,
        sent_representing_address_type: None,
        received_by_email: None,
        received_by_address_type: None,
        received_representing_email: None,
        received_representing_address_type: None,
        sent_at: None,
        received_at: None,
        created_at: None,
        modified_at: None,
        importance: None,
        message_flags: None,
        priority: None,
        sensitivity: None,
        read_receipt_requested: None,
        reply_requested: None,
        delivery_report_requested: None,
        delete_after_submit: None,
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
    use crate::pst::mapi::{
        MapiValue, PR_CLIENT_SUBMIT_TIME, PR_IMPORTANCE, PR_MESSAGE_FLAGS, PR_PRIORITY,
        PR_READ_RECEIPT_REQUESTED, PR_RECEIVED_BY_EMAIL_ADDRESS, PR_SENSITIVITY, PR_SUBJECT,
        PR_SUBJECT_A, PR_TRANSPORT_MESSAGE_HEADERS, PR_TRANSPORT_MESSAGE_HEADERS_A,
    };
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
        let properties = PropertyContext::from_values(values)
            .with_pq10_traversal_status("heap_bth_property_context");

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
        assert!(message
            .extraction_status
            .contains("pq9_tag_shape=plausible:2"));
        assert!(message
            .extraction_status
            .contains("pq10_traversal=heap_bth_property_context"));
    }

    #[test]
    fn surfaces_string8_alias_metadata_when_present() {
        let mut values = HashMap::new();
        values.insert(
            PR_SUBJECT_A,
            PropertyValue {
                tag: PR_SUBJECT_A,
                name: "subject".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String("Re: Alias".to_string())),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_TRANSPORT_MESSAGE_HEADERS_A,
            PropertyValue {
                tag: PR_TRANSPORT_MESSAGE_HEADERS_A,
                name: "transport_message_headers".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String(
                    "Message-ID: <alias@example.com>".to_string(),
                )),
                status: "selected".to_string(),
            },
        );
        let properties = PropertyContext::from_values(values);

        let message = message_from_properties(
            "run_123",
            "pst_123",
            "folder_123",
            "/Inbox",
            NodeId(42),
            &properties,
        );

        assert_eq!(message.subject.as_deref(), Some("Re: Alias"));
        assert_eq!(message.normalized_subject.as_deref(), Some("alias"));
        assert_eq!(
            message.transport_message_headers.as_deref(),
            Some("Message-ID: <alias@example.com>")
        );
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

    #[test]
    fn projects_dates_identity_controls_and_native_addresses() {
        let mut values = HashMap::new();
        values.insert(
            PR_CLIENT_SUBMIT_TIME,
            PropertyValue {
                tag: PR_CLIENT_SUBMIT_TIME,
                name: "sent_at".to_string(),
                raw: vec![1; 8],
                decoded: Some(MapiValue::FileTime("filetime:1".to_string())),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_RECEIVED_BY_EMAIL_ADDRESS,
            PropertyValue {
                tag: PR_RECEIVED_BY_EMAIL_ADDRESS,
                name: "received_by_email_address".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::String(
                    "/o=Exchange/ou=Example/cn=Recipients/cn=1".into(),
                )),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_IMPORTANCE,
            PropertyValue {
                tag: PR_IMPORTANCE,
                name: "importance".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::Integer32(2)),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_MESSAGE_FLAGS,
            PropertyValue {
                tag: PR_MESSAGE_FLAGS,
                name: "message_flags".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::Integer32(5)),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_PRIORITY,
            PropertyValue {
                tag: PR_PRIORITY,
                name: "priority".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::Integer32(2)),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_SENSITIVITY,
            PropertyValue {
                tag: PR_SENSITIVITY,
                name: "sensitivity".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::Integer32(1)),
                status: "selected".to_string(),
            },
        );
        values.insert(
            PR_READ_RECEIPT_REQUESTED,
            PropertyValue {
                tag: PR_READ_RECEIPT_REQUESTED,
                name: "read_receipt_requested".to_string(),
                raw: Vec::new(),
                decoded: Some(MapiValue::Boolean(true)),
                status: "selected".to_string(),
            },
        );

        let message = message_from_properties(
            "run_123",
            "pst_123",
            "folder_123",
            "/Inbox",
            NodeId(42),
            &PropertyContext::from_values(values),
        );

        assert_eq!(message.sent_at.as_deref(), Some("filetime:1"));
        assert_eq!(
            message.received_by_email.as_deref(),
            Some("/o=Exchange/ou=Example/cn=Recipients/cn=1")
        );
        assert_eq!(message.importance.as_deref(), Some("2"));
        assert_eq!(message.message_flags.as_deref(), Some("5"));
        assert_eq!(message.priority.as_deref(), Some("2"));
        assert_eq!(message.sensitivity.as_deref(), Some("1"));
        assert_eq!(message.read_receipt_requested.as_deref(), Some("true"));
        assert!(message.metadata_status.contains("read_receipt=true"));
    }
}
