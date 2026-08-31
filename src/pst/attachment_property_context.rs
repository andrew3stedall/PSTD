use crate::output::{ids, metadata::AttachmentRecord};
use crate::pst::attachments::{
    attachment_metadata_from_properties, attachment_payload,
    unavailable_attachment_record_from_properties, AttachmentMetadata, AttachmentPayload,
    ATTACH_METHOD_EMBEDDED_MESSAGE,
};
use crate::pst::bbt::BbtIndex;
use crate::pst::bth::BthMap;
use crate::pst::data_tree::load_attachment_data_payload;
use crate::pst::heap::HeapOnNode;
use crate::pst::limits::ParserLimits;
use crate::pst::mapi::{
    MapiValue, PR_ATTACH_DATA_BIN, PR_ATTACH_DATA_OBJ, PR_ATTACH_FILENAME, PR_ATTACH_FILENAME_A,
    PR_ATTACH_LONG_FILENAME, PR_ATTACH_LONG_FILENAME_A, PR_ATTACH_METHOD, PR_ATTACH_SIZE,
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
const COMPACT_SLENTRY_BYTES: usize = 12;
const HNID_TYPE_MASK: u32 = 0x1f;
const NID_TYPE_NORMAL_MESSAGE: u32 = 0x04;

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

/// Extracts validated attachment metadata from heap Property Contexts, including unnamed rows.
pub fn attachment_records_from_property_context_subnodes(
    message_key: &str,
    blocks: &[PayloadBlock],
) -> (Vec<AttachmentRecord>, AttachmentPropertyContextReport) {
    attachment_records_from_property_context_subnodes_with_fallback_charset(
        message_key,
        blocks,
        None,
    )
}

pub fn attachment_records_from_property_context_subnodes_with_fallback_charset(
    message_key: &str,
    blocks: &[PayloadBlock],
    fallback_charset: Option<&str>,
) -> (Vec<AttachmentRecord>, AttachmentPropertyContextReport) {
    let mut records = Vec::new();
    let mut property_context_count = 0usize;
    let mut rejected_context_count = 0usize;
    let mut embedded_message_count = 0usize;
    let mut embedded_contexts = Vec::new();

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
        let Ok(report) = PropertyContext::from_bth_with_fallback_charset(&bth, fallback_charset)
        else {
            rejected_context_count += 1;
            continue;
        };
        if positive_integer32_property(&report.context, PR_ATTACH_METHOD)
            == Some(ATTACH_METHOD_EMBEDDED_MESSAGE)
        {
            embedded_contexts.push(report.context);
            continue;
        }

        let ordinal = records.len();
        let Some(record) =
            filename_attachment_record(message_key, ordinal, &report.context, blocks)
        else {
            rejected_context_count += 1;
            continue;
        };
        records.push(record);
    }

    for properties in embedded_contexts {
        let ordinal = records.len();
        let Some(record) = embedded_attachment_record(
            message_key,
            ordinal,
            &properties,
            "embedded_message_metadata_discovered",
        ) else {
            rejected_context_count += 1;
            continue;
        };
        embedded_message_count += 1;
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
    attachment_payloads_from_property_context_subnodes_with_fallback_charset(
        message_key,
        blocks,
        reader,
        bbt,
        limits,
        None,
    )
}

pub fn attachment_payloads_from_property_context_subnodes_with_fallback_charset(
    message_key: &str,
    blocks: &[PayloadBlock],
    reader: &PstByteReader,
    bbt: &BbtIndex,
    limits: ParserLimits,
    fallback_charset: Option<&str>,
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
    let mut embedded_contexts: Vec<(&PayloadBlock, PropertyContext)> = Vec::new();

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
        let Ok(report) = PropertyContext::from_bth_with_fallback_charset(&bth, fallback_charset)
        else {
            rejected_context_count += 1;
            continue;
        };
        if positive_integer32_property(&report.context, PR_ATTACH_METHOD)
            == Some(ATTACH_METHOD_EMBEDDED_MESSAGE)
        {
            embedded_contexts.push((block, report.context));
            continue;
        }

        let ordinal = payloads.len() + records.len();
        let Some(mut record) =
            filename_attachment_record(message_key, ordinal, &report.context, blocks)
        else {
            rejected_context_count += 1;
            continue;
        };

        match resolve_attachment_payload(&report.context, blocks, reader, bbt, limits) {
            Ok((bytes, source_status)) => {
                let metadata = AttachmentMetadata {
                    filename_original: record.filename_original.clone(),
                    content_type: record.content_type.clone(),
                    is_inline: record.is_inline,
                    is_hidden: record.is_hidden,
                    content_id: record.content_id.clone(),
                    attachment_method: record.attachment_method,
                    declared_size_bytes: record.declared_size_bytes,
                    rendering_position: record.rendering_position,
                    mime_sequence: record.mime_sequence,
                };
                let mut payload = attachment_payload(message_key, ordinal, metadata, bytes);
                payload.record.extraction_status = source_status;
                payloads.push(payload);
            }
            Err(reason) => {
                payload_failure_count += 1;
                record.extraction_status = format!(
                    "{}; data_tree_error={}",
                    record.extraction_status,
                    sanitized_status_reason(&reason)
                );
                records.push(record);
            }
        }
    }

    for (block, properties) in embedded_contexts {
        let ordinal = payloads.len() + records.len();
        match embedded_message_candidate(
            message_key,
            ordinal,
            block,
            &properties,
            blocks,
            fallback_charset,
        ) {
            Ok(candidate) => {
                records.push(candidate.attachment_record.clone());
                embedded_messages.push(candidate);
            }
            Err(reason) => {
                embedded_message_failure_count += 1;
                let status = format!("embedded_message_reference_unavailable; {reason}");
                if let Some(record) =
                    embedded_attachment_record(message_key, ordinal, &properties, &status)
                {
                    records.push(record);
                } else {
                    rejected_context_count += 1;
                }
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
    fallback_charset: Option<&str>,
) -> Result<EmbeddedMessageCandidate, String> {
    let object = embedded_object_reference(attachment_block, attachment_properties, blocks)?;
    let property_matches = blocks
        .iter()
        .filter(|block| block.block_id.0 == object.data_bid)
        .collect::<Vec<_>>();
    let normalized_property_match_count = blocks
        .iter()
        .filter(|block| normalized_bid(block.block_id.0) == normalized_bid(object.data_bid))
        .count();
    if property_matches.len() != 1 {
        return Err(format!(
            "stage=property_block; data_nid=0x{:08x}; data_bid=0x{:x}; property_matches={}; normalized_property_matches={normalized_property_match_count}",
            object.data_nid,
            object.data_bid,
            property_matches.len()
        ));
    }
    let property_block = property_matches[0];
    let heap = HeapOnNode::parse(&property_block.bytes, property_block.block_ref.offset.0)
        .map_err(|_| {
            format!(
                "stage=property_heap; data_nid=0x{:08x}; data_bid=0x{:x}",
                object.data_nid, object.data_bid
            )
        })?;
    if heap.header.client_signature != HEAP_CLIENT_PROPERTY_CONTEXT {
        return Err(format!(
            "stage=property_heap_signature; data_nid=0x{:08x}; data_bid=0x{:x}; signature=0x{:02x}",
            object.data_nid, object.data_bid, heap.header.client_signature
        ));
    }
    let bth = BthMap::parse_property_context_from_heap(
        &heap,
        &property_block.bytes,
        property_block.block_ref.offset.0,
    )
    .map_err(|_| {
        format!(
            "stage=property_bth; data_nid=0x{:08x}; data_bid=0x{:x}",
            object.data_nid, object.data_bid
        )
    })?;
    let property_report = PropertyContext::from_bth_with_fallback_charset(&bth, fallback_charset)
        .map_err(|_| {
        format!(
            "stage=property_context; data_nid=0x{:08x}; data_bid=0x{:x}",
            object.data_nid, object.data_bid
        )
    })?;

    let mut attachment_record = embedded_attachment_record(
        message_key,
        ordinal,
        attachment_properties,
        "embedded_message_metadata_extracted",
    )
    .ok_or_else(|| {
        format!(
            "stage=attachment_record; data_nid=0x{:08x}; data_bid=0x{:x}",
            object.data_nid, object.data_bid
        )
    })?;
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

    Ok(EmbeddedMessageCandidate {
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
) -> Result<EmbeddedObjectReference, String> {
    let data_nid = attachment_object_nid(attachment_block, attachment_properties)?;
    let mut objects = blocks
        .iter()
        .filter_map(|payload| unicode_subnode_entries(payload).map(|entries| (payload, entries)))
        .flat_map(|(payload, entries)| entries.into_iter().map(move |entry| (payload, entry)))
        .filter(|(_, entry)| entry.node_id == data_nid)
        .collect::<Vec<_>>();
    if objects.len() != 1 {
        return Err(format!(
            "stage=object_entry; attachment_bid=0x{:x}; data_nid=0x{data_nid:08x}; object_matches={}",
            attachment_block.block_id.0,
            objects.len()
        ));
    }
    let (_object_owner, object) = objects
        .pop()
        .expect("one embedded object reference was validated");
    let subnode_payloads = object
        .subnode_block_id
        .map(|root| loaded_subnode_subtree(blocks, root))
        .unwrap_or_default();

    Ok(EmbeddedObjectReference {
        data_nid,
        data_bid: object.data_block_id.0,
        subnode_bid: object.subnode_block_id.map(|bid| bid.0),
        subnode_payloads,
    })
}

fn normalized_bid(value: u64) -> u64 {
    value & !0x03
}

fn attachment_object_nid(
    attachment_block: &PayloadBlock,
    properties: &PropertyContext,
) -> Result<u32, String> {
    let Some(value) = properties.value(PR_ATTACH_DATA_OBJ) else {
        return Err(format!(
            "stage=data_nid; reason=property_missing; property_family={}",
            attachment_data_object_family_summary(properties)
        ));
    };
    if value.raw.len() != 4 {
        return Err(format!(
            "stage=data_nid; reason=invalid_length; tag=0x{:08x}; raw_len={}; raw_prefix={}",
            value.tag,
            value.raw.len(),
            bounded_hex_prefix(&value.raw)
        ));
    }
    let hnid = u32::from_le_bytes(
        value
            .raw
            .as_slice()
            .try_into()
            .expect("four-byte object HNID was validated"),
    );
    if hnid == 0 || hnid & HNID_TYPE_MASK != 0 {
        return Err(format!(
            "stage=data_nid; reason=object_hnid_not_heap_id; tag=0x{:08x}; hnid=0x{hnid:08x}",
            value.tag
        ));
    }

    let heap = HeapOnNode::parse(&attachment_block.bytes, attachment_block.block_ref.offset.0)
        .map_err(|_| {
            format!(
                "stage=data_nid; reason=indirect_heap_invalid; tag=0x{:08x}; hnid=0x{hnid:08x}",
                value.tag
            )
        })?;
    let allocation = heap
        .allocation_by_hid(
            &attachment_block.bytes,
            hnid,
            attachment_block.block_ref.offset.0,
        )
        .map_err(|_| {
            format!(
                "stage=data_nid; reason=indirect_allocation_missing; tag=0x{:08x}; hnid=0x{hnid:08x}",
                value.tag
            )
        })?;
    embedded_message_nid_from_object_allocation(allocation).map_err(|reason| {
        format!(
            "stage=data_nid; {reason}; tag=0x{:08x}; hnid=0x{hnid:08x}",
            value.tag
        )
    })
}

fn embedded_message_nid_from_object_allocation(allocation: &[u8]) -> Result<u32, String> {
    if allocation.len() != 8 {
        return Err(format!(
            "reason=object_allocation_size; allocation_len={}; allocation_prefix={}",
            allocation.len(),
            bounded_hex_prefix(allocation)
        ));
    }
    let nid = u32::from_le_bytes(
        allocation[0..4]
            .try_into()
            .expect("four-byte object NID slice"),
    );
    let object_size = u32::from_le_bytes(
        allocation[4..8]
            .try_into()
            .expect("four-byte object size slice"),
    );
    if nid == 0 || nid & HNID_TYPE_MASK != NID_TYPE_NORMAL_MESSAGE {
        return Err(format!(
            "reason=object_nid_type; nid=0x{nid:08x}; object_size={object_size}"
        ));
    }
    if object_size == 0 {
        return Err(format!("reason=object_size_zero; nid=0x{nid:08x}"));
    }
    Ok(nid)
}

fn attachment_data_object_family_summary(properties: &PropertyContext) -> String {
    let mut entries = properties
        .values
        .values()
        .filter(|value| value.tag >> 16 == PR_ATTACH_DATA_OBJ >> 16)
        .map(|value| {
            format!(
                "0x{:08x}:len{}:{}",
                value.tag,
                value.raw.len(),
                bounded_hex_prefix(&value.raw)
            )
        })
        .collect::<Vec<_>>();
    entries.sort();
    if entries.is_empty() {
        "none".to_string()
    } else {
        entries.join(",")
    }
}

fn bounded_hex_prefix(value: &[u8]) -> String {
    let prefix = value
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("");
    if value.len() > 8 {
        format!("{prefix}+{}bytes", value.len() - 8)
    } else if prefix.is_empty() {
        "empty".to_string()
    } else {
        prefix
    }
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
    );
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
    record.filename_original = filename.clone();
    record.filename_safe = crate::pst::attachments::safe_filename(filename.as_deref(), ordinal);
    record.extension = crate::pst::attachments::file_extension(&record.filename_safe);
    record.archive_path = format!(
        "attachments/{message_key}/{}_{}",
        record.attachment_key, record.filename_safe
    );
    Some(record)
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

fn resolve_attachment_payload(
    properties: &PropertyContext,
    blocks: &[PayloadBlock],
    reader: &PstByteReader,
    bbt: &BbtIndex,
    limits: ParserLimits,
) -> Result<(Vec<u8>, String), String> {
    if let Some((data_nid, data_bid)) = resolved_subnode_data_reference(properties, blocks) {
        let tree = load_attachment_data_payload(
            reader,
            bbt,
            crate::pst::primitives::BlockId(data_bid),
            non_negative_integer32_property(properties, PR_ATTACH_SIZE).map(|size| size as u64),
            limits,
        )
        .map_err(|reason| reason.to_string())?;
        return Ok((
            tree.bytes,
            format!(
                "attachment_payload_extracted_data_tree; data_nid=0x{data_nid:08x}; data_bid=0x{data_bid:x}; child_blocks={}; {}",
                tree.child_bids.len(),
                sanitized_status_reason(&tree.status)
            ),
        ));
    }

    if let Some(bytes) = inline_object_heap_bytes(properties, blocks) {
        return Ok((
            bytes,
            "attachment_payload_extracted_inline_object_heap".to_string(),
        ));
    }

    if let Some(bytes) = inline_attachment_bytes(properties) {
        return Ok((
            bytes,
            "attachment_payload_extracted_inline_property".to_string(),
        ));
    }

    Err("attachment_payload_reference_unresolved".to_string())
}

fn inline_object_heap_bytes(
    properties: &PropertyContext,
    blocks: &[PayloadBlock],
) -> Option<Vec<u8>> {
    let value = properties.value(PR_ATTACH_DATA_OBJ)?;
    if value.raw.len() != 4 {
        return None;
    }
    let hnid = u32::from_le_bytes(value.raw.as_slice().try_into().ok()?);
    if hnid == 0 || hnid & HNID_TYPE_MASK != 0 {
        return None;
    }

    let object_tag = PR_ATTACH_DATA_OBJ.to_le_bytes();
    let mut matches = Vec::new();
    for block in blocks {
        let Ok(heap) = HeapOnNode::parse(&block.bytes, block.block_ref.offset.0) else {
            continue;
        };
        if heap.header.client_signature != HEAP_CLIENT_PROPERTY_CONTEXT {
            continue;
        }
        let Ok(bth) =
            BthMap::parse_property_context_from_heap(&heap, &block.bytes, block.block_ref.offset.0)
        else {
            continue;
        };
        if bth.lookup(&object_tag) != Some(value.raw.as_slice()) {
            continue;
        }
        let Some(bytes) =
            heap.try_allocation_by_hnid(&block.bytes, hnid, block.block_ref.offset.0)
        else {
            continue;
        };
        matches.push(bytes.to_vec());
    }

    (matches.len() == 1).then(|| matches.remove(0))
}

fn inline_attachment_bytes(properties: &PropertyContext) -> Option<Vec<u8>> {
    for tag in [PR_ATTACH_DATA_BIN, PR_ATTACH_DATA_OBJ] {
        let Some(value) = properties.value(tag) else {
            continue;
        };
        if value.raw.len() != 4 || !indirect_attachment_method(properties) {
            return Some(value.raw.clone());
        }
    }
    // Four-byte variable-property values for reference-based methods are HNID
    // references in the PST attachment layout. Without a validated SLBLOCK
    // target they remain unresolved rather than being emitted as fake bytes.
    None
}

fn indirect_attachment_method(properties: &PropertyContext) -> bool {
    matches!(
        positive_integer32_property(properties, PR_ATTACH_METHOD),
        Some(2..=6)
    )
}

fn resolved_subnode_data_reference(
    properties: &PropertyContext,
    blocks: &[PayloadBlock],
) -> Option<(u32, u64)> {
    let data_nid = attachment_data_nid(properties)?;
    let references = blocks
        .iter()
        .flat_map(|block| slblock_data_bids_for_nid(&block.bytes, data_nid))
        .collect::<Vec<_>>();
    if references.len() != 1 {
        return None;
    }

    let data_bid = references[0];
    (blocks
        .iter()
        .filter(|block| block.block_id.0 == data_bid)
        .count()
        == 1)
        .then_some((data_nid, data_bid))
}

fn attachment_data_nid(properties: &PropertyContext) -> Option<u32> {
    for tag in [PR_ATTACH_DATA_BIN, PR_ATTACH_DATA_OBJ] {
        let Some(value) = properties.value(tag) else {
            continue;
        };
        if value.raw.len() != 4 {
            continue;
        }
        let hnid = u32::from_le_bytes(value.raw.as_slice().try_into().ok()?);
        if hnid & HNID_TYPE_MASK != 0 {
            return Some(hnid);
        }
    }
    None
}

fn slblock_data_bids_for_nid(bytes: &[u8], target_nid: u32) -> Vec<u64> {
    if bytes.len() < UNICODE_SLBLOCK_HEADER_BYTES
        || bytes[0] != UNICODE_SLBLOCK_TYPE
        || bytes[1] != UNICODE_SLBLOCK_LEAF_LEVEL
        || bytes[4..8] != [0, 0, 0, 0]
    {
        return Vec::new();
    }
    let declared_entry_count = u16::from_le_bytes([bytes[2], bytes[3]]) as usize;
    if declared_entry_count == 0 {
        return Vec::new();
    }

    // Unicode uses 8-byte NIDs/BIDs; compact legacy tables use 4-byte fields.
    // Try the wide representation first so a padded Unicode entry cannot be
    // mistaken for a compact entry.
    for entry_width in [UNICODE_SLENTRY_BYTES, COMPACT_SLENTRY_BYTES] {
        let available_entry_count =
            bytes.len().saturating_sub(UNICODE_SLBLOCK_HEADER_BYTES) / entry_width;
        if declared_entry_count > available_entry_count {
            continue;
        }

        let mut bids = Vec::new();
        for index in 0..declared_entry_count {
            let start = UNICODE_SLBLOCK_HEADER_BYTES + index * entry_width;
            let (nid, bid_data) = if entry_width == UNICODE_SLENTRY_BYTES {
                (
                    u64::from_le_bytes(bytes[start..start + 8].try_into().unwrap()),
                    u64::from_le_bytes(bytes[start + 8..start + 16].try_into().unwrap()),
                )
            } else {
                (
                    u32::from_le_bytes(bytes[start..start + 4].try_into().unwrap()) as u64,
                    u32::from_le_bytes(bytes[start + 4..start + 8].try_into().unwrap()) as u64,
                )
            };
            if nid == u64::from(target_nid) && bid_data != 0 {
                bids.push(bid_data);
            }
        }
        return bids;
    }
    Vec::new()
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

    use std::fs;

    use tempfile::NamedTempFile;

    use super::{
        embedded_attachment_record, embedded_message_nid_from_object_allocation,
        embedded_object_reference, filename_attachment_record, resolve_attachment_payload,
        slblock_data_bids_for_nid, COMPACT_SLENTRY_BYTES, UNICODE_SLBLOCK_LEAF_LEVEL,
        UNICODE_SLBLOCK_TYPE,
    };
    use crate::pst::bbt::{BbtEntry, BbtIndex};
    use crate::pst::limits::ParserLimits;
    use crate::pst::mapi::{
        MapiValue, PR_ATTACH_DATA_BIN, PR_ATTACH_DATA_OBJ, PR_ATTACH_LONG_FILENAME,
        PR_ATTACH_METHOD, PR_ATTACH_SIZE,
    };
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset};
    use crate::pst::property_context::{PropertyContext, PropertyValue};
    use crate::pst::reader::PstByteReader;

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

    fn compact_slblock(nid: u32, bid_data: u32) -> Vec<u8> {
        let mut bytes = vec![0; 8 + COMPACT_SLENTRY_BYTES];
        bytes[0] = UNICODE_SLBLOCK_TYPE;
        bytes[1] = UNICODE_SLBLOCK_LEAF_LEVEL;
        bytes[2..4].copy_from_slice(&1u16.to_le_bytes());
        bytes[8..12].copy_from_slice(&nid.to_le_bytes());
        bytes[12..16].copy_from_slice(&bid_data.to_le_bytes());
        bytes
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
                raw: 0x80u32.to_le_bytes().to_vec(),
                decoded: Some(MapiValue::Unknown(0x80u32.to_le_bytes().to_vec())),
                status: "selected".to_string(),
            },
        );
        PropertyContext { values }
    }

    fn embedded_attachment_payload(block_id: u64, object_nid: u32) -> PayloadBlock {
        let mut bytes = vec![0; 64];
        bytes[0..2].copy_from_slice(&48u16.to_le_bytes());
        bytes[2] = 0xec;
        bytes[3] = 0xbc;
        bytes[32..36].copy_from_slice(&object_nid.to_le_bytes());
        bytes[36..40].copy_from_slice(&1u32.to_le_bytes());
        bytes[48..50].copy_from_slice(&4u16.to_le_bytes());
        bytes[52..54].copy_from_slice(&16u16.to_le_bytes());
        bytes[54..56].copy_from_slice(&16u16.to_le_bytes());
        bytes[56..58].copy_from_slice(&16u16.to_le_bytes());
        bytes[58..60].copy_from_slice(&32u16.to_le_bytes());
        bytes[60..62].copy_from_slice(&40u16.to_le_bytes());
        payload(block_id, bytes)
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
    fn resolves_unique_embedded_object_and_isolates_its_subtree() {
        let attachment = embedded_attachment_payload(0x200, 0x684);
        let blocks = vec![
            payload(0x100, slblock_with_sub(0x671, 0x200, 0x300)),
            attachment.clone(),
            payload(0x300, slblock_with_sub(0x684, 0x400, 0x500)),
            payload(0x400, vec![1]),
            payload(0x500, slblock_with_sub(0x692, 0x600, 0)),
            payload(0x600, vec![2]),
            payload(0x800, slblock_with_sub(0x6a4, 0x700, 0)),
            payload(0x700, vec![3]),
        ];

        let object =
            embedded_object_reference(&attachment, &embedded_attachment_properties(), &blocks)
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
    fn rejects_duplicate_embedded_object_nids_in_the_message_scope() {
        let attachment = embedded_attachment_payload(0x200, 0x684);
        let blocks = vec![
            payload(0x300, slblock_with_sub(0x684, 0x400, 0)),
            payload(0x400, vec![1]),
            payload(0x800, slblock_with_sub(0x684, 0x700, 0)),
            payload(0x700, vec![2]),
            attachment.clone(),
        ];

        let error =
            embedded_object_reference(&attachment, &embedded_attachment_properties(), &blocks)
                .expect_err("duplicate object NIDs must fail closed");

        assert!(error.contains("object_matches=2"));
    }

    #[test]
    fn parses_only_exact_normal_message_object_allocations() {
        let mut valid = 0x0020_0104u32.to_le_bytes().to_vec();
        valid.extend_from_slice(&0x50fcu32.to_le_bytes());
        assert_eq!(
            embedded_message_nid_from_object_allocation(&valid),
            Ok(0x0020_0104)
        );

        assert!(embedded_message_nid_from_object_allocation(&valid[..7]).is_err());
        let mut wrong_type = 0x0020_0105u32.to_le_bytes().to_vec();
        wrong_type.extend_from_slice(&1u32.to_le_bytes());
        assert!(embedded_message_nid_from_object_allocation(&wrong_type).is_err());
        let mut zero_size = 0x0020_0104u32.to_le_bytes().to_vec();
        zero_size.extend_from_slice(&0u32.to_le_bytes());
        assert!(embedded_message_nid_from_object_allocation(&zero_size).is_err());
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
    fn resolves_compact_four_byte_slblocks_and_rejects_truncation() {
        assert_eq!(
            slblock_data_bids_for_nid(&compact_slblock(0x833f, 0x650), 0x833f),
            vec![0x650]
        );

        let mut truncated = compact_slblock(0x833f, 0x650);
        truncated[2..4].copy_from_slice(&2u16.to_le_bytes());
        assert_eq!(
            slblock_data_bids_for_nid(&truncated, 0x833f),
            Vec::<u64>::new()
        );
    }

    #[test]
    fn resolves_method_six_ole_payload_through_wide_and_compact_references() {
        let ole_bytes = b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1compound-file".to_vec();

        for slblock_bytes in [slblock(0x833f, 0x632), compact_slblock(0x833f, 0x632)] {
            let first = ole_bytes[..8].to_vec();
            let second = ole_bytes[8..].to_vec();
            let root = xblock(&[0x640, 0x644], ole_bytes.len() as u32);
            let blocks = vec![
                payload(0x6c6, slblock_bytes.clone()),
                payload(0x632, root.clone()),
                payload(0x640, first.clone()),
                payload(0x644, second.clone()),
            ];
            let (file, bbt) = fixture(&[
                (0x6c6, slblock_bytes),
                (0x632, root),
                (0x640, first),
                (0x644, second),
            ]);
            let reader = PstByteReader::open(file.path()).unwrap();
            let properties = method_six_properties(ole_bytes.len(), 0x833f);

            let (bytes, status) = resolve_attachment_payload(
                &properties,
                &blocks,
                &reader,
                &bbt,
                ParserLimits::default(),
            )
            .expect("valid method-six reference");

            assert_eq!(bytes, ole_bytes);
            assert!(status.contains("data_nid=0x0000833f"));
            assert!(status.contains("data_bid=0x632"));
        }
    }

    #[test]
    fn rejects_missing_and_ambiguous_method_six_references() {
        let properties = method_six_properties(4, 0x833f);
        let missing_blocks = vec![payload(0x6c6, slblock(0x833f, 0x632))];
        let (file, bbt) = fixture(&[(0x6c6, slblock(0x833f, 0x632))]);
        let reader = PstByteReader::open(file.path()).unwrap();
        assert_eq!(
            resolve_attachment_payload(
                &properties,
                &missing_blocks,
                &reader,
                &bbt,
                ParserLimits::default(),
            )
            .unwrap_err(),
            "attachment_payload_reference_unresolved"
        );

        let first = slblock(0x833f, 0x632);
        let second = slblock(0x833f, 0x634);
        let ambiguous_blocks = vec![
            payload(0x6c6, first.clone()),
            payload(0x6c7, second.clone()),
            payload(0x632, b"first".to_vec()),
            payload(0x634, b"second".to_vec()),
        ];
        let (file, bbt) = fixture(&[
            (0x6c6, first),
            (0x6c7, second),
            (0x632, b"first".to_vec()),
            (0x634, b"second".to_vec()),
        ]);
        let reader = PstByteReader::open(file.path()).unwrap();
        assert_eq!(
            resolve_attachment_payload(
                &properties,
                &ambiguous_blocks,
                &reader,
                &bbt,
                ParserLimits::default(),
            )
            .unwrap_err(),
            "attachment_payload_reference_unresolved"
        );
    }

    #[test]
    fn rejects_truncated_or_mismatched_unicode_slblocks() {
        assert_eq!(
            slblock_data_bids_for_nid(&slblock(0x833f, 0x650), 0x833f),
            vec![0x650]
        );
        assert_eq!(
            slblock_data_bids_for_nid(&slblock(0x833f, 0x650), 0x835f),
            Vec::<u64>::new()
        );
        let mut truncated = slblock(0x833f, 0x650);
        truncated[2..4].copy_from_slice(&2u16.to_le_bytes());
        assert_eq!(
            slblock_data_bids_for_nid(&truncated, 0x833f),
            Vec::<u64>::new()
        );
    }

    fn method_six_properties(size: usize, data_nid: u32) -> PropertyContext {
        let mut values = HashMap::new();
        values.insert(
            PR_ATTACH_LONG_FILENAME,
            property(
                PR_ATTACH_LONG_FILENAME,
                "attachment_long_filename",
                MapiValue::String("ole-object.cfb".to_string()),
            ),
        );
        values.insert(
            PR_ATTACH_METHOD,
            property(
                PR_ATTACH_METHOD,
                "attachment_method",
                MapiValue::Integer32(6),
            ),
        );
        values.insert(
            PR_ATTACH_SIZE,
            property(
                PR_ATTACH_SIZE,
                "attachment_size",
                MapiValue::Integer32(size as i32),
            ),
        );
        values.insert(
            PR_ATTACH_DATA_OBJ,
            PropertyValue {
                tag: PR_ATTACH_DATA_OBJ,
                name: "attachment_data_object".to_string(),
                raw: data_nid.to_le_bytes().to_vec(),
                decoded: Some(MapiValue::Unknown(data_nid.to_le_bytes().to_vec())),
                status: "selected".to_string(),
            },
        );
        PropertyContext { values }
    }

    fn xblock(child_bids: &[u64], total: u32) -> Vec<u8> {
        let mut bytes = vec![0; 8 + child_bids.len() * 8];
        bytes[0..2].copy_from_slice(&0x0101u16.to_le_bytes());
        bytes[2..4].copy_from_slice(&(child_bids.len() as u16).to_le_bytes());
        bytes[4..8].copy_from_slice(&total.to_le_bytes());
        for (index, bid) in child_bids.iter().enumerate() {
            let start = 8 + index * 8;
            bytes[start..start + 8].copy_from_slice(&bid.to_le_bytes());
        }
        bytes
    }

    fn fixture(blocks: &[(u64, Vec<u8>)]) -> (NamedTempFile, BbtIndex) {
        let file = NamedTempFile::new().unwrap();
        let mut file_bytes = vec![0; 1024];
        let mut entries = Vec::new();
        let mut offset = 600usize;
        for (bid, bytes) in blocks {
            if file_bytes.len() < offset + bytes.len() {
                file_bytes.resize(offset + bytes.len(), 0);
            }
            file_bytes[offset..offset + bytes.len()].copy_from_slice(bytes);
            entries.push(BbtEntry {
                block_id: BlockId(*bid),
                offset: ByteOffset(offset as u64),
                size: bytes.len() as u64,
            });
            offset += bytes.len() + 32;
        }
        fs::write(file.path(), file_bytes).unwrap();
        (
            file,
            BbtIndex {
                root: None,
                entries,
                parsed_pages: 0,
                discovered_child_pages: 0,
                traversal_error_count: 0,
                duplicate_entry_count: 0,
                truncated_entry_count: 0,
                status: "test".to_string(),
            },
        )
    }

    #[test]
    fn preserves_unnamed_but_rejects_incomplete_or_wrongly_typed_attachment_contexts() {
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
        let record = filename_attachment_record("msg", 0, &PropertyContext { values: blank }, &[])
            .expect("method and size validate an unnamed attachment");
        assert_eq!(record.filename_original, None);
        assert_eq!(record.filename_safe, "attachment_0");
        assert_eq!(record.extension, None);
        assert_eq!(record.size_bytes, 0);

        let mut missing_filename = HashMap::new();
        missing_filename.insert(
            PR_ATTACH_METHOD,
            property(
                PR_ATTACH_METHOD,
                "attachment_method",
                MapiValue::Integer32(1),
            ),
        );
        missing_filename.insert(
            PR_ATTACH_SIZE,
            property(PR_ATTACH_SIZE, "attachment_size", MapiValue::Integer32(3)),
        );
        let record = filename_attachment_record(
            "msg",
            1,
            &PropertyContext {
                values: missing_filename,
            },
            &[],
        )
        .expect("missing filename must use the deterministic fallback");
        assert_eq!(record.filename_original, None);
        assert_eq!(record.filename_safe, "attachment_1");

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
