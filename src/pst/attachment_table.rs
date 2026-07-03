use std::collections::HashMap;

use crate::pst::attachments::{
    attachment_payload, attachment_payload_from_properties, AttachmentMetadata, AttachmentPayload,
};
use crate::pst::mapi::{decode_value, property_def};
use crate::pst::payload::PayloadBlock;
use crate::pst::property_context::{PropertyContext, PropertyValue};
use crate::pst::table_context::{TableContext, TableRow};

const COMPACT_ATTACHMENT_TABLE_MAGIC: &[u8; 4] = b"CATB";
const UTF16_COMPACT_ATTACHMENT_TABLE_MAGIC: &[u8; 4] = b"CATW";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttachmentTableWiringReport {
    pub row_count: usize,
    pub payload_count: usize,
    pub missing_payload_count: usize,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttachmentSubnodeWiringReport {
    pub subnode_block_count: usize,
    pub parsed_table_count: usize,
    pub parse_error_count: usize,
    pub row_count: usize,
    pub payload_count: usize,
    pub missing_payload_count: usize,
    pub parse_error_offsets: Vec<u64>,
    pub parse_error_reasons: Vec<String>,
    pub table_statuses: Vec<String>,
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

pub fn attachment_payloads_from_subnode_blocks(
    message_key: &str,
    blocks: &[PayloadBlock],
) -> (Vec<AttachmentPayload>, AttachmentSubnodeWiringReport) {
    let mut payloads = Vec::new();
    let mut parsed_table_count = 0usize;
    let mut parse_error_count = 0usize;
    let mut row_count = 0usize;
    let mut missing_payload_count = 0usize;
    let mut parse_error_offsets = Vec::new();
    let mut parse_error_reasons = Vec::new();
    let mut table_statuses = Vec::new();
    let mut next_ordinal = 0usize;

    for block in blocks {
        match decode_attachment_block(message_key, block, next_ordinal) {
            Ok((mut block_payloads, report)) => {
                parsed_table_count += 1;
                table_statuses.push(report.status);
                row_count += report.row_count;
                missing_payload_count += report.missing_payload_count;
                next_ordinal += report.row_count;
                payloads.append(&mut block_payloads);
            }
            Err(reason) => {
                parse_error_count += 1;
                parse_error_offsets.push(block.block_ref.offset.0);
                parse_error_reasons.push(reason);
            }
        }
    }

    let status = if payloads.is_empty() && parsed_table_count == 0 && parse_error_count == 0 {
        "attachment_subnodes_empty"
    } else if !payloads.is_empty() && parse_error_count == 0 && missing_payload_count == 0 {
        "attachment_subnode_payloads_wired"
    } else if !payloads.is_empty() {
        "attachment_subnode_payloads_partially_wired"
    } else if parsed_table_count > 0 {
        "attachment_subnode_tables_without_payloads"
    } else {
        "attachment_subnode_tables_unavailable"
    };

    let report = AttachmentSubnodeWiringReport {
        subnode_block_count: blocks.len(),
        parsed_table_count,
        parse_error_count,
        row_count,
        payload_count: payloads.len(),
        missing_payload_count,
        parse_error_offsets,
        parse_error_reasons,
        table_statuses,
        status: status.to_string(),
    };

    (payloads, report)
}

fn decode_attachment_block(
    message_key: &str,
    block: &PayloadBlock,
    start_ordinal: usize,
) -> Result<(Vec<AttachmentPayload>, AttachmentTableWiringReport), String> {
    if block.bytes.starts_with(COMPACT_ATTACHMENT_TABLE_MAGIC) {
        return decode_compact_attachment_table(message_key, &block.bytes, start_ordinal);
    }
    if block.bytes.starts_with(UTF16_COMPACT_ATTACHMENT_TABLE_MAGIC) {
        return decode_utf16_compact_attachment_table(message_key, &block.bytes, start_ordinal);
    }

    match TableContext::parse_with_report(&block.bytes, block.block_ref.offset.0) {
        Ok(table_report) => {
            let (payloads, mut report) =
                attachment_payloads_from_table(message_key, &table_report.context);
            report.status = table_report.status;
            Ok((payloads, report))
        }
        Err(reason) => Err(reason.to_string()),
    }
}

fn decode_compact_attachment_table(
    message_key: &str,
    bytes: &[u8],
    start_ordinal: usize,
) -> Result<(Vec<AttachmentPayload>, AttachmentTableWiringReport), String> {
    if bytes.len() < 8 {
        return Err("compact attachment table buffer too short".to_string());
    }

    let row_count = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
    let mut cursor = 8usize;
    let mut payloads = Vec::new();
    let mut missing_payload_count = 0usize;

    for ordinal_offset in 0..row_count {
        if cursor + 8 > bytes.len() {
            return Err(format!(
                "compact attachment table row {ordinal_offset} header truncated"
            ));
        }

        let filename_len = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
        let content_type_len = u16::from_le_bytes([bytes[cursor + 2], bytes[cursor + 3]]) as usize;
        let data_len = u32::from_le_bytes([
            bytes[cursor + 4],
            bytes[cursor + 5],
            bytes[cursor + 6],
            bytes[cursor + 7],
        ]) as usize;
        cursor += 8;

        let row_len = filename_len
            .checked_add(content_type_len)
            .and_then(|value| value.checked_add(data_len))
            .ok_or_else(|| "compact attachment table row length overflow".to_string())?;
        if cursor + row_len > bytes.len() {
            return Err(format!(
                "compact attachment table row {ordinal_offset} data truncated"
            ));
        }

        let filename = decode_utf8_field(&bytes[cursor..cursor + filename_len]);
        cursor += filename_len;
        let content_type = decode_utf8_field(&bytes[cursor..cursor + content_type_len]);
        cursor += content_type_len;
        let data = bytes[cursor..cursor + data_len].to_vec();
        cursor += data_len;

        if data.is_empty() {
            missing_payload_count += 1;
            continue;
        }

        payloads.push(attachment_payload(
            message_key,
            start_ordinal + ordinal_offset,
            AttachmentMetadata {
                filename_original: filename,
                content_type,
                is_inline: false,
                content_id: None,
            },
            data,
        ));
    }

    let status = if missing_payload_count == 0 {
        "compact_attachment_table_payloads_wired"
    } else if payloads.is_empty() {
        "compact_attachment_table_payloads_unavailable"
    } else {
        "compact_attachment_table_payloads_partially_wired"
    };

    Ok((
        payloads,
        AttachmentTableWiringReport {
            row_count,
            payload_count: row_count.saturating_sub(missing_payload_count),
            missing_payload_count,
            status: status.to_string(),
        },
    ))
}

fn decode_utf16_compact_attachment_table(
    message_key: &str,
    bytes: &[u8],
    start_ordinal: usize,
) -> Result<(Vec<AttachmentPayload>, AttachmentTableWiringReport), String> {
    if bytes.len() < 8 {
        return Err("utf16 compact attachment table buffer too short".to_string());
    }

    let row_count = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
    let mut cursor = 8usize;
    let mut payloads = Vec::new();
    let mut missing_payload_count = 0usize;

    for ordinal_offset in 0..row_count {
        if cursor + 8 > bytes.len() {
            return Err(format!(
                "utf16 compact attachment table row {ordinal_offset} header truncated"
            ));
        }

        let filename_len = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
        let content_type_len = u16::from_le_bytes([bytes[cursor + 2], bytes[cursor + 3]]) as usize;
        let data_len = u32::from_le_bytes([
            bytes[cursor + 4],
            bytes[cursor + 5],
            bytes[cursor + 6],
            bytes[cursor + 7],
        ]) as usize;
        cursor += 8;

        if !filename_len.is_multiple_of(2) || !content_type_len.is_multiple_of(2) {
            return Err(format!(
                "utf16 compact attachment table row {ordinal_offset} has odd string byte length"
            ));
        }

        let row_len = filename_len
            .checked_add(content_type_len)
            .and_then(|value| value.checked_add(data_len))
            .ok_or_else(|| "utf16 compact attachment table row length overflow".to_string())?;
        if cursor + row_len > bytes.len() {
            return Err(format!(
                "utf16 compact attachment table row {ordinal_offset} data truncated"
            ));
        }

        let filename = decode_utf16_field(&bytes[cursor..cursor + filename_len]);
        cursor += filename_len;
        let content_type = decode_utf16_field(&bytes[cursor..cursor + content_type_len]);
        cursor += content_type_len;
        let data = bytes[cursor..cursor + data_len].to_vec();
        cursor += data_len;

        if data.is_empty() {
            missing_payload_count += 1;
            continue;
        }

        payloads.push(attachment_payload(
            message_key,
            start_ordinal + ordinal_offset,
            AttachmentMetadata {
                filename_original: filename,
                content_type,
                is_inline: false,
                content_id: None,
            },
            data,
        ));
    }

    let status = if missing_payload_count == 0 {
        "utf16_compact_attachment_table_payloads_wired"
    } else if payloads.is_empty() {
        "utf16_compact_attachment_table_payloads_unavailable"
    } else {
        "utf16_compact_attachment_table_payloads_partially_wired"
    };

    Ok((
        payloads,
        AttachmentTableWiringReport {
            row_count,
            payload_count: row_count.saturating_sub(missing_payload_count),
            missing_payload_count,
            status: status.to_string(),
        },
    ))
}

fn decode_utf8_field(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        None
    } else {
        Some(
            String::from_utf8_lossy(bytes)
                .trim_end_matches('\0')
                .to_string(),
        )
    }
}

fn decode_utf16_field(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    let units = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .take_while(|unit| *unit != 0)
        .collect::<Vec<_>>();
    if units.is_empty() {
        None
    } else {
        Some(String::from_utf16_lossy(&units))
    }
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
    use super::{attachment_payloads_from_subnode_blocks, attachment_payloads_from_table};
    use crate::pst::mapi::{PR_ATTACH_DATA_BIN, PR_ATTACH_LONG_FILENAME, PR_ATTACH_MIME_TAG};
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset};
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

    #[test]
    fn wires_attachment_payloads_from_subnode_table_blocks() {
        let block = payload_block(100, 0, table_buf());

        let (payloads, report) = attachment_payloads_from_subnode_blocks("msg_123", &[block]);
        assert_eq!(payloads.len(), 1);
        assert_eq!(payloads[0].record.filename_safe, "report.pdf");
        assert_eq!(payloads[0].bytes, b"attachment bytes");
        assert_eq!(report.subnode_block_count, 1);
        assert_eq!(report.parsed_table_count, 1);
        assert_eq!(report.payload_count, 1);
        assert_eq!(report.table_statuses, vec!["table_context_parsed"]);
        assert_eq!(report.status, "attachment_subnode_payloads_wired");
    }

    #[test]
    fn wires_compact_attachment_table_blocks() {
        let block = payload_block(100, 0, compact_attachment_table_buf());

        let (payloads, report) = attachment_payloads_from_subnode_blocks("msg_123", &[block]);
        assert_eq!(payloads.len(), 1);
        assert_eq!(payloads[0].record.filename_safe, "compact.txt");
        assert_eq!(
            payloads[0].record.content_type.as_deref(),
            Some("text/plain")
        );
        assert_eq!(payloads[0].bytes, b"compact bytes");
        assert_eq!(report.subnode_block_count, 1);
        assert_eq!(report.parsed_table_count, 1);
        assert_eq!(
            report.table_statuses,
            vec!["compact_attachment_table_payloads_wired"]
        );
        assert_eq!(report.status, "attachment_subnode_payloads_wired");
    }

    #[test]
    fn wires_utf16_compact_attachment_table_blocks() {
        let block = payload_block(100, 0, utf16_compact_attachment_table_buf());

        let (payloads, report) = attachment_payloads_from_subnode_blocks("msg_123", &[block]);
        assert_eq!(payloads.len(), 1);
        assert_eq!(payloads[0].record.filename_safe, "utf16-report.pdf");
        assert_eq!(
            payloads[0].record.content_type.as_deref(),
            Some("application/pdf")
        );
        assert_eq!(payloads[0].bytes, b"utf16 compact bytes");
        assert_eq!(report.subnode_block_count, 1);
        assert_eq!(report.parsed_table_count, 1);
        assert_eq!(
            report.table_statuses,
            vec!["utf16_compact_attachment_table_payloads_wired"]
        );
        assert_eq!(report.status, "attachment_subnode_payloads_wired");
    }

    #[test]
    fn preserves_parse_error_fallback_for_invalid_utf16_compact_rows() {
        let block = payload_block(100, 4096, invalid_utf16_compact_attachment_table_buf());

        let (payloads, report) = attachment_payloads_from_subnode_blocks("msg_123", &[block]);
        assert!(payloads.is_empty());
        assert_eq!(report.parse_error_count, 1);
        assert_eq!(report.parse_error_offsets, vec![4096]);
        assert!(report.parse_error_reasons[0].contains("odd string byte length"));
        assert_eq!(report.status, "attachment_subnode_tables_unavailable");
    }

    #[test]
    fn reports_compact_attachment_table_missing_payloads() {
        let block = payload_block(100, 0, compact_attachment_table_missing_payload_buf());

        let (payloads, report) = attachment_payloads_from_subnode_blocks("msg_123", &[block]);
        assert!(payloads.is_empty());
        assert_eq!(report.parsed_table_count, 1);
        assert_eq!(report.missing_payload_count, 1);
        assert_eq!(
            report.table_statuses,
            vec!["compact_attachment_table_payloads_unavailable"]
        );
        assert_eq!(report.status, "attachment_subnode_tables_without_payloads");
    }

    #[test]
    fn reports_unparseable_subnode_table_blocks() {
        let block = payload_block(100, 4096, vec![1, 2, 3]);

        let (payloads, report) = attachment_payloads_from_subnode_blocks("msg_123", &[block]);
        assert!(payloads.is_empty());
        assert_eq!(report.parse_error_count, 1);
        assert_eq!(report.parse_error_offsets, vec![4096]);
        assert_eq!(report.parse_error_reasons.len(), 1);
        assert!(report.parse_error_reasons[0].contains("table context buffer too short"));
        assert_eq!(report.status, "attachment_subnode_tables_unavailable");
    }

    #[test]
    fn reports_partial_subnode_attachment_compatibility() {
        let blocks = vec![
            payload_block(100, 0, table_buf()),
            payload_block(101, 8192, vec![1, 2, 3]),
            payload_block(102, 16384, missing_payload_table_buf()),
        ];

        let (payloads, report) = attachment_payloads_from_subnode_blocks("msg_123", &blocks);
        assert_eq!(payloads.len(), 1);
        assert_eq!(report.subnode_block_count, 3);
        assert_eq!(report.parsed_table_count, 2);
        assert_eq!(report.parse_error_count, 1);
        assert_eq!(report.row_count, 2);
        assert_eq!(report.payload_count, 1);
        assert_eq!(report.missing_payload_count, 1);
        assert_eq!(report.parse_error_offsets, vec![8192]);
        assert_eq!(report.status, "attachment_subnode_payloads_partially_wired");
    }

    fn payload_block(block_id: u64, offset: u64, bytes: Vec<u8>) -> PayloadBlock {
        PayloadBlock {
            block_id: BlockId(block_id),
            block_ref: BlockRef {
                block_id: BlockId(block_id),
                offset: ByteOffset(offset),
                size: bytes.len() as u64,
            },
            bytes,
            status: "payload_loaded".to_string(),
        }
    }

    fn table_buf() -> Vec<u8> {
        let attachment_bytes = b"attachment bytes";
        let filename = utf16le_fixed("report.pdf", 24);
        let mime = utf16le_fixed("application/pdf", 32);
        let row_width = attachment_bytes.len() + filename.len() + mime.len();
        let mut buf = Vec::new();
        buf.extend_from_slice(&3u16.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&(row_width as u16).to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&PR_ATTACH_DATA_BIN.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&(attachment_bytes.len() as u16).to_le_bytes());
        buf.extend_from_slice(&PR_ATTACH_LONG_FILENAME.to_le_bytes());
        buf.extend_from_slice(&(attachment_bytes.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(filename.len() as u16).to_le_bytes());
        buf.extend_from_slice(&PR_ATTACH_MIME_TAG.to_le_bytes());
        buf.extend_from_slice(&((attachment_bytes.len() + filename.len()) as u16).to_le_bytes());
        buf.extend_from_slice(&(mime.len() as u16).to_le_bytes());
        buf.extend_from_slice(attachment_bytes);
        buf.extend_from_slice(&filename);
        buf.extend_from_slice(&mime);
        buf
    }

    fn missing_payload_table_buf() -> Vec<u8> {
        let filename = utf16le_fixed("missing.pdf", 24);
        let row_width = filename.len();
        let mut buf = Vec::new();
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&(row_width as u16).to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&PR_ATTACH_LONG_FILENAME.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&(filename.len() as u16).to_le_bytes());
        buf.extend_from_slice(&filename);
        buf
    }

    fn compact_attachment_table_buf() -> Vec<u8> {
        let filename = b"compact.txt";
        let mime = b"text/plain";
        let payload = b"compact bytes";
        let mut buf = Vec::new();
        buf.extend_from_slice(b"CATB");
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&(filename.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(mime.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(filename);
        buf.extend_from_slice(mime);
        buf.extend_from_slice(payload);
        buf
    }

    fn compact_attachment_table_missing_payload_buf() -> Vec<u8> {
        let filename = b"compact-missing.txt";
        let mime = b"text/plain";
        let mut buf = Vec::new();
        buf.extend_from_slice(b"CATB");
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&(filename.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(mime.len() as u16).to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(filename);
        buf.extend_from_slice(mime);
        buf
    }

    fn utf16_compact_attachment_table_buf() -> Vec<u8> {
        let filename = utf16le("utf16-report.pdf");
        let mime = utf16le("application/pdf");
        let payload = b"utf16 compact bytes";
        let mut buf = Vec::new();
        buf.extend_from_slice(b"CATW");
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&(filename.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(mime.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(&filename);
        buf.extend_from_slice(&mime);
        buf.extend_from_slice(payload);
        buf
    }

    fn invalid_utf16_compact_attachment_table_buf() -> Vec<u8> {
        let filename = b"x";
        let mime = utf16le("text/plain");
        let payload = b"payload";
        let mut buf = Vec::new();
        buf.extend_from_slice(b"CATW");
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&(filename.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(mime.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(filename);
        buf.extend_from_slice(&mime);
        buf.extend_from_slice(payload);
        buf
    }

    fn utf16le(value: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        for unit in value.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes
    }

    fn utf16le_fixed(value: &str, len: usize) -> Vec<u8> {
        let mut bytes = utf16le(value);
        bytes.resize(len, 0);
        bytes
    }
}
