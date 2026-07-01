use std::collections::HashMap;

use crate::pst::attachments::{attachment_payload_from_properties, AttachmentPayload};
use crate::pst::mapi::{decode_value, property_def};
use crate::pst::property_context::{PropertyContext, PropertyValue};
use crate::pst::table_context::{TableContext, TableRow};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttachmentTableWiringReport {
    pub row_count: usize,
    pub payload_count: usize,
    pub missing_payload_count: usize,
    pub status: String,
}

pub fn attachment_payloads_from_table(
    message_key: &str,
    table: &TableContext,
) -> (Vec<AttachmentPayload>, AttachmentTableWiringReport) {
    let mut payloads = Vec::new();
    let mut missing_payload_count = 0usize;

    for (ordinal, row) in table.rows.iter().enumerate() {
        let properties = property_context_from_table_row(row);
        if let Some(payload) = attachment_payload_from_properties(message_key, ordinal, &properties)
        {
            payloads.push(payload);
        } else {
            missing_payload_count += 1;
        }
    }

    let status = if missing_payload_count == 0 {
        "attachment_table_payloads_wired"
    } else if payloads.is_empty() {
        "attachment_table_payloads_unavailable"
    } else {
        "attachment_table_payloads_partially_wired"
    };

    let report = AttachmentTableWiringReport {
        row_count: table.rows.len(),
        payload_count: payloads.len(),
        missing_payload_count,
        status: status.to_string(),
    };

    (payloads, report)
}

pub fn property_context_from_table_row(row: &TableRow) -> PropertyContext {
    let mut values = HashMap::new();

    for (tag, raw) in &row.values {
        let (name, decoded, status) = if let Some(def) = property_def(*tag) {
            (
                def.name.to_string(),
                decode_value(def.value_type, raw).ok(),
                "selected".to_string(),
            )
        } else {
            (
                format!("unknown_0x{tag:08x}"),
                None,
                "not_selected".to_string(),
            )
        };

        values.insert(
            *tag,
            PropertyValue {
                tag: *tag,
                name,
                raw: raw.clone(),
                decoded,
                status,
            },
        );
    }

    PropertyContext { values }
}

#[cfg(test)]
mod tests {
    use super::attachment_payloads_from_table;
    use crate::pst::mapi::{PR_ATTACH_DATA_BIN, PR_ATTACH_LONG_FILENAME, PR_ATTACH_MIME_TAG};
    use crate::pst::table_context::{TableContext, TableRow};

    #[test]
    fn wires_attachment_payloads_from_table_rows() {
        let table = TableContext {
            columns: Vec::new(),
            rows: vec![TableRow {
                row_id: 0,
                values: vec![
                    (PR_ATTACH_DATA_BIN, b"attachment bytes".to_vec()),
                    (PR_ATTACH_LONG_FILENAME, utf16le("report.pdf")),
                    (PR_ATTACH_MIME_TAG, utf16le("application/pdf")),
                ],
            }],
        };

        let (payloads, report) = attachment_payloads_from_table("msg_123", &table);
        assert_eq!(payloads.len(), 1);
        assert_eq!(payloads[0].record.filename_safe, "report.pdf");
        assert_eq!(
            payloads[0].record.content_type.as_deref(),
            Some("application/pdf")
        );
        assert_eq!(payloads[0].bytes, b"attachment bytes");
        assert_eq!(report.row_count, 1);
        assert_eq!(report.payload_count, 1);
        assert_eq!(report.missing_payload_count, 0);
        assert_eq!(report.status, "attachment_table_payloads_wired");
    }

    #[test]
    fn reports_missing_attachment_payloads() {
        let table = TableContext {
            columns: Vec::new(),
            rows: vec![TableRow {
                row_id: 0,
                values: vec![(PR_ATTACH_LONG_FILENAME, utf16le("missing.pdf"))],
            }],
        };

        let (payloads, report) = attachment_payloads_from_table("msg_123", &table);
        assert!(payloads.is_empty());
        assert_eq!(report.missing_payload_count, 1);
        assert_eq!(report.status, "attachment_table_payloads_unavailable");
    }

    fn utf16le(value: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        for unit in value.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes
    }
}
