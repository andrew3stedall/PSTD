use crate::output::{ids, metadata::AttachmentRecord};
use crate::pst::attachments::{
    attachment_metadata_from_properties, attachment_payload,
    unavailable_attachment_record_from_properties, AttachmentMetadata, AttachmentPayload,
    ATTACH_METHOD_EMBEDDED_MESSAGE,
};
use crate::pst::bbt::BbtIndex;
use crate::pst::bth::BthMap;
use crate::pst::data_tree::load_unicode_xblock_payload;
use crate::pst::heap::HeapOnNode;
use crate::pst::limits::ParserLimits;
use crate::pst::mapi::{
    MapiValue, PR_ATTACH_DATA_BIN, PR_ATTACH_DATA_OBJ, PR_ATTACH_FILENAME,
    PR_ATTACH_FILENAME_A, PR_ATTACH_LONG_FILENAME, PR_ATTACH_LONG_FILENAME_A, PR_ATTACH_METHOD,
    PR_ATTACH_SIZE,
};
use crate::pst::payload::PayloadBlock;
use crate::pst::property_context::{PropertyContext, PropertyContextParseReport};
use crate::pst::reader::PstByteReader;
use crate::pst::subnodes::{loaded_subnode_subtree, unicode_subnode_entries};

const HEAP_CLIENT_PROPERTY_CONTEXT: u8 = 0xbc;
const UNICODE_SLBLOCK_TYPE: u8 = 0x02;
const UNICODE_SLBLOCK_LEAF_LEVEL: u8 = 0x00;
const UNICODE_SLBLOCK_HEADER_BYTES: usize = 8;
const UNICODE_SLENTRY_BYTES: usize = 24;
const HNID_TYPE_MASK: u32 = 0x1f;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentPropertyContextReport {
    pub property_context_count: usize,
    pub attachment_record_count: usize,
    pub filename_record_count: usize,
    pub embedded_message_count: usize,
    pub embedded_message_failure_count: usize,
    pub rejected_context_count: usize,
    pub payload_count: usize,
    pub payload_bytes: u64,
    pub payload_failure_count: usize,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct EmbeddedMessageCandidate {
    pub attachment_record: AttachmentRecord,
    pub embedded_message_key: String,
    pub data_nid: u32,
    pub data_bid: u64,
    pub subnode_bid: Option<u64>,
    pub property_report: PropertyContextParseReport,
    pub subnode_payloads: Vec<PayloadBlock>,
}

#[derive(Debug, Clone)]
struct EmbeddedObjectReference {
    data_nid: u32,
    data_bid: u64,
    subnode_bid: Option<u64>,
    subnode_payloads: Vec<PayloadBlock>,
}

/// Extracts only filename-bearing attachment metadata from validated heap Property Contexts.
pub fn attachment_records_from_property_context_subnodes(
    message_key: &str,
    blocks: &[PayloadBlock],
) -> (Vec<AttachmentRecord>, AttachmentPropertyContextReport) {
    let mut records = Vec::new();
    let mut property_context_count = 0usize;
    let mut rejected_context_count = 0usize;
    let mut embedded_message_count = 0usize;

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
        let ordinal = records.len();
        let record = if positive_integer32_property(&report.context, PR_ATTACH_METHOD)
            == Some(ATTACH_METHOD_EMBEDDED_MESSAGE)
        {
            embedded_message_count += 1;
            embedded_attachment_record(
                message_key,
                ordinal,
                &report.context,
                "embedded_message_metadata_discovered",
            )
        } else {
            filename_attachment_record(message_key, ordinal, &report.context, blocks)
        };
        let Some(record) = record else {
            rejected_context_count += 1;
            continue;
        };
        records.push(record);
    }

    let attachment_record_count = records.len();
    let filename_record_count = records
        .iter()
        .filter(|record| record.filename_original.is_some())
        .count();
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
            attachment_record_count,
            filename_record_count,
            embedded_message_count,
            embedded_message_failure_count: 0,
            rejected_context_count,
            payload_count: 0,
            payload_bytes: 0,
            payload_failure_count: 0,
            status: status.to_string(),
        },
    )
}

pub fn attachment_payloads_from_property_context_subnodes(
    message_key: &str,
    blocks: &[PayloadBlock],
    reader: &PstByteReader,
    bbt: &BbtIndex,
    limits: ParserLimits,
) -> (
    Vec<AttachmentPayload>,
    Vec<AttachmentRecord>,
    Vec<EmbeddedMessageCandidate>,
    AttachmentPropertyContextReport,
) {
    let mut payloads = Vec::new();
    let mut records = Vec::new();
    let mut embedded_messages = Vec::new();
    let mut property_context_count = 0usize;
    let mut rejected_context_count = 0usize;
    let mut payload_failure_count = 0usize;
    let mut embedded_message_failure_count = 0usize;

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
        let ordinal = payloads.len() + records.len();

        if positive_integer32_property(&report.context, PR_ATTACH_METHOD)
            == Some(ATTACH_METHOD_EMBEDDED_MESSAGE)
        {
            match embedded_message_candidate(
                message_key,
                ordinal,
                block,
                &report.context,
                blocks,
            ) {
                Some(candidate) => {
                    records.push(candidate.attachment_record.clone());
                    embedded_messages.push(candidate);
                }
                None => {
                    embedded_message_failure_count += 1;
                    if let Some(record) = embedded_attachment_record(
                        message_key,
                        ordinal,
                        &report.context,
                        "embedded_message_reference_unavailable",
                    ) {
                        records.push(record);
                    } else {
                        rejected_context_count += 1;
                    }
                }
            }
            continue;
        }

        let Some(mut record) =
            filename_attachment_record(message_key, ordinal, &report.context, blocks)
        else {
            rejected_context_count += 1;
            continue;
        };

        let Some((data_nid, data_bid)) = resolved_subnode_data_reference(&report.context, blocks)
        else {
            payload_failure_count += 1;
            records.push(record);
            continue;
        };
        let Some(expected_size) = record.declared_size_bytes else {
            payload_failure_count += 1;
            records.push(record);
            continue;
        };

        match load_unicode_xblock_payload(
            reader,
            bbt,
            crate::pst::primitives::BlockId(data_bid),
            expected_size,
            limits,
        ) {
            Ok(tree) => {
                let metadata = AttachmentMetadata {
                    filename_original: record.filename_original.clone(),
                    content_type: record.content_type.clone(),
                    is_inline: record.is_inline,
                    content_id: record.content_id.clone(),
                    attachment_method: record.attachment_method,
                    declared_size_bytes: record.declared_size_bytes,
                };
                let mut payload = attachment_payload(message_key, ordinal, metadata, tree.bytes);
                payload.record.extraction_status = format!(
                    "attachment_payload_extracted_unicode_xblock; data_nid=0x{data_nid:08x}; data_bid=0x{data_bid:x}; child_blocks={}; zip_signature=504b0304",
                    tree.child_bids.len()
                );
                payloads.push(payload);
            }
            Err(reason) => {
                payload_failure_count += 1;
                record.extraction_status = format!(
                    "{}; data_tree_error={}",
                    record.extraction_status,
                    sanitized_status_reason(&reason.to_string())
                );
                records.push(record);
            }
        }
    }

    let attachment_record_count = payloads.len() + records.len();
    let filename_record_count = payloads
        .iter()
        .map(|payload| &payload.record)
        .chain(records.iter())
        .filter(|record| record.filename_original.is_some())
        .count();
    let embedded_message_count = embedded_messages.len();
    let payload_count = payloads.len();
    let payload_bytes = payloads
        .iter()
        .map(|payload| payload.record.size_bytes)
        .sum::<u64>();
    let status = if embedded_message_count > 0 && payload_count > 0 && payload_failure_count == 0 {
        "attachment_property_context_payloads_and_embedded_messages_extracted"
    } else if payload_count > 0 && payload_failure_count == 0 {
        "attachment_property_context_payloads_extracted"
    } else if payload_count > 0 {
        "attachment_property_context_payloads_partially_extracted"
    } else if attachment_record_count > 0 {
        "attachment_property_context_payloads_unavailable"
    } else {
        "attachment_property_context_filename_absent"
    };

    (
        payloads,
        records,
        embedded_messages,
        AttachmentPropertyContextReport {
            property_context_count,
            attachment_record_count,
            filename_record_count,
            embedded_message_count,
            embedded_message_failure_count,
            rejected_context_count,
            payload_count,
            payload_bytes,
            payload_failure_count,
            status: status.to_string(),
        },
    )
}

fn embedded_attachment_record(
    message_key: &str,
    ordinal: usize,
    properties: &PropertyContext,
    status: &str,
) -> Option<AttachmentRecord> {
    if positive_integer32_property(properties, PR_ATTACH_METHOD)
        != Some(ATTACH_METHOD_EMBEDDED_MESSAGE)
    {
        return None;
    }
    let metadata = attachment_metadata_from_properties(properties);
    let mut record =
        unavailable_attachment_record_from_properties(message_key, ordinal, properties, status);
    if record.filename_original.is_none() || record.extension.is_none() {
        record.filename_safe = format!("{}.eml", record.filename_safe);
        record.extension = Some("eml".to_string());
    }
    record.archive_path = format!(
        "attachments/{message_key}/{}_{}",
        record.attachment_key, record.filename_safe
    );
    record.extraction_status = status.to_string();
    if metadata.attachment_method != Some(ATTACH_METHOD_EMBEDDED_MESSAGE) {
        return None;
    }
    Some(record)
}

fn embedded_message_candidate(
    message_key: &str,
    ordinal: usize,
    attachment_block: &PayloadBlock,
    attachment_properties: &PropertyContext,
    blocks: &[PayloadBlock],
) -> Option<EmbeddedMessageCandidate> {
    let object = embedded_object_reference(attachment_block, attachment_properties, blocks)?;
    let property_block = blocks
        .iter()
        .find(|block| block.block_id.0 == object.data_bid)?;
    let heap = HeapOnNode::parse(&property_block.bytes, property_block.block_ref.offset.0).ok()?;
    if heap.header.client_signature != HEAP_CLIENT_PROPERTY_CONTEXT {
        return None;
    }
    let bth = BthMap::parse_property_context_from_heap(
        &heap,
        &property_block.bytes,
        property_block.block_ref.offset.0,
    )
    .ok()?;
    let property_report = PropertyContext::from_bth_with_report(&bth).ok()?;

    let mut attachment_record = embedded_attachment_record(
        message_key,
        ordinal,
        attachment_properties,
        "embedded_message_metadata_extracted",
    )?;
    let embedded_message_key = ids::embedded_message_key(
        message_key,
        &attachment_record.attachment_key,
        object.data_nid,
    );
    attachment_record.embedded_message_key = Some(embedded_message_key.clone());
    attachment_record.extraction_status = format!(
        "embedded_message_metadata_extracted; embedded_message_key={embedded_message_key}; data_nid=0x{:08x}; data_bid=0x{:x}; subnode_bid={}",
        object.data_nid,
        object.data_bid,
        object
            .subnode_bid
            .map(|bid| format!("0x{bid:x}"))
            .unwrap_or_else(|| "none".to_string())
    );

    Some(EmbeddedMessageCandidate {
        attachment_record,
        embedded_message_key,
        data_nid: object.data_nid,
        data_bid: object.data_bid,
        subnode_bid: object.subnode_bid,
        property_report,
        subnode_payloads: object.subnode_payloads,
    })
}

fn embedded_object_reference(
    attachment_block: &PayloadBlock,
    attachment_properties: &PropertyContext,
    blocks: &[PayloadBlock],
) -> Option<EmbeddedObjectReference> {
    let data_nid = attachment_object_nid(attachment_properties)?;
    let mut owners = blocks
        .iter()
        .filter_map(|payload| {
            unicode_subnode_entries(payload).map(|entries| (payload, entries))
        })
        .flat_map(|(payload, entries)| {
            entries
                .into_iter()
                .map(move |entry| (payload, entry))
        })
        .filter(|(_, entry)| entry.data_block_id == attachment_block.block_id)
        .collect::<Vec<_>>();
    if owners.len() != 1 {
        return None;
    }
    let (owner_payload, owner_entry) = owners.pop()?;
    let scope = owner_entry
        .subnode_block_id
        .map(|root| loaded_subnode_subtree(blocks, root))
        .unwrap_or_else(|| vec![owner_payload.clone()]);

    let mut objects = scope
        .iter()
        .filter_map(|payload| unicode_subnode_entries(payload))
        .flatten()
        .filter(|entry| entry.node_id == data_nid)
        .collect::<Vec<_>>();
    if objects.len() != 1 {
        return None;
    }
    let object = objects.pop()?;
    let subnode_payloads = object
        .subnode_block_id
        .map(|root| loaded_subnode_subtree(blocks, root))
        .unwrap_or_default();

    Some(EmbeddedObjectReference {
        data_nid,
        data_bid: object.data_block_id.0,
        subnode_bid: object.subnode_block_id.map(|bid| bid.0),
        subnode_payloads,
    })
}

fn attachment_object_nid(properties: &PropertyContext) -> Option<u32> {
    let value = properties.value(PR_ATTACH_DATA_OBJ)?;
    if value.raw.len() != 4 {
        return None;
    }
    let hnid = u32::from_le_bytes(value.raw.as_slice().try_into().ok()?);
    (hnid != 0 && hnid & HNID_TYPE_MASK != 0).then_some(hnid)
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
    let available_entry_count =
        bytes.len().saturating_sub(UNICODE_SLBLOCK_HEADER_BYTES) / UNICODE_SLENTRY_BYTES;
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

fn sanitized_status_reason(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => ch,
            _ => '_',
        })
        .collect::<String>()
        .trim_matches('_')
        .chars()
        .take(120)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        embedded_attachment_record, embedded_object_reference, filename_attachment_record,
        slblock_data_bid_for_nid,
    };
    use crate::pst::mapi::{
        MapiValue, PR_ATTACH_DATA_BIN, PR_ATTACH_DATA_OBJ, PR_ATTACH_LONG_FILENAME,
        PR_ATTACH_METHOD, PR_ATTACH_SIZE,
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
        slblock_with_sub(nid, bid_data, 0)
    }

    fn slblock_with_sub(nid: u32, bid_data: u64, bid_sub: u64) -> Vec<u8> {
        let mut bytes = vec![0; 8 + 24];
        bytes[0] = 0x02;
        bytes[1] = 0x00;
        bytes[2..4].copy_from_slice(&1u16.to_le_bytes());
        bytes[8..16].copy_from_slice(&u64::from(nid).to_le_bytes());
        bytes[16..24].copy_from_slice(&bid_data.to_le_bytes());
        bytes[24..32].copy_from_slice(&bid_sub.to_le_bytes());
        bytes
    }

    fn embedded_attachment_properties() -> PropertyContext {
        let mut values = HashMap::new();
        values.insert(
            PR_ATTACH_METHOD,
            property(
                PR_ATTACH_METHOD,
                "attachment_method",
                MapiValue::Integer32(5),
            ),
        );
        values.insert(
            PR_ATTACH_DATA_OBJ,
            PropertyValue {
                tag: PR_ATTACH_DATA_OBJ,
                name: "attachment_data_object".to_string(),
                raw: 0x684u32.to_le_bytes().to_vec(),
                decoded: Some(MapiValue::Unknown(0x684u32.to_le_bytes().to_vec())),
                status: "selected".to_string(),
            },
        );
        PropertyContext { values }
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
    fn exposes_method_five_attachment_without_source_filename() {
        let record = embedded_attachment_record(
            "msg_parent",
            0,
            &embedded_attachment_properties(),
            "embedded_message_reference_unavailable",
        )
        .expect("method-five attachment metadata");

        assert_eq!(record.attachment_method, Some(5));
        assert_eq!(record.filename_original, None);
        assert_eq!(record.filename_safe, "attachment_0.eml");
        assert_eq!(record.extension.as_deref(), Some("eml"));
        assert_eq!(record.embedded_message_key, None);
        assert_eq!(
            record.extraction_status,
            "embedded_message_reference_unavailable"
        );
    }

    #[test]
    fn scopes_embedded_object_reference_to_the_owning_attachment() {
        let attachment = payload(0x200, vec![0]);
        let blocks = vec![
            payload(0x100, slblock_with_sub(0x671, 0x200, 0x300)),
            attachment.clone(),
            payload(0x300, slblock_with_sub(0x684, 0x400, 0x500)),
            payload(0x400, vec![1]),
            payload(0x500, slblock_with_sub(0x692, 0x600, 0)),
            payload(0x600, vec![2]),
            payload(0x800, slblock_with_sub(0x684, 0x700, 0)),
            payload(0x700, vec![3]),
        ];

        let object = embedded_object_reference(
            &attachment,
            &embedded_attachment_properties(),
            &blocks,
        )
        .expect("unambiguous embedded object");

        assert_eq!(object.data_nid, 0x684);
        assert_eq!(object.data_bid, 0x400);
        assert_eq!(object.subnode_bid, Some(0x500));
        assert_eq!(
            object
                .subnode_payloads
                .iter()
                .map(|payload| payload.block_id)
                .collect::<Vec<_>>(),
            vec![BlockId(0x500), BlockId(0x600)]
        );
    }

    #[test]
    fn exposes_validated_attachment_filename_metadata() {
        let record =
            filename_attachment_record("msg_c6163b9157944cc9", 0, &attachment_properties(), &[])
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
        assert!(
            filename_attachment_record("msg", 0, &PropertyContext { values: blank }, &[]).is_none()
        );

        let mut incomplete = HashMap::new();
        incomplete.insert(
            PR_ATTACH_LONG_FILENAME,
            property(
                PR_ATTACH_LONG_FILENAME,
                "attachment_long_filename",
                MapiValue::String("attachment.docx".to_string()),
            ),
        );
        assert!(
            filename_attachment_record("msg", 0, &PropertyContext { values: incomplete }, &[])
                .is_none()
        );

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
        assert!(
            filename_attachment_record("msg", 0, &PropertyContext { values: wrong_type }, &[])
                .is_none()
        );
    }
}
