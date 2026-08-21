use crate::output::ids;
use crate::output::metadata::{ContactRecord, MessageRecord};

pub fn build_contact_records(messages: &[MessageRecord]) -> Vec<ContactRecord> {
    let mut records = messages
        .iter()
        .filter(|message| is_contact(message.message_class.as_deref()))
        .map(|message| {
            let full_name = message.sender_name.clone();
            let email = message.sender_email.clone();
            let authoritative = full_name.is_some() || email.is_some();
            let status = if authoritative {
                "contact_partial_source_fields"
            } else {
                "contact_fields_unavailable"
            };
            ContactRecord {
                contact_key: ids::stable_id("contact", &[&message.message_key]),
                message_key: message.message_key.clone(),
                folder_path: message.folder_path.clone(),
                source_node_id: message.message_node_id.clone(),
                full_name,
                email,
                address_type: message.sender_address_type.clone(),
                notes: None,
                categories: Vec::new(),
                status: status.to_string(),
                authoritative,
                synthetic: false,
                raw_evidence_refs: vec![format!("message_record:{}", message.message_key)],
            }
        })
        .collect::<Vec<_>>();
    records.sort_by_key(|record| record.contact_key.clone());
    records
}

pub fn serialize_vcards(records: &[ContactRecord]) -> String {
    let mut output = String::new();
    for record in records {
        output.push_str("BEGIN:VCARD\r\nVERSION:3.0\r\n");
        output.push_str(&format!("UID:{}\r\n", record.contact_key));
        if let Some(name) = record.full_name.as_deref() {
            output.push_str(&format!("FN:{}\r\n", escape_vcard(name)));
        }
        if let Some(email) = record.email.as_deref() {
            let address_type = record.address_type.as_deref().unwrap_or("unknown");
            output.push_str(&format!(
                "EMAIL;TYPE={}:{}\r\n",
                escape_vcard(address_type),
                escape_vcard(email)
            ));
        }
        output.push_str(&format!(
            "X-PSTD-STATUS:{}\r\nX-PSTD-AUTHORITATIVE:{}\r\n",
            escape_vcard(&record.status),
            record.authoritative
        ));
        for category in &record.categories {
            output.push_str(&format!("CATEGORIES:{}\r\n", escape_vcard(category)));
        }
        if let Some(notes) = record.notes.as_deref() {
            output.push_str(&format!("NOTE:{}\r\n", escape_vcard(notes)));
        }
        output.push_str("END:VCARD\r\n");
    }
    output
}

pub fn serialize_contact_list(records: &[ContactRecord]) -> String {
    records
        .iter()
        .map(|record| {
            let name = record.full_name.as_deref().unwrap_or("<unnamed>");
            let email = record.email.as_deref().unwrap_or("<address-unavailable>");
            format!("{} <{}>\n", name, email)
        })
        .collect()
}

fn is_contact(message_class: Option<&str>) -> bool {
    message_class
        .map(str::trim)
        .map(|value| value.to_ascii_lowercase())
        .is_some_and(|value| {
            value.starts_with("ipm.contact")
                || value.contains("distlist")
                || value.contains("distribution")
        })
}

fn escape_vcard(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\r', "")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::{build_contact_records, serialize_contact_list, serialize_vcards};
    use crate::output::metadata::MessageRecord;

    fn contact() -> MessageRecord {
        MessageRecord {
            run_id: "run".to_string(),
            pst_id: "pst".to_string(),
            folder_key: "folder".to_string(),
            message_key: "msg-contact".to_string(),
            message_node_id: Some("node-1".to_string()),
            folder_path: "/Contacts".to_string(),
            item_type: "message_metadata".to_string(),
            message_class: Some("IPM.Contact".to_string()),
            subject: None,
            sender_name: Some("Doe, Jane".to_string()),
            sender_email: Some("jane@example.test".to_string()),
            sender_raw_address: Some("jane@example.test".to_string()),
            sender_address_type: Some("SMTP".to_string()),
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
            normalized_subject: None,
            has_text_body: false,
            has_html_body: false,
            has_attachments: false,
            attachment_count: 0,
            metadata_status: "synthetic_contact_source".to_string(),
            threading_status: "not_applicable".to_string(),
            body_status: "not_applicable".to_string(),
            attachment_status: "not_applicable".to_string(),
            extraction_status: "contact_source".to_string(),
        }
    }

    #[test]
    fn emits_stable_vcard_and_contact_list_from_source_fields() {
        let records = build_contact_records(&[contact()]);
        assert_eq!(records.len(), 1);
        let vcard = serialize_vcards(&records);
        assert!(vcard.contains("VERSION:3.0\r\n"));
        assert!(vcard.contains("FN:Doe\\, Jane\r\n"));
        assert!(vcard.contains("EMAIL;TYPE=SMTP:jane@example.test\r\n"));
        assert_eq!(serialize_contact_list(&records), "Doe, Jane <jane@example.test>\n");
    }

    #[test]
    fn does_not_promote_non_contacts() {
        let mut message = contact();
        message.message_class = Some("IPM.Note".to_string());
        assert!(build_contact_records(&[message]).is_empty());
    }
}
