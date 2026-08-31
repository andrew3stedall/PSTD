use crate::error::{PstdError, PstdResult};
use encoding_rs::{BIG5, EUC_KR, GBK, SHIFT_JIS, UTF_8};

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
pub const PR_MESSAGE_CODEPAGE: u32 = 0x3ffd_0003;
pub const PR_INTERNET_CPID: u32 = 0x3fde_0003;
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
        tag: PR_MESSAGE_CODEPAGE,
        name: "message_codepage",
        value_type: MapiValueType::Integer32,
    },
    MapiPropertyDef {
        tag: PR_INTERNET_CPID,
        name: "internet_cpid",
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

pub fn canonical_fallback_charset(value: &str) -> Option<&'static str> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("iso-8859-1")
        || value.eq_ignore_ascii_case("latin-1")
        || value.eq_ignore_ascii_case("latin1")
    {
        Some("iso-8859-1")
    } else if value.eq_ignore_ascii_case("windows-1252") || value.eq_ignore_ascii_case("cp1252") {
        Some("windows-1252")
    } else if value.eq_ignore_ascii_case("shift-jis")
        || value.eq_ignore_ascii_case("shift_jis")
        || value.eq_ignore_ascii_case("cp932")
        || value.eq_ignore_ascii_case("windows-31j")
    {
        Some("shift-jis")
    } else if value.eq_ignore_ascii_case("gbk") || value.eq_ignore_ascii_case("cp936") {
        Some("gbk")
    } else if value.eq_ignore_ascii_case("euc-kr")
        || value.eq_ignore_ascii_case("euc_kr")
        || value.eq_ignore_ascii_case("cp949")
    {
        Some("euc-kr")
    } else if value.eq_ignore_ascii_case("big5") || value.eq_ignore_ascii_case("cp950") {
        Some("big5")
    } else if value.eq_ignore_ascii_case("utf-8") || value.eq_ignore_ascii_case("utf8") {
        Some("utf-8")
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CharsetResolution {
    pub charset: String,
    pub source: String,
    pub status: String,
}

/// Maps the small set of code pages that PSTD can decode without guessing.
/// Unknown pages deliberately return None so callers can retain raw bytes and
/// use their configured fallback instead of silently corrupting text.
pub fn charset_for_code_page(code_page: i32) -> Option<&'static str> {
    match code_page {
        65001 => Some("utf-8"),
        1252 => Some("windows-1252"),
        28591 => Some("iso-8859-1"),
        932 => Some("shift-jis"),
        936 => Some("gbk"),
        949 => Some("euc-kr"),
        950 => Some("big5"),
        _ => None,
    }
}

pub fn resolve_string8_charset(
    message_codepage: Option<&[u8]>,
    internet_cpid: Option<&[u8]>,
    fallback_charset: Option<&str>,
) -> CharsetResolution {
    let fallback = canonical_fallback_charset(fallback_charset.unwrap_or("iso-8859-1"))
        .unwrap_or("iso-8859-1");

    if let Some(charset) = fallback_charset.and_then(canonical_fallback_charset) {
        return CharsetResolution {
            charset: charset.to_string(),
            source: "explicit_override".to_string(),
            status: format!(
                "charset_override_authoritative; charset={charset}; code_page_metadata=ignored"
            ),
        };
    }

    let message = code_page_evidence("message_codepage", message_codepage);
    let internet = code_page_evidence("internet_cpid", internet_cpid);
    let evidence = [message.as_ref(), internet.as_ref()];
    let valid = evidence
        .iter()
        .filter_map(|item| item.as_ref().and_then(|item| item.charset))
        .collect::<Vec<_>>();
    let issues = evidence
        .iter()
        .filter_map(|item| item.as_ref().and_then(|item| item.issue.as_deref()))
        .collect::<Vec<_>>();

    if !issues.is_empty() {
        return CharsetResolution {
            charset: fallback.to_string(),
            source: "fallback".to_string(),
            status: format!(
                "charset_metadata_rejected; {}; fallback_charset={fallback}",
                issues.join(",")
            ),
        };
    }

    if valid.len() == 2 && valid[0] != valid[1] {
        return CharsetResolution {
            charset: fallback.to_string(),
            source: "fallback".to_string(),
            status: format!(
                "charset_metadata_conflict; message_codepage_charset={}; internet_cpid_charset={}; fallback_charset={fallback}",
                valid[0], valid[1]
            ),
        };
    }

    if let Some(charset) = valid.first() {
        let source = match (message_codepage.is_some(), internet_cpid.is_some()) {
            (true, true) => "message_codepage+internet_cpid",
            (true, false) => "message_codepage",
            (false, true) => "internet_cpid",
            (false, false) => "fallback",
        };
        let code_pages = [message, internet]
            .into_iter()
            .flatten()
            .filter_map(|item| item.code_page)
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(",");
        return CharsetResolution {
            charset: (*charset).to_string(),
            source: source.to_string(),
            status: format!(
                "charset_metadata_selected; source={source}; code_pages={code_pages}; charset={charset}"
            ),
        };
    }

    CharsetResolution {
        charset: fallback.to_string(),
        source: "default_fallback".to_string(),
        status: format!("charset_metadata_absent; fallback_charset={fallback}"),
    }
}

#[derive(Debug, Clone)]
struct CodePageEvidence {
    code_page: Option<i32>,
    charset: Option<&'static str>,
    issue: Option<String>,
}

fn code_page_evidence(name: &str, raw: Option<&[u8]>) -> Option<CodePageEvidence> {
    let raw = raw?;
    if raw.len() != 4 {
        return Some(CodePageEvidence {
            code_page: None,
            charset: None,
            issue: Some(format!("{name}_invalid_length={}", raw.len())),
        });
    }
    let code_page = i32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
    if !(1..=65535).contains(&code_page) {
        return Some(CodePageEvidence {
            code_page: Some(code_page),
            charset: None,
            issue: Some(format!("{name}_out_of_range={code_page}")),
        });
    }
    let Some(charset) = charset_for_code_page(code_page) else {
        return Some(CodePageEvidence {
            code_page: Some(code_page),
            charset: None,
            issue: Some(format!("{name}_unsupported={code_page}")),
        });
    };
    Some(CodePageEvidence {
        code_page: Some(code_page),
        charset: Some(charset),
        issue: None,
    })
}

pub fn decode_value(value_type: MapiValueType, raw: &[u8]) -> PstdResult<MapiValue> {
    decode_value_with_fallback(value_type, raw, None)
}

#[allow(clippy::chunks_exact_to_as_chunks)]
pub fn decode_value_with_fallback(
    value_type: MapiValueType,
    raw: &[u8],
    fallback_charset: Option<&str>,
) -> PstdResult<MapiValue> {
    match value_type {
        MapiValueType::String => {
            let utf16: Vec<u16> = raw
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .take_while(|value| *value != 0)
                .collect();
            Ok(MapiValue::String(String::from_utf16_lossy(&utf16)))
        }
        MapiValueType::String8 => Ok(MapiValue::String(
            decode_string8_with_status(raw, fallback_charset).0,
        )),
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

/// Decode a NUL-terminated legacy String8 value and report whether the selected
/// codec had to replace malformed input. Raw bytes remain available in the
/// owning property record when `had_errors` is true.
pub fn decode_string8_with_status(
    raw: &[u8],
    fallback_charset: Option<&str>,
) -> (String, bool) {
    let nul_index = raw.iter().position(|byte| *byte == 0).unwrap_or(raw.len());
    let raw = &raw[..nul_index];

    match fallback_charset.and_then(canonical_fallback_charset) {
        Some("utf-8") => decode_with_encoding(UTF_8, raw),
        Some("windows-1252") => (
            raw.iter().map(|byte| windows_1252_char(*byte)).collect(),
            false,
        ),
        Some("shift-jis") => decode_with_encoding(SHIFT_JIS, raw),
        Some("gbk") => decode_with_encoding(GBK, raw),
        Some("euc-kr") => decode_with_encoding(EUC_KR, raw),
        Some("big5") => decode_with_encoding(BIG5, raw),
        _ => {
            // String8 is a byte-oriented MAPI value. The parser's documented
            // default charset is ISO-8859-1, whose code points map one-to-one
            // to these bytes.
            (raw.iter().map(|byte| char::from(*byte)).collect(), false)
        }
    }
}

fn decode_with_encoding(encoding: &'static encoding_rs::Encoding, raw: &[u8]) -> (String, bool) {
    let (decoded, _, had_errors) = encoding.decode(raw);
    (decoded.into_owned(), had_errors)
}

fn windows_1252_char(byte: u8) -> char {
    match byte {
        0x80 => '\u{20ac}',
        0x82 => '\u{201a}',
        0x83 => '\u{192}',
        0x84 => '\u{201e}',
        0x85 => '\u{2026}',
        0x86 => '\u{2020}',
        0x87 => '\u{2021}',
        0x88 => '\u{2c6}',
        0x89 => '\u{2030}',
        0x8a => '\u{160}',
        0x8b => '\u{2039}',
        0x8c => '\u{152}',
        0x8e => '\u{17d}',
        0x91 => '\u{2018}',
        0x92 => '\u{2019}',
        0x93 => '\u{201c}',
        0x94 => '\u{201d}',
        0x95 => '\u{2022}',
        0x96 => '\u{2013}',
        0x97 => '\u{2014}',
        0x98 => '\u{2dc}',
        0x99 => '\u{2122}',
        0x9a => '\u{161}',
        0x9b => '\u{203a}',
        0x9c => '\u{153}',
        0x9e => '\u{17e}',
        0x9f => '\u{178}',
        _ => char::from(byte),
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

#[cfg(test)]
mod tests {
    use super::{
        byte_swapped_tag, canonical_fallback_charset, decode_string8_with_status, decode_value,
        decode_value_with_fallback, has_known_value_type, property_def, resolve_string8_charset,
        MapiValue, MapiValueType, PR_ATTACH_DATA_OBJ, PR_BODY_A, PR_INTERNET_CPID,
        PR_MESSAGE_CODEPAGE, PR_SUBJECT, PR_SUBJECT_A,
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

    #[test]
    fn applies_explicit_fallback_charset_to_string8_values() {
        let value =
            decode_value_with_fallback(MapiValueType::String8, &[0x80, 0x93, 0x94], Some("cp1252"))
                .unwrap();
        match value {
            MapiValue::String(value) => assert_eq!(value, "€“”"),
            other => panic!("unexpected decoded value: {other:?}"),
        }

        let value = decode_value_with_fallback(
            MapiValueType::String8,
            &[b'c', b'a', b'f', 0xc3, 0xa9],
            Some("utf-8"),
        )
        .unwrap();
        match value {
            MapiValue::String(value) => assert_eq!(value, "café"),
            other => panic!("unexpected decoded value: {other:?}"),
        }
    }

    #[test]
    fn canonicalizes_supported_fallback_charset_aliases() {
        assert_eq!(canonical_fallback_charset("Latin1"), Some("iso-8859-1"));
        assert_eq!(
            canonical_fallback_charset("WINDOWS-1252"),
            Some("windows-1252")
        );
        assert_eq!(canonical_fallback_charset("utf8"), Some("utf-8"));
        assert_eq!(canonical_fallback_charset("cp932"), Some("shift-jis"));
        assert_eq!(canonical_fallback_charset("936"), None);
        assert_eq!(canonical_fallback_charset("cp949"), Some("euc-kr"));
        assert_eq!(canonical_fallback_charset("cp950"), Some("big5"));
        assert_eq!(canonical_fallback_charset("koi8-r"), None);
    }

    #[test]
    fn decodes_common_non_western_string8_code_pages() {
        let cases = [
            ("shift-jis", &[0x93, 0xfa, 0x96, 0x7b, 0x8c, 0xea][..], "日本語"),
            ("gbk", &[0xd6, 0xd0, 0xce, 0xc4][..], "中文"),
            ("euc-kr", &[0xc7, 0xd1, 0xb1, 0xdb][..], "한글"),
            ("big5", &[0xa4, 0xa4, 0xa4, 0xe5][..], "中文"),
        ];

        for (charset, raw, expected) in cases {
            let (decoded, had_errors) = decode_string8_with_status(raw, Some(charset));
            assert_eq!(decoded, expected, "charset={charset}");
            assert!(!had_errors, "charset={charset}");
        }
    }

    #[test]
    fn reports_malformed_non_western_string8_without_dropping_raw_contract() {
        let (decoded, had_errors) = decode_string8_with_status(&[0x82, 0x20], Some("shift-jis"));
        assert!(had_errors);
        assert!(decoded.contains('\u{fffd}'));

        let value = decode_value_with_fallback(
            MapiValueType::String8,
            &[0x82, 0x20],
            Some("shift-jis"),
        )
        .unwrap();
        assert_eq!(value, MapiValue::String(decoded));
    }

    #[test]
    fn selects_supported_message_code_pages() {
        let cp1252 = 1252i32.to_le_bytes();
        let resolution = resolve_string8_charset(Some(&cp1252), None, None);
        assert_eq!(resolution.charset, "windows-1252");
        assert_eq!(resolution.source, "message_codepage");
        assert!(resolution.status.contains("code_pages=1252"));

        let cpid = 65001i32.to_le_bytes();
        let resolution = resolve_string8_charset(None, Some(&cpid), None);
        assert_eq!(resolution.charset, "utf-8");
        assert_eq!(resolution.source, "internet_cpid");
        assert!(property_def(PR_MESSAGE_CODEPAGE).is_some());
        assert!(property_def(PR_INTERNET_CPID).is_some());
    }

    #[test]
    fn rejects_conflicting_or_unsupported_code_page_evidence() {
        let cp1252 = 1252i32.to_le_bytes();
        let cp65001 = 65001i32.to_le_bytes();
        let resolution = resolve_string8_charset(Some(&cp1252), Some(&cp65001), None);
        assert_eq!(resolution.charset, "iso-8859-1");
        assert_eq!(resolution.source, "fallback");
        assert!(resolution.status.contains("charset_metadata_conflict"));

        let unsupported = 9500i32.to_le_bytes();
        let resolution = resolve_string8_charset(Some(&unsupported), None, None);
        assert_eq!(resolution.charset, "iso-8859-1");
        assert!(resolution
            .status
            .contains("message_codepage_unsupported=9500"));
    }

    #[test]
    fn selects_common_non_western_message_code_pages() {
        for (code_page, charset) in [
            (932i32, "shift-jis"),
            (936i32, "gbk"),
            (949i32, "euc-kr"),
            (950i32, "big5"),
        ] {
            let raw = code_page.to_le_bytes();
            let resolution = resolve_string8_charset(Some(&raw), None, None);
            assert_eq!(resolution.charset, charset, "code_page={code_page}");
            assert_eq!(resolution.source, "message_codepage");
        }
    }

    #[test]
    fn explicit_fallback_overrides_code_page_metadata() {
        let cp1252 = 1252i32.to_le_bytes();
        let resolution = resolve_string8_charset(Some(&cp1252), None, Some("utf8"));
        assert_eq!(resolution.charset, "utf-8");
        assert_eq!(resolution.source, "explicit_override");
        assert!(resolution.status.contains("authoritative"));
    }
}
