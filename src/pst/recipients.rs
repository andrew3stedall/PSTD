use crate::output::ids;
use crate::output::metadata::RecipientRecord;
use crate::pst::mapi::{
    decode_value, MapiValue, MapiValueType, PR_DISPLAY_NAME, PR_RECIPIENT_ADDRTYPE,
    PR_RECIPIENT_DISPLAY_NAME, PR_RECIPIENT_EMAIL_ADDRESS, PR_RECIPIENT_TYPE, PR_SMTP_ADDRESS,
};
use crate::pst::table_context::{TableContext, TableRow};

pub fn recipients_from_table(message_key: &str, table: &TableContext) -> Vec<RecipientRecord> {
    table
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| recipient_from_row(message_key, row, index))
        .collect()
}

pub fn recipient_from_row(message_key: &str, row: &TableRow, ordinal: usize) -> RecipientRecord {
    let recipient_type_raw = integer_value(row, PR_RECIPIENT_TYPE);
    let recipient_type = recipient_type_label(recipient_type_raw);
    let display_name =
        string_value(row, PR_RECIPIENT_DISPLAY_NAME).or_else(|| string_value(row, PR_DISPLAY_NAME));
    let raw_address = string_value(row, PR_RECIPIENT_EMAIL_ADDRESS);
    let address_type = string_value(row, PR_RECIPIENT_ADDRTYPE);
    let smtp_address =
        string_value(row, PR_SMTP_ADDRESS).or_else(|| smtp_from_raw(&raw_address, &address_type));
    let resolution_status = resolution_status(&raw_address, &address_type, &smtp_address);

    RecipientRecord {
        message_key: message_key.to_string(),
        recipient_key: ids::recipient_key(message_key, &recipient_type, ordinal),
        recipient_type,
        display_name,
        raw_address,
        address_type,
        smtp_address,
        resolution_status,
        ordinal: ordinal as u64,
    }
}

pub fn recipient_type_label(value: Option<i32>) -> String {
    match value {
        Some(1) => "to".to_string(),
        Some(2) => "cc".to_string(),
        Some(3) => "bcc".to_string(),
        Some(4) => "reply_to".to_string(),
        Some(other) => format!("unknown_{other}"),
        None => "unknown".to_string(),
    }
}

fn row_value(row: &TableRow, tag: u32) -> Option<&[u8]> {
    row.values
        .iter()
        .find(|(candidate, _)| *candidate == tag)
        .map(|(_, value)| value.as_slice())
}

fn string_value(row: &TableRow, tag: u32) -> Option<String> {
    row_value(row, tag)
        .and_then(|raw| decode_value(MapiValueType::String, raw).ok())
        .and_then(|value| match value {
            MapiValue::String(value) => clean_string(value),
            _ => None,
        })
}

fn integer_value(row: &TableRow, tag: u32) -> Option<i32> {
    row_value(row, tag)
        .and_then(|raw| decode_value(MapiValueType::Integer32, raw).ok())
        .and_then(|value| match value {
            MapiValue::Integer32(value) => Some(value),
            _ => None,
        })
}

fn clean_string(value: String) -> Option<String> {
    let cleaned = value.trim_matches('\0').trim().to_string();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn smtp_from_raw(raw_address: &Option<String>, address_type: &Option<String>) -> Option<String> {
    let raw_address = raw_address.as_ref()?;
    let address_type = address_type.as_deref().unwrap_or_default();
    if address_type.eq_ignore_ascii_case("SMTP") || raw_address.contains('@') {
        Some(raw_address.clone())
    } else {
        None
    }
}

fn resolution_status(
    raw_address: &Option<String>,
    address_type: &Option<String>,
    smtp_address: &Option<String>,
) -> String {
    if smtp_address.is_some() {
        "smtp_available".to_string()
    } else if raw_address.is_some() && address_type.is_some() {
        "raw_address_preserved".to_string()
    } else if raw_address.is_some() {
        "raw_address_without_type".to_string()
    } else {
        "address_unavailable".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{recipient_type_label, recipients_from_table};
    use crate::pst::mapi::{
        PR_RECIPIENT_ADDRTYPE, PR_RECIPIENT_DISPLAY_NAME, PR_RECIPIENT_EMAIL_ADDRESS,
        PR_RECIPIENT_TYPE, PR_SMTP_ADDRESS,
    };
    use crate::pst::table_context::{TableContext, TableRow};

    #[test]
    fn maps_recipient_type_values() {
        assert_eq!(recipient_type_label(Some(1)), "to");
        assert_eq!(recipient_type_label(Some(2)), "cc");
        assert_eq!(recipient_type_label(Some(3)), "bcc");
        assert_eq!(recipient_type_label(Some(4)), "reply_to");
        assert_eq!(recipient_type_label(Some(99)), "unknown_99");
        assert_eq!(recipient_type_label(None), "unknown");
    }

    #[test]
    fn converts_recipient_rows_to_records() {
        let table = TableContext {
            columns: Vec::new(),
            rows: vec![TableRow {
                row_id: 0,
                values: vec![
                    (PR_RECIPIENT_TYPE, 1i32.to_le_bytes().to_vec()),
                    (PR_RECIPIENT_DISPLAY_NAME, utf16le("Alice Example")),
                    (PR_RECIPIENT_EMAIL_ADDRESS, utf16le("alice@example.com")),
                    (PR_RECIPIENT_ADDRTYPE, utf16le("SMTP")),
                ],
            }],
        };

        let recipients = recipients_from_table("msg_123", &table);
        assert_eq!(recipients.len(), 1);
        assert_eq!(recipients[0].recipient_type, "to");
        assert_eq!(recipients[0].display_name.as_deref(), Some("Alice Example"));
        assert_eq!(
            recipients[0].raw_address.as_deref(),
            Some("alice@example.com")
        );
        assert_eq!(
            recipients[0].smtp_address.as_deref(),
            Some("alice@example.com")
        );
        assert_eq!(recipients[0].resolution_status, "smtp_available");
        assert_eq!(recipients[0].ordinal, 0);
    }

    #[test]
    fn preserves_exchange_style_raw_addresses() {
        let table = TableContext {
            columns: Vec::new(),
            rows: vec![TableRow {
                row_id: 0,
                values: vec![
                    (PR_RECIPIENT_TYPE, 2i32.to_le_bytes().to_vec()),
                    (
                        PR_RECIPIENT_EMAIL_ADDRESS,
                        utf16le("/O=EXAMPLE/OU=ORG/CN=RECIPIENTS/CN=ALICE"),
                    ),
                    (PR_RECIPIENT_ADDRTYPE, utf16le("EX")),
                ],
            }],
        };

        let recipients = recipients_from_table("msg_123", &table);
        assert_eq!(recipients[0].recipient_type, "cc");
        assert_eq!(
            recipients[0].raw_address.as_deref(),
            Some("/O=EXAMPLE/OU=ORG/CN=RECIPIENTS/CN=ALICE")
        );
        assert_eq!(recipients[0].smtp_address, None);
        assert_eq!(recipients[0].resolution_status, "raw_address_preserved");
    }

    #[test]
    fn prefers_explicit_smtp_address() {
        let table = TableContext {
            columns: Vec::new(),
            rows: vec![TableRow {
                row_id: 0,
                values: vec![
                    (PR_RECIPIENT_TYPE, 3i32.to_le_bytes().to_vec()),
                    (
                        PR_RECIPIENT_EMAIL_ADDRESS,
                        utf16le("/O=EXAMPLE/OU=ORG/CN=RECIPIENTS/CN=BOB"),
                    ),
                    (PR_RECIPIENT_ADDRTYPE, utf16le("EX")),
                    (PR_SMTP_ADDRESS, utf16le("bob@example.com")),
                ],
            }],
        };

        let recipients = recipients_from_table("msg_123", &table);
        assert_eq!(recipients[0].recipient_type, "bcc");
        assert_eq!(
            recipients[0].smtp_address.as_deref(),
            Some("bob@example.com")
        );
        assert_eq!(recipients[0].resolution_status, "smtp_available");
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
