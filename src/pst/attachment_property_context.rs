use crate::output::metadata::AttachmentRecord;
use crate::pst::attachments::unavailable_attachment_record_from_properties;
use crate::pst::bth::BthMap;
use crate::pst::heap::HeapOnNode;
use crate::pst::mapi::{
    MapiValue, PR_ATTACH_DATA_BIN, PR_ATTACH_FILENAME, PR_ATTACH_FILENAME_A,
    PR_ATTACH_LONG_FILENAME, PR_ATTACH_LONG_FILENAME_A, PR_ATTACH_METHOD, PR_ATTACH_SIZE,
};
use crate::pst::payload::PayloadBlock;
use crate::pst::property_context::PropertyContext;

const HEAP_CLIENT_PROPERTY_CONTEXT: u8 = 0xbc;
const UNICODE_SLBLOCK_TYPE: u8 = 0x02;
const UNICODE_SLBLOCK_LEAF_LEVEL: u8 = 0x00;
const UNICODE_SLBLOCK_HEADER_BYTES: usize = 8;
const UNICODE_SLENTRY_BYTES: usize = 24;
const HNID_TYPE_MASK: u32 = 0x1f;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentPropertyContextReport {
    pub property_context_count: usize,
    pub filename_record_count: usize,
    pub rejected_context_count: usize,
    pub status: String,
}

/// Extracts only filename-bearing attachment metadata from validated heap Property Contexts.
pub fn attachment_records_from_property_context_subnodes(
    message_key: &str,
    blocks: &[PayloadBlock],
) -> (Vec<AttachmentRecord>, AttachmentPropertyContextReport) {
    let mut records = Vec::new();
    let mut property_context_count = 0usize;
    let mut rejected_context_count = 0usize;

    for block in blocks {
        let Ok(heap) = HeapOnNode::parse(&block.bytes, block.block_ref.offset.0) else {
            continue;
        };
        if heap.header.client_signature != HEAP_CLIENT_PROPERTY_CONTEXT {
            continue;
        }
        property_context_count += 1;

        let Ok(bth) =
            BthMap::parse_property_context_from_heap(&heap, &block.bytes, block.block_ref.offset.0)
        else {
            rejected_context_count += 1;
            continue;
        };
        let Ok(report) = PropertyContext::from_bth_with_report(&bth) else {
            rejected_context_count += 1;
            continue;
        };
        let Some(record) =
            filename_attachment_record(message_key, records.len(), &report.context, blocks)
        else {
            rejected_context_count += 1;
            continue;
        };
        records.push(record);
    }

    let filename_record_count = records.len();
    let status = if records.is_empty() {
        "attachment_property_context_filename_absent"
    } else if rejected_context_count == 0 {
        "attachment_property_context_filenames_extracted"
    } else {
        "attachment_property_context_filenames_partially_extracted"
    };
    (
        records,
        AttachmentPropertyContextReport {
            property_context_count,
            filename_record_count,
            rejected_context_count,
            status: status.to_string(),
        },
    )
}

fn filename_attachment_record(
    message_key: &str,
    ordinal: usize,
    properties: &PropertyContext,
    blocks: &[PayloadBlock],
) -> Option<AttachmentRecord> {
    let filename = first_non_empty_string(
        properties,
        &[
            PR_ATTACH_LONG_FILENAME,
            PR_ATTACH_LONG_FILENAME_A,
            PR_ATTACH_FILENAME,
            PR_ATTACH_FILENAME_A,
        ],
    )?;
    let method = positive_integer32_property(properties, PR_ATTACH_METHOD)?;
    let declared_size = non_negative_integer32_property(properties, PR_ATTACH_SIZE)?;

    let extraction_status = match resolved_subnode_data_reference(properties, blocks) {
        Some((data_nid, data_bid)) => format!(
            "attachment_metadata_extracted_payload_subnode_reference_resolved; data_nid=0x{data_nid:08x}; data_bid=0x{data_bid:x}"
        ),
        None => "attachment_metadata_extracted_payload_reference_unresolved".to_string(),
    };
    let mut record = unavailable_attachment_record_from_properties(
        message_key,
        ordinal,
        properties,
        &extraction_status,
    );
    if record.attachment_method != Some(method)
        || record.declared_size_bytes != Some(declared_size as u64)
    {
        return None;
    }
    record.filename_original = Some(filename.clone());
    record.filename_safe = crate::pst::attachments::safe_filename(Some(&filename), ordinal);
    record.extension = crate::pst::attachments::file_extension(&record.filename_safe);
    record.archive_path = format!(
        "attachments/{message_key}/{}_{}",
        record.attachment_key, record.filename_safe
    );
    Some(record)
}

fn resolved_subnode_data_reference(
    properties: &PropertyContext,
    blocks: &[PayloadBlock],
) -> Option<(u32, u64)> {
    let data_nid = attachment_data_nid(properties)?;
    let data_bid = blocks
        .iter()
        .find_map(|block| slblock_data_bid_for_nid(&block.bytes, data_nid))?;
    blocks
        .iter()
        .any(|block| block.block_id.0 == data_bid)
        .then_some((data_nid, data_bid))
}

fn attachment_data_nid(properties: &PropertyContext) -> Option<u32> {
    let value = properties.value(PR_ATTACH_DATA_BIN)?;
    if value.raw.len() != 4 {
        return None;
    }
    let hnid = u32::from_le_bytes(value.raw.as_slice().try_into().ok()?);
    (hnid & HNID_TYPE_MASK != 0).then_some(hnid)
}

fn slblock_data_bid_for_nid(bytes: &[u8], target_nid: u32) -> Option<u64> {
    if bytes.len() < UNICODE_SLBLOCK_HEADER_BYTES
        || bytes[0] != UNICODE_SLBLOCK_TYPE
        || bytes[1] != UNICODE_SLBLOCK_LEAF_LEVEL
        || bytes[4..8] != [0, 0, 0, 0]
    {
        return None;
    }
    let declared_entry_count = u16::from_le_bytes([bytes[2], bytes[3]]) as usize;
    let available_entry_count = bytes
        .len()
        .saturating_sub(UNICODE_SLBLOCK_HEADER_BYTES)
        / UNICODE_SLENTRY_BYTES;
    if declared_entry_count == 0 || declared_entry_count > available_entry_count {
        return None;
    }

    for index in 0..declared_entry_count {
        let start = UNICODE_SLBLOCK_HEADER_BYTES + index * UNICODE_SLENTRY_BYTES;
        let nid = u64::from_le_bytes(bytes[start..start + 8].try_into().ok()?);
        let bid_data = u64::from_le_bytes(bytes[start + 8..start + 16].try_into().ok()?);
        if nid == u64::from(target_nid) && bid_data != 0 {
            return Some(bid_data);
        }
    }
    None
}

fn first_non_empty_string(properties: &PropertyContext, tags: &[u32]) -> Option<String> {
    tags.iter().find_map(|tag| {
        let value = properties.value(*tag)?;
        match value.decoded.as_ref() {
            Some(MapiValue::String(value)) if !value.trim().is_empty() => {
                Some(value.trim().to_string())
            }
            _ => None,
        }
    })
}

fn positive_integer32_property(properties: &PropertyContext, tag: u32) -> Option<i32> {
    match properties.value(tag)?.decoded.as_ref()? {
        MapiValue::Integer32(value) if *value > 0 => Some(*value),
        _ => None,
    }
}

fn non_negative_integer32_property(properties: &PropertyContext, tag: u32) -> Option<i32> {
    match properties.value(tag)?.decoded.as_ref()? {
        MapiValue::Integer32(value) if *value >= 0 => Some(*value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{filename_attachment_record, slblock_data_bid_for_nid};
    use crate::pst::mapi::{
        MapiValue, PR_ATTACH_DATA_BIN, PR_ATTACH_LONG_FILENAME, PR_ATTACH_METHOD, PR_ATTACH_SIZE,
    };
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset};
    use crate::pst::property_context::{PropertyContext, PropertyValue};

    fn property(tag: u32, name: &str, decoded: MapiValue) -> PropertyValue {
        PropertyValue {
            tag,
            name: name.to_string(),
            raw: Vec::new(),
            decoded: Some(decoded),
            status: "selected".to_string(),
        }
    }

    fn payload(block_id: u64, bytes: Vec<u8>) -> PayloadBlock {
        PayloadBlock {
            block_id: BlockId(block_id),
            block_ref: BlockRef {
                block_id: BlockId(block_id),
                offset: ByteOffset(0),
                size: bytes.len() as u64,
            },
            bytes,
            status: "test".to_string(),
        }
    }

    fn slblock(nid: u32, bid_data: u64) -> Vec<u8> {
        let mut bytes = vec![0; 8 + 24];
        bytes[0] = 0x02;
        bytes[1] = 0x00;
        bytes[2..4].copy_from_slice(&1u16.to_le_bytes());
        bytes[8..16].copy_from_slice(&u64::from(nid).to_le_bytes());
        bytes[16..24].copy_from_slice(&bid_data.to_le_bytes());
        bytes
    }

    fn attachment_properties() -> PropertyContext {
        let mut values = HashMap::new();
        values.insert(
            PR_ATTACH_LONG_FILENAME,
            property(
                PR_ATTACH_LONG_FILENAME,
                "attachment_long_filename",
                MapiValue::String("attachment.docx".to_string()),
            ),
        );
        values.insert(
            PR_ATTACH_METHOD,
            property(
                PR_ATTACH_METHOD,
                "attachment_method",
                MapiValue::Integer32(1),
            ),
        );
        values.insert(
            PR_ATTACH_SIZE,
            property(
                PR_ATTACH_SIZE,
                "attachment_size",
                MapiValue::Integer32(15_503),
            ),
        );
        values.insert(
            PR_ATTACH_DATA_BIN,
            PropertyValue {
                tag: PR_ATTACH_DATA_BIN,
                name: "attachment_data".to_string(),
                raw: 0x833fu32.to_le_bytes().to_vec(),
                decoded: Some(MapiValue::Binary(0x833fu32.to_le_bytes().to_vec())),
                status: "selected".to_string(),
            },
        );
        PropertyContext { values }
    }

    #[test]
    fn exposes_validated_attachment_filename_metadata() {
        let record = filename_attachment_record(
            "msg_c6163b9157944cc9",
            0,
            &attachment_properties(),
            &[],
        )
        .expect("validated filename-bearing attachment context");

        assert_eq!(record.filename_original.as_deref(), Some("attachment.docx"));
        assert_eq!(record.filename_safe, "attachment.docx");
        assert_eq!(record.extension.as_deref(), Some("docx"));
        assert_eq!(record.declared_size_bytes, Some(15_503));
        assert_eq!(record.attachment_method, Some(1));
        assert_eq!(record.size_bytes, 0);
        assert_eq!(
            record.extraction_status,
            "attachment_metadata_extracted_payload_reference_unresolved"
        );
    }

    #[test]
    fn resolves_attachment_data_nid_to_loaded_data_bid() {
        let blocks = vec![
            payload(0x6c6, slblock(0x833f, 0x650)),
            payload(0x650, vec![1]),
        ];
        let record = filename_attachment_record(
            "msg_c6163b9157944cc9",
            0,
            &attachment_properties(),
            &blocks,
        )
        .expect("validated filename-bearing attachment context");

        assert_eq!(
            record.extraction_status,
            "attachment_metadata_extracted_payload_subnode_reference_resolved; data_nid=0x0000833f; data_bid=0x650"
        );
    }

    #[test]
    fn rejects_truncated_or_mismatched_slblocks() {
        assert_eq!(
            slblock_data_bid_for_nid(&slblock(0x833f, 0x650), 0x833f),
            Some(0x650)
        );
        assert_eq!(
            slblock_data_bid_for_nid(&slblock(0x833f, 0x650), 0x835f),
            None
        );
        let mut truncated = slblock(0x833f, 0x650);
        truncated[2..4].copy_from_slice(&2u16.to_le_bytes());
        assert_eq!(slblock_data_bid_for_nid(&truncated, 0x833f), None);
    }

    #[test]
    fn rejects_blank_incomplete_or_wrongly_typed_attachment_contexts() {
        let mut blank = HashMap::new();
        blank.insert(
            PR_ATTACH_LONG_FILENAME,
            property(
                PR_ATTACH_LONG_FILENAME,
                "attachment_long_filename",
                MapiValue::String("  ".to_string()),
            ),
        );
        blank.insert(
            PR_ATTACH_METHOD,
            property(
                PR_ATTACH_METHOD,
                "attachment_method",
                MapiValue::Integer32(1),
            ),
        );
        blank.insert(
            PR_ATTACH_SIZE,
            property(PR_ATTACH_SIZE, "attachment_size", MapiValue::Integer32(1)),
        );
        assert!(filename_attachment_record(
            "msg",
            0,
            &PropertyContext { values: blank },
            &[]
        )
        .is_none());

        let mut incomplete = HashMap::new();
        incomplete.insert(
            PR_ATTACH_LONG_FILENAME,
            property(
                PR_ATTACH_LONG_FILENAME,
                "attachment_long_filename",
                MapiValue::String("attachment.docx".to_string()),
            ),
        );
        assert!(filename_attachment_record(
            "msg",
            0,
            &PropertyContext { values: incomplete },
            &[]
        )
        .is_none());

        let mut wrong_type = HashMap::new();
        wrong_type.insert(
            PR_ATTACH_LONG_FILENAME,
            property(
                PR_ATTACH_LONG_FILENAME,
                "attachment_long_filename",
                MapiValue::String("attachment.docx".to_string()),
            ),
        );
        wrong_type.insert(
            PR_ATTACH_METHOD,
            property(
                PR_ATTACH_METHOD,
                "attachment_method",
                MapiValue::String("1".to_string()),
            ),
        );
        wrong_type.insert(
            PR_ATTACH_SIZE,
            property(
                PR_ATTACH_SIZE,
                "attachment_size",
                MapiValue::Integer32(15_503),
            ),
        );
        assert!(filename_attachment_record(
            "msg",
            0,
            &PropertyContext { values: wrong_type },
            &[]
        )
        .is_none());
    }
}
