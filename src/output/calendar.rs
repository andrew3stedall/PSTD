use crate::output::ids;
use crate::output::metadata::{CalendarRecord, MessageRecord};

pub fn build_calendar_records(messages: &[MessageRecord]) -> Vec<CalendarRecord> {
    let mut records = messages
        .iter()
        .filter(|message| is_appointment(message.message_class.as_deref()))
        .map(|message| {
            let source_backed_count = message.subject.is_some()
                || message.sender_name.is_some()
                || message.sender_email.is_some();
            let status = if source_backed_count {
                "calendar_partial_source_fields"
            } else {
                "calendar_fields_unavailable"
            };
            CalendarRecord {
                calendar_key: ids::stable_id("calendar", &[&message.message_key]),
                message_key: message.message_key.clone(),
                folder_path: message.folder_path.clone(),
                source_node_id: message.message_node_id.clone(),
                message_class: message.message_class.clone(),
                uid: ids::stable_id("calendar-uid", &[&message.message_key]),
                summary: message.subject.clone(),
                description: None,
                location: None,
                organizer_name: message.sender_name.clone(),
                organizer_email: message.sender_email.clone(),
                dtstart: None,
                dtend: None,
                timezone: None,
                all_day: None,
                recurrence_rule: None,
                recurrence_raw: None,
                recurrence_status: "recurrence_unavailable_source_properties_not_decoded"
                    .to_string(),
                exception_status: "exceptions_unavailable_source_properties_not_decoded"
                    .to_string(),
                alarm_status: "alarm_unavailable_source_properties_not_decoded".to_string(),
                categories: Vec::new(),
                status: status.to_string(),
                authoritative: false,
                synthetic: false,
                raw_evidence_refs: vec![
                    format!("message_record:{}", message.message_key),
                    format!(
                        "message_class:{}",
                        message.message_class.as_deref().unwrap_or("<missing>")
                    ),
                ],
            }
        })
        .collect::<Vec<_>>();
    records.sort_by_key(|record| record.calendar_key.clone());
    records
}

pub fn serialize_icalendar(records: &[CalendarRecord]) -> String {
    let mut output = String::from(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//PSTD//readpst parity//EN\r\nCALSCALE:GREGORIAN\r\n",
    );
    for record in records {
        output.push_str("BEGIN:VEVENT\r\n");
        output.push_str(&format!("UID:{}\r\n", escape_ical(&record.uid)));
        output.push_str("DTSTAMP:19700101T000000Z\r\n");
        output.push_str("X-PSTD-DTSTAMP-SYNTHETIC:TRUE\r\n");
        if let Some(summary) = record.summary.as_deref() {
            output.push_str(&format!("SUMMARY:{}\r\n", escape_ical(summary)));
        }
        if let Some(name) = record.organizer_name.as_deref() {
            if let Some(email) = record.organizer_email.as_deref() {
                output.push_str(&format!(
                    "ORGANIZER;CN={}:mailto:{}\r\n",
                    escape_ical_parameter(name),
                    escape_ical(email)
                ));
            }
        }
        if let Some(start) = record.dtstart.as_deref() {
            output.push_str(&format!("DTSTART:{}\r\n", escape_ical(start)));
        }
        if let Some(end) = record.dtend.as_deref() {
            output.push_str(&format!("DTEND:{}\r\n", escape_ical(end)));
        }
        if let Some(timezone) = record.timezone.as_deref() {
            output.push_str(&format!("X-WR-TIMEZONE:{}\r\n", escape_ical(timezone)));
        }
        if let Some(rule) = record.recurrence_rule.as_deref() {
            output.push_str(&format!("RRULE:{}\r\n", escape_ical(rule)));
        }
        for category in &record.categories {
            output.push_str(&format!("CATEGORIES:{}\r\n", escape_ical(category)));
        }
        output.push_str(&format!(
            "X-PSTD-MESSAGE-CLASS:{}\r\nX-PSTD-STATUS:{}\r\nX-PSTD-AUTHORITATIVE:{}\r\nX-PSTD-SYNTHETIC:{}\r\nX-PSTD-RECURRENCE-STATUS:{}\r\nX-PSTD-EXCEPTION-STATUS:{}\r\nX-PSTD-ALARM-STATUS:{}\r\n",
            escape_ical(record.message_class.as_deref().unwrap_or("<missing>")),
            escape_ical(&record.status),
            record.authoritative,
            record.synthetic,
            escape_ical(&record.recurrence_status),
            escape_ical(&record.exception_status),
            escape_ical(&record.alarm_status),
        ));
        output.push_str("END:VEVENT\r\n");
    }
    output.push_str("END:VCALENDAR\r\n");
    output
}

fn is_appointment(message_class: Option<&str>) -> bool {
    message_class
        .map(str::trim)
        .map(|value| value.to_ascii_lowercase())
        .is_some_and(|value| value.starts_with("ipm.appointment"))
}

fn escape_ical(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\r', "")
        .replace('\n', "\\n")
}

fn escape_ical_parameter(value: &str) -> String {
    escape_ical(value).replace(':', "\\:")
}

#[cfg(test)]
mod tests {
    use super::{build_calendar_records, serialize_icalendar};
    use crate::output::metadata::MessageRecord;

    fn appointment() -> MessageRecord {
        MessageRecord {
            run_id: "run".to_string(),
            pst_id: "pst".to_string(),
            folder_key: "folder".to_string(),
            message_key: "msg-appointment".to_string(),
            message_node_id: Some("node-appointment".to_string()),
            folder_path: "/Calendar".to_string(),
            item_type: "message_metadata".to_string(),
            message_class: Some("IPM.Appointment".to_string()),
            subject: Some("Planning, sync".to_string()),
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
            metadata_status: "appointment_source".to_string(),
            threading_status: "not_applicable".to_string(),
            body_status: "not_applicable".to_string(),
            attachment_status: "not_applicable".to_string(),
            extraction_status: "appointment_source".to_string(),
        }
    }

    #[test]
    fn emits_deterministic_partial_icalendar_without_guessing_dates() {
        let records = build_calendar_records(&[appointment()]);
        assert_eq!(records.len(), 1);
        let ics = serialize_icalendar(&records);
        assert_eq!(ics.matches("BEGIN:VEVENT").count(), 1);
        assert!(ics.contains("SUMMARY:Planning\\, sync\r\n"));
        assert!(ics.contains("X-PSTD-RECURRENCE-STATUS:recurrence_unavailable"));
        assert!(ics.contains("DTSTAMP:19700101T000000Z\r\n"));
        assert!(!ics.contains("DTSTART:"));
        assert_eq!(ics, serialize_icalendar(&records));
    }

    #[test]
    fn rejects_non_appointment_classes() {
        let mut message = appointment();
        message.message_class = Some("IPM.Note".to_string());
        assert!(build_calendar_records(&[message]).is_empty());
    }
}
