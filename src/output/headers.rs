//! RFC-aware helpers for generated mail headers and MIME parameters.
//!
//! Canonical PSTD records remain UTF-8 and lossless. These helpers are only used
//! when projecting those records into RFC 5322/MIME output, where raw non-ASCII
//! header text or parameter values are not interoperable on their own.

/// Remove header-injection characters and surrounding whitespace.
pub fn clean_header_value(value: &str) -> Option<String> {
    if value.contains('\r') || value.contains('\n') {
        return None;
    }
    let cleaned = value
        .chars()
        .filter(|character| !character.is_control() || *character == '\t')
        .collect::<String>();
    let cleaned = cleaned.trim();
    (!cleaned.is_empty()).then(|| cleaned.to_string())
}

/// Normalize a Content-ID for use in a MIME header.
///
/// MAPI stores the identifier with and without RFC angle brackets depending
/// on the producer. Keep the identifier itself unchanged while ensuring that
/// generated MIME has exactly one pair of brackets. Reject malformed or
/// non-ASCII values rather than emitting an unsafe header.
pub fn normalize_content_id(value: &str) -> Option<String> {
    let cleaned = clean_header_value(value)?;
    let identifier = match (cleaned.starts_with('<'), cleaned.ends_with('>')) {
        (true, true) if cleaned.len() > 2 => &cleaned[1..cleaned.len() - 1],
        (true, true) => return None,
        (true, false) | (false, true) => return None,
        (false, false) => cleaned.as_str(),
    };
    if !identifier.is_ascii()
        || identifier.is_empty()
        || identifier
            .chars()
            .any(|character| character.is_whitespace() || matches!(character, '<' | '>'))
    {
        return None;
    }
    Some(format!("<{identifier}>"))
}

/// Encode an unstructured RFC 5322 header value when it contains non-ASCII
/// characters. ASCII values are returned unchanged after sanitisation.
pub fn encode_unstructured_value(value: &str) -> String {
    let value = clean_header_value(value).unwrap_or_default();
    if value.is_ascii() {
        return value;
    }
    encode_utf8_words(&value)
}

/// Encode a display name for use in an RFC 5322 address field.
pub fn encode_display_name(value: &str) -> String {
    let value = clean_header_value(value).unwrap_or_default();
    if value.is_ascii()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || " ._-".contains(character))
    {
        return value;
    }
    if value.is_ascii() {
        return format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""));
    }
    encode_utf8_words(&value)
}

/// Build an RFC 2231 MIME parameter. ASCII values retain the compact quoted
/// form. Non-ASCII or long values retain an ASCII fallback and add a UTF-8
/// percent-encoded continuation/value for interoperable readers.
pub fn encode_mime_parameter(name: &str, value: &str) -> String {
    let value = clean_header_value(value).unwrap_or_default();
    let quoted = quote_parameter_value(&value);
    let encoded = percent_encode_utf8(value.as_bytes());
    if value.is_ascii() && encoded.len() <= 60 {
        return format!("{name}=\"{quoted}\"");
    }

    let fallback = ascii_fallback(&value);
    let segments = split_percent_encoded(&encoded, 60);
    if segments.len() == 1 {
        return format!("{name}=\"{fallback}\"; {name}*=UTF-8''{}", segments[0]);
    }

    let mut result = format!("{name}=\"{fallback}\"");
    for (index, segment) in segments.iter().enumerate() {
        if index == 0 {
            result.push_str(&format!("; {name}*0*=UTF-8''{segment}"));
        } else {
            result.push_str(&format!("; {name}*{index}*={segment}"));
        }
    }
    result
}

fn encode_utf8_words(value: &str) -> String {
    const MAX_RAW_BYTES_PER_WORD: usize = 45;
    let mut words = Vec::new();
    let mut chunk = String::new();
    let mut chunk_bytes = 0usize;
    for character in value.chars() {
        let character_bytes = character.len_utf8();
        if chunk_bytes > 0 && chunk_bytes + character_bytes > MAX_RAW_BYTES_PER_WORD {
            words.push(encoded_word(&chunk));
            chunk.clear();
            chunk_bytes = 0;
        }
        chunk.push(character);
        chunk_bytes += character_bytes;
    }
    if !chunk.is_empty() {
        words.push(encoded_word(&chunk));
    }
    words.join(" ")
}

fn encoded_word(value: &str) -> String {
    format!("=?UTF-8?B?{}?=", base64_encode(value.as_bytes()))
}

fn quote_parameter_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn ascii_fallback(value: &str) -> String {
    let mut fallback = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_ascii() && !character.is_ascii_control() && !matches!(character, '"' | '\\')
        {
            fallback.push(character);
        } else {
            fallback.push('_');
        }
    }
    if fallback.is_empty() {
        "attachment".to_string()
    } else {
        fallback
    }
}

fn percent_encode_utf8(bytes: &[u8]) -> String {
    fn is_attr_char(byte: u8) -> bool {
        byte.is_ascii_alphanumeric()
            || matches!(
                byte,
                b'!' | b'#' | b'$' | b'&' | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~'
            )
    }

    let mut encoded = String::with_capacity(bytes.len());
    for &byte in bytes {
        if is_attr_char(byte) {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push(hex_digit(byte >> 4));
            encoded.push(hex_digit(byte & 0x0f));
        }
    }
    encoded
}

fn split_percent_encoded(value: &str, max_len: usize) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut index = 0usize;
    while index < value.len() {
        let width = if value.as_bytes()[index] == b'%' {
            3
        } else {
            1
        };
        if !current.is_empty() && current.len() + width > max_len {
            segments.push(std::mem::take(&mut current));
        }
        current.push_str(&value[index..index + width]);
        index += width;
    }
    if !current.is_empty() {
        segments.push(current);
    }
    if segments.is_empty() {
        vec![String::new()]
    } else {
        segments
    }
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'A' + value - 10) as char,
        _ => unreachable!("hex digit is four bits"),
    }
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(TABLE[(first >> 2) as usize] as char);
        output.push(TABLE[(((first & 0x03) << 4) | (second >> 4)) as usize] as char);
        output.push(if chunk.len() > 1 {
            TABLE[(((second & 0x0f) << 2) | (third >> 6)) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            TABLE[(third & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{
        encode_display_name, encode_mime_parameter, encode_unstructured_value, normalize_content_id,
    };

    #[test]
    fn keeps_ascii_headers_byte_compact() {
        assert_eq!(encode_unstructured_value("Hello"), "Hello");
        assert_eq!(encode_display_name("Jane Doe"), "Jane Doe");
        assert_eq!(encode_display_name("Doe, Jane"), "\"Doe, Jane\"");
    }

    #[test]
    fn encodes_non_ascii_headers_as_utf8_base64_words() {
        assert_eq!(
            encode_unstructured_value("Résumé — réunion"),
            "=?UTF-8?B?UsOpc3Vtw6kg4oCUIHLDqXVuaW9u?="
        );
        assert_eq!(
            encode_display_name("Zoë Example"),
            "=?UTF-8?B?Wm/DqyBFeGFtcGxl?="
        );
    }

    #[test]
    fn emits_rfc2231_utf8_filename_and_preserves_ascii_fallback() {
        let parameter = encode_mime_parameter("filename", "résumé final.pdf");
        assert!(parameter.starts_with("filename=\"r_sum_ final.pdf\"; filename*=UTF-8''"));
        assert!(parameter.contains("r%C3%A9sum%C3%A9%20final.pdf"));
    }

    #[test]
    fn splits_long_parameters_without_breaking_percent_triplets() {
        let parameter = encode_mime_parameter("filename", &format!("é{}", "a".repeat(100)));
        assert!(parameter.contains("filename*0*=UTF-8''"));
        assert!(parameter.contains("filename*1*="));
        assert!(!parameter.contains("%C3% A9"));
    }

    #[test]
    fn rejects_header_injection_and_repeats_encoding_deterministically() {
        let injected = "subject\r\nX-Injected: yes";
        assert_eq!(encode_unstructured_value(injected), "");
        assert_eq!(encode_display_name(injected), "");
        assert_eq!(encode_mime_parameter("filename", injected), "filename=\"\"");

        let first = encode_mime_parameter("filename", "résumé final.pdf");
        let second = encode_mime_parameter("filename", "résumé final.pdf");
        assert_eq!(first, second);
    }

    #[test]
    fn normalizes_content_ids_without_synthesizing_the_identifier() {
        assert_eq!(
            normalize_content_id("image-1@example.com").as_deref(),
            Some("<image-1@example.com>")
        );
        assert_eq!(
            normalize_content_id("<image-1@example.com>").as_deref(),
            Some("<image-1@example.com>")
        );
    }

    #[test]
    fn rejects_malformed_content_ids() {
        for value in [
            "",
            "<missing-end",
            "missing-start>",
            "<>",
            "image one@example.com",
        ] {
            assert!(normalize_content_id(value).is_none(), "accepted {value:?}");
        }
        assert!(normalize_content_id("image\r\nX-Injected: yes").is_none());
        assert!(normalize_content_id("é@example.com").is_none());
    }
}
