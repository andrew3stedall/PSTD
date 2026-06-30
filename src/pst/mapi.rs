use crate::error::{PstdError, PstdResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MapiValueType {
    String,
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
pub const PR_MESSAGE_CLASS: u32 = 0x001a_001f;
pub const PR_SENDER_NAME: u32 = 0x0c1a_001f;
pub const PR_SENDER_EMAIL_ADDRESS: u32 = 0x0c1f_001f;
pub const PR_SENDER_ADDRTYPE: u32 = 0x0c1e_001f;
pub const PR_SENT_REPRESENTING_EMAIL_ADDRESS: u32 = 0x0065_001f;
pub const PR_CLIENT_SUBMIT_TIME: u32 = 0x0039_0040;
pub const PR_MESSAGE_DELIVERY_TIME: u32 = 0x0e06_0040;
pub const PR_CREATION_TIME: u32 = 0x3007_0040;
pub const PR_LAST_MODIFICATION_TIME: u32 = 0x3008_0040;
pub const PR_IMPORTANCE: u32 = 0x0017_0003;
pub const PR_MESSAGE_FLAGS: u32 = 0x0e07_0003;
pub const PR_MESSAGE_SIZE: u32 = 0x0e08_0003;
pub const PR_HASATTACH: u32 = 0x0e1b_000b;
pub const PR_DISPLAY_NAME: u32 = 0x3001_001f;
pub const PR_CONTENT_COUNT: u32 = 0x3602_0003;
pub const PR_CONTENT_UNREAD: u32 = 0x3603_0003;
pub const PR_TRANSPORT_MESSAGE_HEADERS: u32 = 0x007d_001f;
pub const PR_INTERNET_MESSAGE_ID: u32 = 0x1035_001f;
pub const PR_IN_REPLY_TO_ID: u32 = 0x1042_001f;
pub const PR_INTERNET_REFERENCES: u32 = 0x1039_001f;
pub const PR_CONVERSATION_TOPIC: u32 = 0x0070_001f;
pub const PR_CONVERSATION_INDEX: u32 = 0x0071_0102;
pub const PR_RECIPIENT_TYPE: u32 = 0x0c15_0003;
pub const PR_RECIPIENT_DISPLAY_NAME: u32 = 0x5ff6_001f;
pub const PR_RECIPIENT_EMAIL_ADDRESS: u32 = 0x3003_001f;
pub const PR_RECIPIENT_ADDRTYPE: u32 = 0x3002_001f;
pub const PR_SMTP_ADDRESS: u32 = 0x39fe_001f;

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
}

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
