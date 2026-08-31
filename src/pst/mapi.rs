use crate::error::{PstdError, PstdResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MapiValueType {
    String,
    String8,
    Integer32,
    Integer64,
    Boolean,
    FileTime,
    Binary,
    Unknown,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct MapiPropertyDef {
    pub tag: u32,
    pub name: &'static str,
    pub value_type: MapiValueType,
}

pub const PR_SUBJECT: u32 = 0x0037_001f;
pub const PR_SUBJECT_A: u32 = 0x0037_001e;
pub const PR_MESSAGE_CLASS: u32 = 0x001a_001f;
pub const PR_MESSAGE_CLASS_A: u32 = 0x001a_001e;
pub const PR_SENDER_NAME: u32 = 0x0c1a_001f;
pub const PR_SENDER_NAME_A: u32 = 0x0c1a_001e;
pub const PR_SENDER_EMAIL_ADDRESS: u32 = 0x0c1f_001f;
pub const PR_SENDER_EMAIL_ADDRESS_A: u32 = 0x0c1f_001e;
pub const PR_SENDER_ADDRTYPE: u32 = 0x0c1e_001f;
pub const PR_SENDER_ADDRTYPE_A: u32 = 0x0c1e_001e;
pub const PR_SENT_REPRESENTING_EMAIL_ADDRESS: u32 = 0x0065_001f;
pub const PR_SENT_REPRESENTING_EMAIL_ADDRESS_A: u32 = 0x0065_001e;
pub const PR_SENT_REPRESENTING_ADDRTYPE: u32 = 0x0064_001f;
pub const PR_SENT_REPRESENTING_ADDRTYPE_A: u32 = 0x0064_001e;
pub const PR_RECEIVED_BY_ADDRTYPE: u32 = 0x0075_001f;
pub const PR_RECEIVED_BY_ADDRTYPE_A: u32 = 0x0075_001e;
pub const PR_RECEIVED_BY_EMAIL_ADDRESS: u32 = 0x0076_001f;
pub const PR_RECEIVED_BY_EMAIL_ADDRESS_A: u32 = 0x0076_001e;
pub const PR_RECEIVED_REPRESENTING_ADDRTYPE: u32 = 0x0077_001f;
pub const PR_RECEIVED_REPRESENTING_ADDRTYPE_A: u32 = 0x0077_001e;
pub const PR_RECEIVED_REPRESENTING_EMAIL_ADDRESS: u32 = 0x0078_001f;
pub const PR_RECEIVED_REPRESENTING_EMAIL_ADDRESS_A: u32 = 0x0078_001e;
pub const PR_CLIENT_SUBMIT_TIME: u32 = 0x0039_0040;
pub const PR_MESSAGE_DELIVERY_TIME: u32 = 0x0e06_0040;
pub const PR_CREATION_TIME: u32 = 0x3007_0040;
pub const PR_LAST_MODIFICATION_TIME: u32 = 0x3008_0040;
pub const PR_IMPORTANCE: u32 = 0x0017_0003;
pub const PR_PRIORITY: u32 = 0x0026_0003;
pub const PR_SENSITIVITY: u32 = 0x0036_0003;
pub const PR_MESSAGE_FLAGS: u32 = 0x0e07_0003;
pub const PR_MESSAGE_SIZE: u32 = 0x0e08_0003;
pub const PR_HASATTACH: u32 = 0x0e1b_000b;
pub const PR_DELETE_AFTER_SUBMIT: u32 = 0x0e01_000b;
pub const PR_ORIGINATOR_DELIVERY_REPORT_REQUESTED: u32 = 0x0023_000b;
pub const PR_READ_RECEIPT_REQUESTED: u32 = 0x0029_000b;
pub const PR_REPLY_REQUESTED: u32 = 0x0c17_000b;
pub const PR_DISPLAY_NAME: u32 = 0x3001_001f;
pub const PR_DISPLAY_NAME_A: u32 = 0x3001_001e;
pub const PR_CONTENT_COUNT: u32 = 0x3602_0003;
pub const PR_CONTENT_UNREAD: u32 = 0x3603_0003;
pub const PR_TRANSPORT_MESSAGE_HEADERS: u32 = 0x007d_001f;
pub const PR_TRANSPORT_MESSAGE_HEADERS_A: u32 = 0x007d_001e;
pub const PR_INTERNET_MESSAGE_ID: u32 = 0x1035_001f;
pub const PR_INTERNET_MESSAGE_ID_A: u32 = 0x1035_001e;
pub const PR_IN_REPLY_TO_ID: u32 = 0x1042_001f;
pub const PR_IN_REPLY_TO_ID_A: u32 = 0x1042_001e;
pub const PR_INTERNET_REFERENCES: u32 = 0x1039_001f;
pub const PR_INTERNET_REFERENCES_A: u32 = 0x1039_001e;
pub const PR_CONVERSATION_TOPIC: u32 = 0x0070_001f;
pub const PR_CONVERSATION_TOPIC_A: u32 = 0x0070_001e;
pub const PR_CONVERSATION_INDEX: u32 = 0x0071_0102;
pub const PR_RECIPIENT_TYPE: u32 = 0x0c15_0003;
pub const PR_RECIPIENT_DISPLAY_NAME: u32 = 0x5ff6_001f;
pub const PR_RECIPIENT_DISPLAY_NAME_A: u32 = 0x5ff6_001e;
pub const PR_RECIPIENT_EMAIL_ADDRESS: u32 = 0x3003_001f;
pub const PR_RECIPIENT_EMAIL_ADDRESS_A: u32 = 0x3003_001e;
pub const PR_RECIPIENT_ADDRTYPE: u32 = 0x3002_001f;
pub const PR_RECIPIENT_ADDRTYPE_A: u32 = 0x3002_001e;
pub const PR_SMTP_ADDRESS: u32 = 0x39fe_001f;
pub const PR_SMTP_ADDRESS_A: u32 = 0x39fe_001e;
pub const PR_BODY: u32 = 0x1000_001f;
pub const PR_BODY_A: u32 = 0x1000_001e;
pub const PR_RTF_COMPRESSED: u32 = 0x1009_0102;
pub const PR_HTML_STRING: u32 = 0x1013_001f;
pub const PR_HTML_STRING_A: u32 = 0x1013_001e;
pub const PR_HTML: u32 = 0x1013_0102;
pub const PR_ATTACH_DATA_OBJ: u32 = 0x3701_000d;
pub const PR_ATTACH_DATA_BIN: u32 = 0x3701_0102;
pub const PR_ATTACH_FILENAME: u32 = 0x3704_001f;
pub const PR_ATTACH_FILENAME_A: u32 = 0x3704_001e;
pub const PR_ATTACH_METHOD: u32 = 0x3705_0003;
pub const PR_ATTACH_LONG_FILENAME: u32 = 0x3707_001f;
pub const PR_ATTACH_LONG_FILENAME_A: u32 = 0x3707_001e;
pub const PR_RENDERING_POSITION: u32 = 0x370b_0003;
pub const PR_ATTACH_MIME_TAG: u32 = 0x370e_001f;
pub const PR_ATTACH_MIME_TAG_A: u32 = 0x370e_001e;
pub const PR_ATTACH_MIME_SEQUENCE: u32 = 0x3710_0003;
pub const PR_ATTACH_CONTENT_ID: u32 = 0x3712_001f;
pub const PR_ATTACH_CONTENT_ID_A: u32 = 0x3712_001e;
pub const PR_ATTACH_SIZE: u32 = 0x0e20_0003;
pub const PR_ATTACHMENT_HIDDEN: u32 = 0x7ffe_000b;

const KNOWN_VALUE_TYPE_CODES: &[u16] = &[
    0x0002, // PtypInteger16
    0x0003, // PtypInteger32
    0x0005, // PtypFloating64
    0x000b, // PtypBoolean
    0x000d, // PtypObject
    0x0014, // PtypInteger64
    0x001e, // PtypString8
    0x001f, // PtypString
    0x0040, // PtypTime
    0x0048, // PtypGuid
    0x0102, // PtypBinary
    0x1002, // PtypMultipleInteger16
    0x1003, // PtypMultipleInteger32
    0x101e, // PtypMultipleString8
    0x101f, // PtypMultipleString
    0x1040, // PtypMultipleTime
    0x1102, // PtypMultipleBinary
];

pub const SELECTED_PROPERTIES: &[MapiPropertyDef] = &[
    MapiPropertyDef {
        tag: PR_SUBJECT,
        name: "subject",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_MESSAGE_CLASS,
        name: "message_class",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_SENDER_NAME,
        name: "sender_name",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_SENDER_EMAIL_ADDRESS,
        name: "sender_email_address",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_SENDER_ADDRTYPE,
        name: "sender_address_type",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_SENT_REPRESENTING_EMAIL_ADDRESS,
        name: "sent_representing_email_address",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_SENT_REPRESENTING_ADDRTYPE,
        name: "sent_representing_address_type",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_RECEIVED_BY_ADDRTYPE,
        name: "received_by_address_type",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_RECEIVED_BY_EMAIL_ADDRESS,
        name: "received_by_email_address",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_RECEIVED_REPRESENTING_ADDRTYPE,
        name: "received_representing_address_type",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_RECEIVED_REPRESENTING_EMAIL_ADDRESS,
        name: "received_representing_email_address",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_CLIENT_SUBMIT_TIME,
        name: "sent_at",
        value_type: MapiValueType::FileTime,
    },
    MapiPropertyDef {
        tag: PR_MESSAGE_DELIVERY_TIME,
        name: "received_at",
        value_type: MapiValueType::FileTime,
    },
    MapiPropertyDef {
        tag: PR_CREATION_TIME,
        name: "created_at",
        value_type: MapiValueType::FileTime,
    },
    MapiPropertyDef {
        tag: PR_LAST_MODIFICATION_TIME,
        name: "modified_at",
        value_type: MapiValueType::FileTime,
    },
    MapiPropertyDef {
        tag: PR_IMPORTANCE,
        name: "importance",
        value_type: MapiValueType::Integer32,
    },
    MapiPropertyDef {
        tag: PR_PRIORITY,
        name: "priority",
        value_type: MapiValueType::Integer32,
    },
    MapiPropertyDef {
        tag: PR_SENSITIVITY,
        name: "sensitivity",
        value_type: MapiValueType::Integer32,
    },
    MapiPropertyDef {
        tag: PR_MESSAGE_FLAGS,
        name: "message_flags",
        value_type: MapiValueType::Integer32,
    },
    MapiPropertyDef {
        tag: PR_MESSAGE_SIZE,
        name: "message_size",
        value_type: MapiValueType::Integer32,
    },
    MapiPropertyDef {
        tag: PR_HASATTACH,
        name: "has_attachments",
        value_type: MapiValueType::Boolean,
    },
    MapiPropertyDef {
        tag: PR_DELETE_AFTER_SUBMIT,
        name: "delete_after_submit",
        value_type: MapiValueType::Boolean,
    },
    MapiPropertyDef {
        tag: PR_ORIGINATOR_DELIVERY_REPORT_REQUESTED,
        name: "delivery_report_requested",
        value_type: MapiValueType::Boolean,
    },
    MapiPropertyDef {
        tag: PR_READ_RECEIPT_REQUESTED,
        name: "read_receipt_requested",
        value_type: MapiValueType::Boolean,
    },
    MapiPropertyDef {
        tag: PR_REPLY_REQUESTED,
        name: "reply_requested",
        value_type: MapiValueType::Boolean,
    },
    MapiPropertyDef {
        tag: PR_DISPLAY_NAME,
        name: "display_name",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_CONTENT_COUNT,
        name: "content_count",
        value_type: MapiValueType::Integer32,
    },
    MapiPropertyDef {
        tag: PR_CONTENT_UNREAD,
        name: "content_unread",
        value_type: MapiValueType::Integer32,
    },
    MapiPropertyDef {
        tag: PR_TRANSPORT_MESSAGE_HEADERS,
        name: "transport_message_headers",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_INTERNET_MESSAGE_ID,
        name: "internet_message_id",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_IN_REPLY_TO_ID,
        name: "in_reply_to_id",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_INTERNET_REFERENCES,
        name: "internet_references",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_CONVERSATION_TOPIC,
        name: "conversation_topic",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_CONVERSATION_INDEX,
        name: "conversation_index",
        value_type: MapiValueType::Binary,
    },
    MapiPropertyDef {
        tag: PR_RECIPIENT_TYPE,
        name: "recipient_type",
        value_type: MapiValueType::Integer32,
    },
    MapiPropertyDef {
        tag: PR_RECIPIENT_DISPLAY_NAME,
        name: "recipient_display_name",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_RECIPIENT_EMAIL_ADDRESS,
        name: "recipient_email_address",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_RECIPIENT_ADDRTYPE,
        name: "recipient_address_type",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_SMTP_ADDRESS,
        name: "smtp_address",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_BODY,
        name: "body_text",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_RTF_COMPRESSED,
        name: "body_rtf_compressed",
        value_type: MapiValueType::Binary,
    },
    MapiPropertyDef {
        tag: PR_HTML_STRING,
        name: "body_html_unicode",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_HTML,
        name: "body_html",
        value_type: MapiValueType::Binary,
    },
    MapiPropertyDef {
        tag: PR_ATTACH_DATA_OBJ,
        name: "attachment_data_object",
        value_type: MapiValueType::Unknown,
    },
    MapiPropertyDef {
        tag: PR_ATTACH_DATA_BIN,
        name: "attachment_data",
        value_type: MapiValueType::Binary,
    },
    MapiPropertyDef {
        tag: PR_ATTACH_FILENAME,
        name: "attachment_filename",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_ATTACH_METHOD,
        name: "attachment_method",
        value_type: MapiValueType::Integer32,
    },
    MapiPropertyDef {
        tag: PR_ATTACH_LONG_FILENAME,
        name: "attachment_long_filename",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_RENDERING_POSITION,
        name: "attachment_rendering_position",
        value_type: MapiValueType::Integer32,
    },
    MapiPropertyDef {
        tag: PR_ATTACH_MIME_TAG,
        name: "attachment_mime_tag",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_ATTACH_MIME_SEQUENCE,
        name: "attachment_mime_sequence",
        value_type: MapiValueType::Integer32,
    },
    MapiPropertyDef {
        tag: PR_ATTACH_CONTENT_ID,
        name: "attachment_content_id",
        value_type: MapiValueType::String,
    },
    MapiPropertyDef {
        tag: PR_ATTACH_SIZE,
        name: "attachment_size",
        value_type: MapiValueType::Integer32,
    },
    MapiPropertyDef {
        tag: PR_ATTACHMENT_HIDDEN,
        name: "attachment_hidden",
        value_type: MapiValueType::Boolean,
    },
];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MapiValue {
    String(String),
    Integer32(i32),
    Integer64(i64),
    Boolean(bool),
    FileTime(String),
    Binary(Vec<u8>),
    Unknown(Vec<u8>),
}

pub fn property_def(tag: u32) -> Option<MapiPropertyDef> {
    SELECTED_PROPERTIES
        .iter()
        .copied()
        .find(|def| def.tag == tag)
        .or_else(|| string8_property_def(tag))
}

pub fn property_value_type_code(tag: u32) -> u16 {
    (tag & 0xffff) as u16
}

pub fn has_known_value_type(tag: u32) -> bool {
    KNOWN_VALUE_TYPE_CODES.contains(&property_value_type_code(tag))
}

pub fn byte_swapped_tag(tag: u32) -> u32 {
    tag.swap_bytes()
}

fn string8_property_def(tag: u32) -> Option<MapiPropertyDef> {
    let name = match tag {
        PR_SUBJECT_A => "subject",
        PR_MESSAGE_CLASS_A => "message_class",
        PR_SENDER_NAME_A => "sender_name",
        PR_SENDER_EMAIL_ADDRESS_A => "sender_email_address",
        PR_SENDER_ADDRTYPE_A => "sender_address_type",
        PR_SENT_REPRESENTING_EMAIL_ADDRESS_A => "sent_representing_email_address",
        PR_SENT_REPRESENTING_ADDRTYPE_A => "sent_representing_address_type",
        PR_RECEIVED_BY_ADDRTYPE_A => "received_by_address_type",
        PR_RECEIVED_BY_EMAIL_ADDRESS_A => "received_by_email_address",
        PR_RECEIVED_REPRESENTING_ADDRTYPE_A => "received_representing_address_type",
        PR_RECEIVED_REPRESENTING_EMAIL_ADDRESS_A => "received_representing_email_address",
        PR_DISPLAY_NAME_A => "display_name",
        PR_TRANSPORT_MESSAGE_HEADERS_A => "transport_message_headers",
        PR_INTERNET_MESSAGE_ID_A => "internet_message_id",
        PR_IN_REPLY_TO_ID_A => "in_reply_to_id",
        PR_INTERNET_REFERENCES_A => "internet_references",
        PR_CONVERSATION_TOPIC_A => "conversation_topic",
        PR_RECIPIENT_DISPLAY_NAME_A => "recipient_display_name",
        PR_RECIPIENT_EMAIL_ADDRESS_A => "recipient_email_address",
        PR_RECIPIENT_ADDRTYPE_A => "recipient_address_type",
        PR_SMTP_ADDRESS_A => "smtp_address",
        PR_BODY_A => "body_text",
        PR_HTML_STRING_A => "body_html_unicode",
        PR_ATTACH_FILENAME_A => "attachment_filename",
        PR_ATTACH_LONG_FILENAME_A => "attachment_long_filename",
        PR_ATTACH_MIME_TAG_A => "attachment_mime_tag",
        PR_ATTACH_CONTENT_ID_A => "attachment_content_id",
        _ => return None,
    };

    Some(MapiPropertyDef {
        tag,
        name,
        value_type: MapiValueType::String8,
    })
}

#[allow(clippy::chunks_exact_to_as_chunks)]
pub fn decode_value(value_type: MapiValueType, raw: &[u8]) -> PstdResult<MapiValue> {
    match value_type {
        MapiValueType::String => {
            let utf16: Vec<u16> = raw
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .take_while(|value| *value != 0)
                .collect();
            Ok(MapiValue::String(String::from_utf16_lossy(&utf16)))
        }
        MapiValueType::String8 => Ok(MapiValue::String(decode_string8(raw))),
        MapiValueType::Integer32 => {
            if raw.len() < 4 {
                return Err(PstdError::pst_parse(None, "i32 value too short"));
            }
            Ok(MapiValue::Integer32(i32::from_le_bytes([
                raw[0], raw[1], raw[2], raw[3],
            ])))
        }
        MapiValueType::Integer64 => {
            if raw.len() < 8 {
                return Err(PstdError::pst_parse(None, "i64 value too short"));
            }
            Ok(MapiValue::Integer64(i64::from_le_bytes([
                raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
            ])))
        }
        MapiValueType::Boolean => Ok(MapiValue::Boolean(raw.first().copied().unwrap_or(0) != 0)),
        MapiValueType::FileTime => {
            if raw.len() < 8 {
                return Err(PstdError::pst_parse(None, "filetime value too short"));
            }
            let ticks = u64::from_le_bytes([
                raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
            ]);
            Ok(MapiValue::FileTime(format!("filetime:{ticks}")))
        }
        MapiValueType::Binary => Ok(MapiValue::Binary(raw.to_vec())),
        MapiValueType::Unknown => Ok(MapiValue::Unknown(raw.to_vec())),
    }
}

fn decode_string8(raw: &[u8]) -> String {
    let nul_index = raw.iter().position(|byte| *byte == 0).unwrap_or(raw.len());

    // String8 is a byte-oriented MAPI value. The parser's documented default
    // charset is ISO-8859-1, whose code points map one-to-one to these bytes.
    raw[..nul_index]
        .iter()
        .map(|byte| char::from(*byte))
        .collect()
}

pub fn value_summary(value: &MapiValue) -> String {
    match value {
        MapiValue::String(value) => value.clone(),
        MapiValue::Integer32(value) => value.to_string(),
        MapiValue::Integer64(value) => value.to_string(),
        MapiValue::Boolean(value) => value.to_string(),
        MapiValue::FileTime(value) => value.clone(),
        MapiValue::Binary(value) => format!("{} bytes", value.len()),
        MapiValue::Unknown(value) => format!("{} bytes unknown", value.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        byte_swapped_tag, decode_value, has_known_value_type, property_def, MapiValue,
        MapiValueType, PR_ATTACH_DATA_OBJ, PR_BODY_A, PR_SUBJECT, PR_SUBJECT_A,
    };

    #[test]
    fn maps_string8_aliases_to_selected_property_defs() {
        let subject = property_def(PR_SUBJECT_A).unwrap();
        assert_eq!(subject.name, "subject");
        assert_eq!(subject.value_type, MapiValueType::String8);

        let body = property_def(PR_BODY_A).unwrap();
        assert_eq!(body.name, "body_text");
        assert_eq!(body.value_type, MapiValueType::String8);
    }

    #[test]
    fn identifies_known_property_value_types() {
        assert!(has_known_value_type(PR_SUBJECT));
        assert!(has_known_value_type(PR_ATTACH_DATA_OBJ));
        assert!(!has_known_value_type(0x001f_0037));
        assert_eq!(byte_swapped_tag(0x1f00_3700), PR_SUBJECT);
    }

    #[test]
    fn decodes_string8_values() {
        let value = decode_value(MapiValueType::String8, b"Hello\0ignored").unwrap();
        match value {
            MapiValue::String(value) => assert_eq!(value, "Hello"),
            other => panic!("unexpected decoded value: {other:?}"),
        }
    }

    #[test]
    fn decodes_string8_high_bit_bytes_without_replacement() {
        let value = decode_value(MapiValueType::String8, &[b'c', b'a', b'f', 0xe9, 0]).unwrap();
        match value {
            MapiValue::String(value) => assert_eq!(value, "café"),
            other => panic!("unexpected decoded value: {other:?}"),
        }

        let value = decode_value(MapiValueType::String8, &[0x80, 0xff]).unwrap();
        match value {
            MapiValue::String(value) => assert_eq!(value, "\u{80}\u{ff}"),
            other => panic!("unexpected decoded value: {other:?}"),
        }
    }
}
