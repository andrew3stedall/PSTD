use std::collections::{BTreeMap, BTreeSet};

use crate::output::ids;
use crate::output::metadata::{AttachmentRecord, CidReferenceRecord};
use crate::pst::messages::BodyPayload;

/// Correlate HTML `cid:` references with attachment Content-IDs.
///
/// The correlation is deliberately conservative: it only uses an explicitly
/// stored Content-ID, never a filename or ordinal. A duplicate Content-ID is
/// reported as ambiguous and an unmatched reference or inline attachment is
/// retained as evidence instead of being silently discarded.
pub fn build_cid_references(
    body_payloads: &[BodyPayload],
    attachments: &[AttachmentRecord],
) -> Vec<CidReferenceRecord> {
    let mut output = Vec::new();
    let mut message_bodies = BTreeMap::<String, Vec<&BodyPayload>>::new();
    for body in body_payloads
        .iter()
        .filter(|payload| payload.record.body_type == "html")
    {
        message_bodies
            .entry(body.record.message_key.clone())
            .or_default()
            .push(body);
    }
    for bodies in message_bodies.values_mut() {
        bodies.sort_by_key(|body| body.record.body_key.clone());
    }

    let mut message_attachments = BTreeMap::<String, Vec<&AttachmentRecord>>::new();
    for attachment in attachments {
        message_attachments
            .entry(attachment.message_key.clone())
            .or_default()
            .push(attachment);
    }
    for records in message_attachments.values_mut() {
        records.sort_by_key(|attachment| {
            (
                attachment.ordinal,
                attachment.attachment_key.clone(),
            )
        });
    }

    let mut message_keys = message_bodies.keys().cloned().collect::<BTreeSet<_>>();
    message_keys.extend(message_attachments.keys().cloned());
    for message_key in message_keys {
        let bodies = message_bodies
            .get(&message_key)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let candidates = message_attachments
            .get(&message_key)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let mut by_content_id = BTreeMap::<String, Vec<&AttachmentRecord>>::new();
        for attachment in candidates {
            if let Some(content_id) = attachment.content_id.as_deref() {
                if let Some(normalized) = normalize_cid(content_id) {
                    by_content_id
                        .entry(normalized)
                        .or_default()
                        .push(*attachment);
                }
            }
        }

        let mut referenced_attachment_keys = BTreeSet::new();
        for body in bodies {
            for (reference_ordinal, (offset, raw_cid)) in extract_cid_references(&body.bytes)
                .into_iter()
                .enumerate()
            {
                let normalized_cid = normalize_cid(&raw_cid);
                let matches = normalized_cid
                    .as_ref()
                    .and_then(|cid| by_content_id.get(cid))
                    .cloned()
                    .unwrap_or_default();
                for attachment in &matches {
                    referenced_attachment_keys.insert(attachment.attachment_key.clone());
                }
                let status = match normalized_cid.as_ref() {
                    None => "invalid_html_cid_reference",
                    Some(_) if matches.is_empty() => "unmatched_html_cid_reference",
                    Some(_) if matches.len() > 1 => "duplicate_attachment_content_id",
                    Some(_) => "matched_unique_attachment",
                };
                let attachment_key =
                    (matches.len() == 1).then(|| matches[0].attachment_key.clone());
                let attachment_keys = matches
                    .iter()
                    .map(|attachment| attachment.attachment_key.clone())
                    .collect::<Vec<_>>();
                let reference_key = ids::stable_id(
                    "cid",
                    &[
                        &body.record.message_key,
                        &body.record.body_key,
                        "html",
                        &reference_ordinal.to_string(),
                        &raw_cid,
                    ],
                );
                output.push(CidReferenceRecord {
                    message_key: body.record.message_key.clone(),
                    reference_key,
                    reference_kind: "html_cid".to_string(),
                    body_key: Some(body.record.body_key.clone()),
                    cid: Some(raw_cid),
                    normalized_cid,
                    attachment_key,
                    attachment_keys,
                    byte_offset: offset as u64,
                    status: status.to_string(),
                    source: format!("body:{}:byte:{offset}", body.record.body_key),
                    authoritative: body.record.status == "extracted",
                    synthetic: false,
                });
            }
        }

        for attachment in candidates {
            if !attachment.is_inline
                || referenced_attachment_keys.contains(&attachment.attachment_key)
            {
                continue;
            }
            let (normalized_cid, status) = match attachment.content_id.as_deref() {
                Some(content_id) => match normalize_cid(content_id) {
                    Some(normalized) => (Some(normalized), "unmatched_inline_attachment"),
                    None => (None, "inline_attachment_content_id_invalid"),
                },
                None => (None, "inline_attachment_content_id_absent"),
            };
            let reference_key = ids::stable_id(
                "cid",
                &[
                    &attachment.message_key,
                    &attachment.attachment_key,
                    "inline_attachment",
                ],
            );
            output.push(CidReferenceRecord {
                message_key: attachment.message_key.clone(),
                reference_key,
                reference_kind: "inline_attachment".to_string(),
                body_key: None,
                cid: attachment.content_id.clone(),
                normalized_cid,
                attachment_key: Some(attachment.attachment_key.clone()),
                attachment_keys: vec![attachment.attachment_key.clone()],
                byte_offset: 0,
                status: status.to_string(),
                source: format!("attachment:{}", attachment.attachment_key),
                authoritative: attachment.extraction_status == "extracted",
                synthetic: false,
            });
        }
    }

    output.sort_by_key(|record| record.reference_key.clone());
    output
}

fn extract_cid_references(bytes: &[u8]) -> Vec<(usize, String)> {
    let mut references = Vec::new();
    let mut cursor = 0;
    while cursor + 4 <= bytes.len() {
        if bytes[cursor..cursor + 4].eq_ignore_ascii_case(b"cid:")
            && (cursor == 0 || !is_cid_token_character(bytes[cursor - 1]))
        {
            let start = cursor + 4;
            let mut end = start;
            while end < bytes.len() && !is_cid_delimiter(bytes[end]) {
                end += 1;
            }
            references.push((cursor, String::from_utf8_lossy(&bytes[start..end]).into_owned()));
            cursor = end.max(cursor + 4);
        } else {
            cursor += 1;
        }
    }
    references
}

fn is_cid_token_character(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'+' | b'.')
}

fn is_cid_delimiter(byte: u8) -> bool {
    byte.is_ascii_whitespace()
        || matches!(byte, b'"' | b'\'' | b'<' | b'>' | b'(' | b')' | b'[' | b']' | b'{' | b'}' | b',' | b';' | b'#')
}

fn normalize_cid(value: &str) -> Option<String> {
    let value = value.trim();
    let value = match (value.starts_with('<'), value.ends_with('>')) {
        (true, true) if value.len() > 2 => &value[1..value.len() - 1],
        (true, true) => return None,
        (true, false) | (false, true) => return None,
        (false, false) => value,
    };
    if value.is_empty() || !value.is_ascii() {
        return None;
    }

    let mut decoded = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return None;
            }
            let high = hex_nibble(bytes[index + 1])?;
            let low = hex_nibble(bytes[index + 2])?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    if decoded.is_empty()
        || !decoded.is_ascii()
        || decoded
            .iter()
            .any(|byte| byte.is_ascii_whitespace() || matches!(byte, b'<' | b'>'))
    {
        return None;
    }
    String::from_utf8(decoded).ok()
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::build_cid_references;
    use crate::output::metadata::AttachmentRecord;
    use crate::pst::attachments::{attachment_payload, AttachmentMetadata};
    use crate::pst::messages::html_body_payload;

    fn attachment(
        message_key: &str,
        ordinal: usize,
        content_id: Option<&str>,
        is_inline: bool,
    ) -> AttachmentRecord {
        attachment_payload(
            message_key,
            ordinal,
            AttachmentMetadata {
                filename_original: Some(format!("image-{ordinal}.png")),
                content_type: Some("image/png".to_string()),
                is_inline,
                content_id: content_id.map(str::to_string),
                ..AttachmentMetadata::default()
            },
            vec![1, 2, 3],
        )
        .record
    }

    #[test]
    fn correlates_unique_and_percent_encoded_cids() {
        let body = html_body_payload(
            "msg",
            br#"<img src="CID:logo%40example.test"><img src='cid:other@example.test'>"#,
        );
        let attachments = vec![
            attachment("msg", 0, Some("<logo@example.test>"), true),
            attachment("msg", 1, Some("other@example.test"), false),
        ];
        let records = build_cid_references(&[body], &attachments);

        assert_eq!(records.len(), 2);
        assert!(records
            .iter()
            .all(|record| record.status == "matched_unique_attachment"));
        assert!(records.iter().any(|record| {
            record.normalized_cid.as_deref() == Some("logo@example.test")
                && record.attachment_key.as_deref() == Some(&attachments[0].attachment_key)
        }));
    }

    #[test]
    fn retains_unmatched_duplicate_and_invalid_evidence() {
        let body = html_body_payload(
            "msg",
            b"<img src=\"cid:missing@example.test\"><img src=\"cid:dup@example.test\"><img src=\"cid:\">",
        );
        let attachments = vec![
            attachment("msg", 0, Some("dup@example.test"), true),
            attachment("msg", 1, Some("<dup@example.test>"), true),
            attachment("msg", 2, Some("orphan@example.test"), true),
        ];
        let records = build_cid_references(&[body], &attachments);

        assert!(records
            .iter()
            .any(|record| record.status == "unmatched_html_cid_reference"));
        assert!(records
            .iter()
            .any(|record| record.status == "duplicate_attachment_content_id"));
        assert!(records
            .iter()
            .any(|record| record.status == "invalid_html_cid_reference"));
        assert!(records
            .iter()
            .any(|record| record.status == "unmatched_inline_attachment"));
    }

    #[test]
    fn ignores_cid_text_embedded_in_a_larger_token() {
        let body = html_body_payload("msg", b"notcid:ignored@example.test");
        let attachment = attachment("msg", 0, Some("ignored@example.test"), false);
        assert!(build_cid_references(&[body], &[attachment]).is_empty());
    }

    #[test]
    fn retains_inline_attachment_without_html_body() {
        let attachment = attachment("msg", 0, Some("orphan@example.test"), true);
        let records = build_cid_references(&[], &[attachment]);

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].status, "unmatched_inline_attachment");
    }

}
