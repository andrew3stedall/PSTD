use std::collections::{HashSet, VecDeque};

use crate::pst::bbt::BbtIndex;
use crate::pst::heap::HeapOnNode;
use crate::pst::limits::ParserLimits;
use crate::pst::nbt::{NbtEntry, NbtIndex};
use crate::pst::payload::{load_payload_block, PayloadBlock};
use crate::pst::primitives::{BlockId, NodeId};
use crate::pst::reader::PstByteReader;
use crate::pst::table_context::TableContext;

const SYNTHETIC_CHILD_REF_MAGIC: &[u8; 4] = b"SNOD";
const TABLE_PAYLOAD_PREFIX_BYTES: usize = 32;
const SLBLOCK_TYPE: u8 = 0x02;
const SLBLOCK_LEAF_LEVEL: u8 = 0x00;
const UNICODE_SLBLOCK_HEADER_BYTES: usize = 8;
const UNICODE_SLENTRY_BYTES: usize = 24;
const HEAP_SIGNATURE: u8 = 0xec;
const HEAP_CLIENT_TABLE_CONTEXT: u8 = 0x7c;
const HEAP_CLIENT_BTH: u8 = 0xb5;
const HEAP_CLIENT_PROPERTY_CONTEXT: u8 = 0xbc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubnodeReference {
    pub node_id: NodeId,
    pub subnode_block_id: BlockId,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubnodeReferenceReport {
    pub node_count: usize,
    pub subnode_reference_count: usize,
    pub references: Vec<SubnodeReference>,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubnodeDecodePlan {
    pub root_node_id: NodeId,
    pub root_subnode_block_id: BlockId,
    pub requested_depth: usize,
    pub max_depth: usize,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubnodeDecodeReport {
    pub root_node_id: NodeId,
    pub root_subnode_block_id: BlockId,
    pub requested_depth: usize,
    pub max_depth: usize,
    pub decoded_block_count: usize,
    pub failed_block_count: usize,
    pub decoded_bytes: u64,
    pub recursive_child_reference_count: usize,
    pub recursive_child_decode_count: usize,
    pub layout_statuses: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubnodeBlockLayout {
    pub block_id: BlockId,
    pub offset: u64,
    pub size: u64,
    pub byte_len: usize,
    pub layout_kind: String,
    pub child_block_ids: Vec<BlockId>,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubnodeLayoutReport {
    pub block_count: usize,
    pub table_layout_count: usize,
    pub child_reference_layout_count: usize,
    pub unsupported_layout_count: usize,
    pub child_reference_count: usize,
    pub layouts: Vec<SubnodeBlockLayout>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct LoadedSubnodeBlocks {
    pub payloads: Vec<PayloadBlock>,
    pub report: SubnodeDecodeReport,
    pub layout_report: SubnodeLayoutReport,
}

pub fn subnode_references_from_index(index: &NbtIndex) -> SubnodeReferenceReport {
    subnode_references_from_entries(&index.entries)
}

pub fn subnode_references_from_entries(entries: &[NbtEntry]) -> SubnodeReferenceReport {
    let references = entries
        .iter()
        .filter_map(|entry| {
            entry
                .subnode_block_id
                .map(|subnode_block_id| SubnodeReference {
                    node_id: entry.node_id,
                    subnode_block_id,
                    status: "subnode_reference_discovered".to_string(),
                })
        })
        .collect::<Vec<_>>();
    let status = if references.is_empty() {
        "no_subnode_references"
    } else {
        "subnode_references_discovered"
    };
    SubnodeReferenceReport {
        node_count: entries.len(),
        subnode_reference_count: references.len(),
        references,
        status: status.to_string(),
    }
}

pub fn subnode_decode_plans(
    references: &[SubnodeReference],
    limits: ParserLimits,
) -> Vec<SubnodeDecodePlan> {
    references
        .iter()
        .map(|reference| subnode_decode_plan(reference, 1, limits))
        .collect()
}

pub fn subnode_decode_plan(
    reference: &SubnodeReference,
    requested_depth: usize,
    limits: ParserLimits,
) -> SubnodeDecodePlan {
    let status = if requested_depth > limits.max_subnode_depth {
        "subnode_depth_limit_exceeded"
    } else {
        "subnode_decode_planned"
    };
    SubnodeDecodePlan {
        root_node_id: reference.node_id,
        root_subnode_block_id: reference.subnode_block_id,
        requested_depth,
        max_depth: limits.max_subnode_depth,
        status: status.to_string(),
    }
}

pub fn load_recursive_subnode_blocks(
    reader: &PstByteReader,
    bbt: &BbtIndex,
    reference: &SubnodeReference,
    requested_depth: usize,
    limits: ParserLimits,
) -> LoadedSubnodeBlocks {
    if requested_depth > limits.max_subnode_depth {
        return LoadedSubnodeBlocks {
            payloads: Vec::new(),
            report: SubnodeDecodeReport {
                root_node_id: reference.node_id,
                root_subnode_block_id: reference.subnode_block_id,
                requested_depth,
                max_depth: limits.max_subnode_depth,
                decoded_block_count: 0,
                failed_block_count: 0,
                decoded_bytes: 0,
                recursive_child_reference_count: 0,
                recursive_child_decode_count: 0,
                layout_statuses: Vec::new(),
                status: "subnode_depth_limit_exceeded".to_string(),
            },
            layout_report: empty_layout_report(),
        };
    }

    let mut payloads = Vec::new();
    let mut failed_block_count = 0usize;
    let mut skipped_child_count = 0usize;
    let mut queue = VecDeque::from([(reference.subnode_block_id, requested_depth)]);
    let mut seen = HashSet::new();

    while let Some((block_id, depth)) = queue.pop_front() {
        if !seen.insert(block_id) {
            continue;
        }
        if depth > limits.max_subnode_depth {
            skipped_child_count += 1;
            continue;
        }
        match load_payload_block(reader, bbt, block_id, limits) {
            Ok(payload) => {
                let layout = classify_subnode_block_layout(&payload);
                if depth < limits.max_subnode_depth {
                    for child_block_id in &layout.child_block_ids {
                        if !seen.contains(child_block_id) {
                            queue.push_back((*child_block_id, depth + 1));
                        }
                    }
                } else {
                    skipped_child_count += layout.child_block_ids.len();
                }
                payloads.push(payload);
            }
            Err(_) => failed_block_count += 1,
        }
    }

    let layout_report = classify_subnode_payloads(&payloads);
    let decoded_bytes = payloads
        .iter()
        .map(|payload| payload.bytes.len() as u64)
        .sum();
    let recursive_child_decode_count = payloads.len().saturating_sub(1);
    let status = if payloads.is_empty() {
        "subnode_recursive_blocks_unavailable"
    } else if skipped_child_count > 0 {
        "subnode_recursive_depth_limit_reached"
    } else if failed_block_count > 0 {
        "subnode_recursive_blocks_partially_loaded"
    } else if recursive_child_decode_count > 0 {
        "subnode_recursive_blocks_loaded"
    } else {
        "subnode_root_block_loaded"
    };
    LoadedSubnodeBlocks {
        report: SubnodeDecodeReport {
            root_node_id: reference.node_id,
            root_subnode_block_id: reference.subnode_block_id,
            requested_depth,
            max_depth: limits.max_subnode_depth,
            decoded_block_count: payloads.len(),
            failed_block_count,
            decoded_bytes,
            recursive_child_reference_count: layout_report.child_reference_count,
            recursive_child_decode_count,
            layout_statuses: layout_report
                .layouts
                .iter()
                .map(|layout| layout.status.clone())
                .collect(),
            status: status.to_string(),
        },
        payloads,
        layout_report,
    }
}

pub fn classify_subnode_payloads(payloads: &[PayloadBlock]) -> SubnodeLayoutReport {
    let layouts = payloads
        .iter()
        .map(classify_subnode_block_layout)
        .collect::<Vec<_>>();
    let table_layout_count = layouts
        .iter()
        .filter(|layout| layout.layout_kind == "table_context")
        .count();
    let child_reference_layout_count = layouts
        .iter()
        .filter(|layout| {
            matches!(
                layout.layout_kind.as_str(),
                "synthetic_child_reference" | "unicode_slblock"
            )
        })
        .count();
    let unsupported_layout_count = layouts
        .iter()
        .filter(|layout| layout.layout_kind == "unsupported")
        .count();
    let heap_layout_count = layouts
        .iter()
        .filter(|layout| layout.layout_kind.starts_with("heap_"))
        .count();
    let child_reference_count = layouts
        .iter()
        .map(|layout| layout.child_block_ids.len())
        .sum::<usize>();
    let base_status = if layouts.is_empty() {
        "subnode_layouts_empty"
    } else if unsupported_layout_count == 0 {
        "subnode_layouts_classified"
    } else if table_layout_count > 0 || child_reference_layout_count > 0 || heap_layout_count > 0 {
        "subnode_layouts_partially_classified"
    } else {
        "subnode_layouts_unsupported"
    };
    let first_table_layout_status = layouts
        .iter()
        .find(|layout| layout.layout_kind == "table_context")
        .map(|layout| layout.status.as_str())
        .unwrap_or("");
    let first_unsupported_layout_status = layouts
        .iter()
        .find(|layout| layout.layout_kind == "unsupported")
        .map(|layout| layout.status.as_str())
        .unwrap_or("");
    let status = format!(
        "{base_status}; subnode_heap_contexts={heap_layout_count}; subnode_heap_table_contexts={}; subnode_heap_property_contexts={}; subnode_heap_bth_contexts={}; subnode_slblock_entries={}; subnode_slblock_data_references={}; subnode_slblock_sub_references={}; subnode_unsupported_payload_block_id={}; subnode_unsupported_payload_byte_len={}; subnode_unsupported_payload_prefix_hex={}; subnode_table_declared_columns={}; subnode_table_columns={}; subnode_table_declared_rows={}; subnode_table_rows={}; subnode_table_row_width={}; subnode_table_values={}; subnode_table_omitted_values={}; subnode_table_selected_columns={}; subnode_table_plausible_columns={}; subnode_table_unknown_columns={}; subnode_table_selected_values={}; subnode_table_plausible_values={}; subnode_table_unknown_values={}; subnode_table_byte_swapped_selected_columns={}; subnode_table_byte_swapped_plausible_columns={}; subnode_table_low_word_known_type_columns={}; subnode_table_high_word_known_type_columns={}; subnode_table_byte_swapped_selected_values={}; subnode_table_byte_swapped_plausible_values={}; subnode_table_low_word_known_type_values={}; subnode_table_high_word_known_type_values={}; subnode_table_first_unknown_tag={}; subnode_table_second_unknown_tag={}; subnode_table_first_unknown_tag_low_word={}; subnode_table_first_unknown_tag_high_word={}; subnode_table_second_unknown_tag_low_word={}; subnode_table_second_unknown_tag_high_word={}; subnode_table_first_unknown_offset={}; subnode_table_first_unknown_width={}; subnode_table_second_unknown_offset={}; subnode_table_second_unknown_width={}; subnode_table_payload_byte_len={}; subnode_table_payload_prefix_byte_len={}; subnode_table_payload_prefix_truncated={}; subnode_table_payload_prefix_hex={}",
        layouts
            .iter()
            .filter(|layout| layout.layout_kind == "heap_table_context")
            .count(),
        layouts
            .iter()
            .filter(|layout| layout.layout_kind == "heap_property_context")
            .count(),
        layouts
            .iter()
            .filter(|layout| layout.layout_kind == "heap_bth")
            .count(),
        status_sum(&layouts, "subnode_slblock_entries"),
        status_sum(&layouts, "subnode_slblock_data_references"),
        status_sum(&layouts, "subnode_slblock_sub_references"),
        status_counter(first_unsupported_layout_status, "subnode_payload_block_id"),
        status_counter(first_unsupported_layout_status, "subnode_payload_byte_len"),
        status_value(first_unsupported_layout_status, "subnode_payload_prefix_hex"),
        status_sum(&layouts, "subnode_table_declared_columns"),
        status_sum(&layouts, "subnode_table_columns"),
        status_sum(&layouts, "subnode_table_declared_rows"),
        status_sum(&layouts, "subnode_table_rows"),
        status_sum(&layouts, "subnode_table_row_width"),
        status_sum(&layouts, "subnode_table_values"),
        status_sum(&layouts, "subnode_table_omitted_values"),
        status_sum(&layouts, "subnode_table_selected_columns"),
        status_sum(&layouts, "subnode_table_plausible_columns"),
        status_sum(&layouts, "subnode_table_unknown_columns"),
        status_sum(&layouts, "subnode_table_selected_values"),
        status_sum(&layouts, "subnode_table_plausible_values"),
        status_sum(&layouts, "subnode_table_unknown_values"),
        status_sum(&layouts, "subnode_table_byte_swapped_selected_columns"),
        status_sum(&layouts, "subnode_table_byte_swapped_plausible_columns"),
        status_sum(&layouts, "subnode_table_low_word_known_type_columns"),
        status_sum(&layouts, "subnode_table_high_word_known_type_columns"),
        status_sum(&layouts, "subnode_table_byte_swapped_selected_values"),
        status_sum(&layouts, "subnode_table_byte_swapped_plausible_values"),
        status_sum(&layouts, "subnode_table_low_word_known_type_values"),
        status_sum(&layouts, "subnode_table_high_word_known_type_values"),
        status_sum(&layouts, "subnode_table_first_unknown_tag"),
        status_sum(&layouts, "subnode_table_second_unknown_tag"),
        status_sum(&layouts, "subnode_table_first_unknown_tag_low_word"),
        status_sum(&layouts, "subnode_table_first_unknown_tag_high_word"),
        status_sum(&layouts, "subnode_table_second_unknown_tag_low_word"),
        status_sum(&layouts, "subnode_table_second_unknown_tag_high_word"),
        status_sum(&layouts, "subnode_table_first_unknown_offset"),
        status_sum(&layouts, "subnode_table_first_unknown_width"),
        status_sum(&layouts, "subnode_table_second_unknown_offset"),
        status_sum(&layouts, "subnode_table_second_unknown_width"),
        status_counter(first_table_layout_status, "subnode_table_payload_byte_len"),
        status_counter(first_table_layout_status, "subnode_table_payload_prefix_byte_len"),
        status_counter(first_table_layout_status, "subnode_table_payload_prefix_truncated"),
        status_value(first_table_layout_status, "subnode_table_payload_prefix_hex"),
    );
    SubnodeLayoutReport {
        block_count: layouts.len(),
        table_layout_count,
        child_reference_layout_count,
        unsupported_layout_count,
        child_reference_count,
        layouts,
        status,
    }
}

pub fn classify_subnode_block_layout(payload: &PayloadBlock) -> SubnodeBlockLayout {
    if payload.bytes.len() < 8 {
        return SubnodeBlockLayout {
            block_id: payload.block_id,
            offset: payload.block_ref.offset.0,
            size: payload.block_ref.size,
            byte_len: payload.bytes.len(),
            layout_kind: "unsupported".to_string(),
            child_block_ids: Vec::new(),
            status: "subnode_layout_unsupported_short_block".to_string(),
        };
    }
    if payload.bytes.starts_with(SYNTHETIC_CHILD_REF_MAGIC) {
        return classify_synthetic_child_reference_layout(payload);
    }
    if is_unicode_slblock(&payload.bytes) {
        return classify_unicode_slblock_layout(payload);
    }
    if payload.bytes.get(2) == Some(&HEAP_SIGNATURE) {
        return classify_heap_layout(payload);
    }
    if !is_admissible_flat_table(&payload.bytes) {
        return unsupported_payload_layout(payload, "no_supported_payload_signature");
    }
    match TableContext::parse_with_report(&payload.bytes, payload.block_ref.offset.0) {
        Ok(report) => {
            let value_count = report
                .context
                .rows
                .iter()
                .map(|row| row.values.len())
                .sum::<usize>();
            let prefix_len = payload.bytes.len().min(TABLE_PAYLOAD_PREFIX_BYTES);
            let prefix_hex = hex::encode(&payload.bytes[..prefix_len]);
            let prefix_truncated = usize::from(prefix_len < payload.bytes.len());
            SubnodeBlockLayout {
                block_id: payload.block_id,
                offset: payload.block_ref.offset.0,
                size: payload.block_ref.size,
                byte_len: payload.bytes.len(),
                layout_kind: "table_context".to_string(),
                child_block_ids: Vec::new(),
                status: format!(
                    "{}; subnode_table_declared_columns={}; subnode_table_columns={}; subnode_table_declared_rows={}; subnode_table_rows={}; subnode_table_row_width={}; subnode_table_values={}; subnode_table_omitted_values={}; subnode_table_selected_columns={}; subnode_table_plausible_columns={}; subnode_table_unknown_columns={}; subnode_table_selected_values={}; subnode_table_plausible_values={}; subnode_table_unknown_values={}; subnode_table_byte_swapped_selected_columns={}; subnode_table_byte_swapped_plausible_columns={}; subnode_table_low_word_known_type_columns={}; subnode_table_high_word_known_type_columns={}; subnode_table_byte_swapped_selected_values={}; subnode_table_byte_swapped_plausible_values={}; subnode_table_low_word_known_type_values={}; subnode_table_high_word_known_type_values={}; subnode_table_payload_byte_len={}; subnode_table_payload_prefix_byte_len={}; subnode_table_payload_prefix_truncated={}; subnode_table_payload_prefix_hex={}",
                    report.status,
                    report.declared_column_count,
                    report.parsed_column_count,
                    report.declared_row_count,
                    report.parsed_row_count,
                    report.row_width,
                    value_count,
                    report.omitted_value_count,
                    report.selected_column_count,
                    report.plausible_column_count,
                    report.unknown_column_count,
                    report.selected_value_count,
                    report.plausible_value_count,
                    report.unknown_value_count,
                    report.byte_swapped_selected_column_count,
                    report.byte_swapped_plausible_column_count,
                    report.low_word_known_type_column_count,
                    report.high_word_known_type_column_count,
                    report.byte_swapped_selected_value_count,
                    report.byte_swapped_plausible_value_count,
                    report.low_word_known_type_value_count,
                    report.high_word_known_type_value_count,
                    payload.bytes.len(),
                    prefix_len,
                    prefix_truncated,
                    prefix_hex,
                ),
            }
        }
        Err(reason) => unsupported_payload_layout(payload, &reason.to_string()),
    }
}

fn classify_heap_layout(payload: &PayloadBlock) -> SubnodeBlockLayout {
    match HeapOnNode::parse(&payload.bytes, payload.block_ref.offset.0) {
        Ok(heap) => {
            let layout_kind = match heap.header.client_signature {
                HEAP_CLIENT_TABLE_CONTEXT => "heap_table_context",
                HEAP_CLIENT_PROPERTY_CONTEXT => "heap_property_context",
                HEAP_CLIENT_BTH => "heap_bth",
                _ => "heap_other",
            };
            SubnodeBlockLayout {
                block_id: payload.block_id,
                offset: payload.block_ref.offset.0,
                size: payload.block_ref.size,
                byte_len: payload.bytes.len(),
                layout_kind: layout_kind.to_string(),
                child_block_ids: Vec::new(),
                status: format!(
                    "subnode_layout_{layout_kind}; subnode_heap_client_signature={}; subnode_heap_allocations={}; subnode_payload_block_id={}; subnode_payload_byte_len={}",
                    heap.header.client_signature,
                    heap.allocations.len(),
                    payload.block_id.0,
                    payload.bytes.len()
                ),
            }
        }
        Err(reason) => unsupported_payload_layout(payload, &format!("heap_invalid:{reason}")),
    }
}

fn is_admissible_flat_table(bytes: &[u8]) -> bool {
    if bytes.len() < 8 {
        return false;
    }
    let columns = u16::from_le_bytes([bytes[0], bytes[1]]) as usize;
    let rows = u16::from_le_bytes([bytes[2], bytes[3]]) as usize;
    let row_width = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
    if columns == 0 || columns > 256 || row_width == 0 {
        return false;
    }
    let Some(descriptor_end) = columns.checked_mul(8).and_then(|size| 8usize.checked_add(size))
    else {
        return false;
    };
    let Some(row_bytes) = rows.checked_mul(row_width) else {
        return false;
    };
    if descriptor_end > bytes.len() || row_bytes > bytes.len() - descriptor_end {
        return false;
    }
    (0..columns).all(|index| {
        let start = 8 + index * 8;
        let offset = u16::from_le_bytes([bytes[start + 4], bytes[start + 5]]) as usize;
        let width = u16::from_le_bytes([bytes[start + 6], bytes[start + 7]]) as usize;
        width > 0 && offset.checked_add(width).is_some_and(|end| end <= row_width)
    })
}

fn unsupported_payload_layout(payload: &PayloadBlock, reason: &str) -> SubnodeBlockLayout {
    let prefix_len = payload.bytes.len().min(TABLE_PAYLOAD_PREFIX_BYTES);
    SubnodeBlockLayout {
        block_id: payload.block_id,
        offset: payload.block_ref.offset.0,
        size: payload.block_ref.size,
        byte_len: payload.bytes.len(),
        layout_kind: "unsupported".to_string(),
        child_block_ids: Vec::new(),
        status: format!(
            "subnode_layout_unsupported; reason={reason}; subnode_payload_block_id={}; subnode_payload_byte_len={}; subnode_payload_prefix_hex={}",
            payload.block_id.0,
            payload.bytes.len(),
            hex::encode(&payload.bytes[..prefix_len])
        ),
    }
}

fn is_unicode_slblock(bytes: &[u8]) -> bool {
    bytes.len() >= UNICODE_SLBLOCK_HEADER_BYTES
        && bytes[0] == SLBLOCK_TYPE
        && bytes[1] == SLBLOCK_LEAF_LEVEL
        && u16::from_le_bytes([bytes[2], bytes[3]]) > 0
        && bytes[4..8] == [0, 0, 0, 0]
}

fn classify_unicode_slblock_layout(payload: &PayloadBlock) -> SubnodeBlockLayout {
    let declared_entry_count = u16::from_le_bytes([payload.bytes[2], payload.bytes[3]]) as usize;
    let available_entry_count = payload
        .bytes
        .len()
        .saturating_sub(UNICODE_SLBLOCK_HEADER_BYTES)
        / UNICODE_SLENTRY_BYTES;
    let parsed_entry_count = declared_entry_count.min(available_entry_count);
    let mut child_block_ids = Vec::new();
    let mut data_reference_count = 0usize;
    let mut sub_reference_count = 0usize;

    for index in 0..parsed_entry_count {
        let start = UNICODE_SLBLOCK_HEADER_BYTES + index * UNICODE_SLENTRY_BYTES;
        let bid_data = read_u64_le(&payload.bytes, start + 8);
        let bid_sub = read_u64_le(&payload.bytes, start + 16);
        if bid_data != 0 {
            data_reference_count += 1;
            child_block_ids.push(BlockId(bid_data));
        }
        if bid_sub != 0 {
            sub_reference_count += 1;
            child_block_ids.push(BlockId(bid_sub));
        }
    }

    let status = if parsed_entry_count == declared_entry_count {
        "subnode_layout_unicode_slblock_classified"
    } else {
        "subnode_layout_unicode_slblock_truncated"
    };
    SubnodeBlockLayout {
        block_id: payload.block_id,
        offset: payload.block_ref.offset.0,
        size: payload.block_ref.size,
        byte_len: payload.bytes.len(),
        layout_kind: "unicode_slblock".to_string(),
        child_block_ids,
        status: format!(
            "{status}; subnode_slblock_entries={parsed_entry_count}; subnode_slblock_declared_entries={declared_entry_count}; subnode_slblock_data_references={data_reference_count}; subnode_slblock_sub_references={sub_reference_count}"
        ),
    }
}

fn read_u64_le(bytes: &[u8], offset: usize) -> u64 {
    let mut value = [0u8; 8];
    value.copy_from_slice(&bytes[offset..offset + 8]);
    u64::from_le_bytes(value)
}

fn classify_synthetic_child_reference_layout(payload: &PayloadBlock) -> SubnodeBlockLayout {
    let declared_child_count = u16::from_le_bytes([payload.bytes[4], payload.bytes[5]]) as usize;
    let mut child_block_ids = Vec::new();
    let mut cursor = 8usize;
    for _ in 0..declared_child_count {
        if cursor + 8 > payload.bytes.len() {
            break;
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&payload.bytes[cursor..cursor + 8]);
        child_block_ids.push(BlockId(u64::from_le_bytes(bytes)));
        cursor += 8;
    }
    let status = if child_block_ids.len() == declared_child_count {
        "subnode_layout_child_references_classified"
    } else {
        "subnode_layout_child_references_truncated"
    };
    SubnodeBlockLayout {
        block_id: payload.block_id,
        offset: payload.block_ref.offset.0,
        size: payload.block_ref.size,
        byte_len: payload.bytes.len(),
        layout_kind: "synthetic_child_reference".to_string(),
        child_block_ids,
        status: status.to_string(),
    }
}

fn empty_layout_report() -> SubnodeLayoutReport {
    SubnodeLayoutReport {
        block_count: 0,
        table_layout_count: 0,
        child_reference_layout_count: 0,
        unsupported_layout_count: 0,
        child_reference_count: 0,
        layouts: Vec::new(),
        status: "subnode_layouts_empty; subnode_heap_contexts=0; subnode_heap_table_contexts=0; subnode_heap_property_contexts=0; subnode_heap_bth_contexts=0; subnode_slblock_entries=0; subnode_slblock_data_references=0; subnode_slblock_sub_references=0; subnode_unsupported_payload_block_id=0; subnode_unsupported_payload_byte_len=0; subnode_unsupported_payload_prefix_hex=; subnode_table_declared_columns=0; subnode_table_columns=0; subnode_table_declared_rows=0; subnode_table_rows=0; subnode_table_row_width=0; subnode_table_values=0; subnode_table_omitted_values=0; subnode_table_selected_columns=0; subnode_table_plausible_columns=0; subnode_table_unknown_columns=0; subnode_table_selected_values=0; subnode_table_plausible_values=0; subnode_table_unknown_values=0; subnode_table_byte_swapped_selected_columns=0; subnode_table_byte_swapped_plausible_columns=0; subnode_table_low_word_known_type_columns=0; subnode_table_high_word_known_type_columns=0; subnode_table_byte_swapped_selected_values=0; subnode_table_byte_swapped_plausible_values=0; subnode_table_low_word_known_type_values=0; subnode_table_high_word_known_type_values=0; subnode_table_first_unknown_tag=0; subnode_table_second_unknown_tag=0; subnode_table_first_unknown_tag_low_word=0; subnode_table_first_unknown_tag_high_word=0; subnode_table_second_unknown_tag_low_word=0; subnode_table_second_unknown_tag_high_word=0; subnode_table_first_unknown_offset=0; subnode_table_first_unknown_width=0; subnode_table_second_unknown_offset=0; subnode_table_second_unknown_width=0; subnode_table_payload_byte_len=0; subnode_table_payload_prefix_byte_len=0; subnode_table_payload_prefix_truncated=0; subnode_table_payload_prefix_hex=".to_string(),
    }
}

fn status_sum(layouts: &[SubnodeBlockLayout], key: &str) -> usize {
    layouts
        .iter()
        .map(|layout| status_counter(&layout.status, key))
        .sum()
}

fn status_counter(status: &str, key: &str) -> usize {
    let marker = format!("{key}=");
    status
        .split(&marker)
        .nth(1)
        .and_then(|tail| tail.split([';', ',']).next())
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0)
}

fn status_value(status: &str, key: &str) -> String {
    let marker = format!("{key}=");
    status
        .split(&marker)
        .nth(1)
        .and_then(|tail| tail.split([';', ',']).next())
        .map(str::trim)
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use super::{
        classify_subnode_block_layout, classify_subnode_payloads, load_recursive_subnode_blocks,
        SubnodeReference,
    };
    use crate::pst::bbt::{BbtEntry, BbtIndex};
    use crate::pst::limits::ParserLimits;
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset, NodeId};
    use crate::pst::reader::PstByteReader;

    #[test]
    fn classifies_unicode_slblock_before_table_context() {
        let payload = payload_block(unicode_slblock(0x692, 0x7c, 0x7a));

        let layout = classify_subnode_block_layout(&payload);

        assert_eq!(layout.layout_kind, "unicode_slblock");
        assert_eq!(layout.child_block_ids, vec![BlockId(0x7c), BlockId(0x7a)]);
        assert!(layout.status.contains("subnode_slblock_entries=1"));
        assert!(layout.status.contains("subnode_slblock_data_references=1"));
        assert!(layout.status.contains("subnode_slblock_sub_references=1"));
    }

    #[test]
    fn resolves_slentry_targets_without_following_cycles() {
        let root = unicode_slblock(0x692, 0x7c, 0x7a);
        let mut table = Vec::new();
        table.extend_from_slice(&1u16.to_le_bytes());
        table.extend_from_slice(&1u16.to_le_bytes());
        table.extend_from_slice(&4u16.to_le_bytes());
        table.extend_from_slice(&0u16.to_le_bytes());
        table.extend_from_slice(&0x0037_001fu32.to_le_bytes());
        table.extend_from_slice(&0u16.to_le_bytes());
        table.extend_from_slice(&4u16.to_le_bytes());
        table.extend_from_slice(&[1, 2, 3, 4]);

        let file = NamedTempFile::new().unwrap();
        let mut file_bytes = root.clone();
        file_bytes.extend_from_slice(&table);
        fs::write(file.path(), file_bytes).unwrap();
        let reader = PstByteReader::open(file.path()).unwrap();
        let bbt = BbtIndex {
            root: None,
            entries: vec![
                bbt_entry(0x7a, 0, root.len() as u64),
                bbt_entry(0x7c, root.len() as u64, table.len() as u64),
            ],
            parsed_pages: 0,
            discovered_child_pages: 0,
            traversal_error_count: 0,
            duplicate_entry_count: 0,
            truncated_entry_count: 0,
            status: "test".to_string(),
        };
        let reference = SubnodeReference {
            node_id: NodeId(1),
            subnode_block_id: BlockId(0x7a),
            status: "test".to_string(),
        };

        let loaded =
            load_recursive_subnode_blocks(&reader, &bbt, &reference, 1, ParserLimits::default());

        assert_eq!(loaded.report.decoded_block_count, 2);
        assert_eq!(loaded.report.recursive_child_decode_count, 1);
        assert_eq!(loaded.layout_report.child_reference_count, 2);
        assert_eq!(loaded.layout_report.child_reference_layout_count, 1);
        assert_eq!(loaded.layout_report.table_layout_count, 1);
        assert_eq!(loaded.layout_report.unsupported_layout_count, 0);
        assert_eq!(
            loaded
                .layout_report
                .layouts
                .iter()
                .filter(|layout| layout.layout_kind == "unicode_slblock")
                .count(),
            1
        );
    }

    #[test]
    fn table_layout_status_captures_bounded_payload_prefix() {
        let payload = payload_block(valid_table_payload(40));

        let layout = classify_subnode_block_layout(&payload);

        assert_eq!(layout.layout_kind, "table_context");
        assert!(layout.status.contains("subnode_table_payload_byte_len=56"));
        assert!(layout
            .status
            .contains("subnode_table_payload_prefix_byte_len=32"));
        assert!(layout
            .status
            .contains("subnode_table_payload_prefix_truncated=1"));
        assert!(layout
            .status
            .contains("subnode_table_payload_prefix_hex=01000100280000001f00370000000400"));
    }

    #[test]
    fn layout_report_propagates_first_table_payload_prefix() {
        let payload = payload_block(valid_table_payload(4));

        let report = classify_subnode_payloads(&[payload]);

        assert!(report.status.contains("subnode_table_payload_byte_len=20"));
        assert!(report
            .status
            .contains("subnode_table_payload_prefix_byte_len=20"));
        assert!(report
            .status
            .contains("subnode_table_payload_prefix_truncated=0"));
        assert!(report
            .status
            .contains("subnode_table_payload_prefix_hex=01000100040000001f0037000000040000000000"));
    }

    #[test]
    fn rejects_implausible_flat_table_declarations() {
        let payload = payload_block(vec![0xec, 0xa3, 0x40, 0x5e, 0xae, 0x82, 0x41, 0x82]);

        let layout = classify_subnode_block_layout(&payload);

        assert_eq!(layout.layout_kind, "unsupported");
        assert!(layout.status.contains("no_supported_payload_signature"));
        assert!(layout.status.contains("subnode_payload_block_id=7"));
    }

    #[test]
    fn classifies_valid_heap_client_signature() {
        let payload = payload_block(table_heap_payload());

        let layout = classify_subnode_block_layout(&payload);

        assert_eq!(layout.layout_kind, "heap_table_context");
        assert!(layout.status.contains("subnode_heap_client_signature=124"));
    }

    fn payload_block(bytes: Vec<u8>) -> PayloadBlock {
        PayloadBlock {
            block_id: BlockId(7),
            block_ref: BlockRef {
                block_id: BlockId(7),
                offset: ByteOffset(100),
                size: bytes.len() as u64,
            },
            bytes,
            status: "payload_loaded".to_string(),
        }
    }

    fn unicode_slblock(nid: u64, bid_data: u64, bid_sub: u64) -> Vec<u8> {
        let mut bytes = vec![0x02, 0x00, 0x01, 0x00, 0, 0, 0, 0];
        bytes.extend_from_slice(&nid.to_le_bytes());
        bytes.extend_from_slice(&bid_data.to_le_bytes());
        bytes.extend_from_slice(&bid_sub.to_le_bytes());
        bytes
    }

    fn bbt_entry(block_id: u64, offset: u64, size: u64) -> BbtEntry {
        BbtEntry {
            block_id: BlockId(block_id),
            offset: ByteOffset(offset),
            size,
        }
    }

    fn valid_table_payload(row_width: u16) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&row_width.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0x0037_001fu32.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.resize(16 + row_width as usize, 0);
        bytes
    }

    fn table_heap_payload() -> Vec<u8> {
        let mut bytes = vec![0u8; 48];
        bytes[0..2].copy_from_slice(&32u16.to_le_bytes());
        bytes[2] = 0xec;
        bytes[3] = 0x7c;
        bytes[4..8].copy_from_slice(&0x20u32.to_le_bytes());
        bytes[32..34].copy_from_slice(&1u16.to_le_bytes());
        bytes[34..36].copy_from_slice(&0u16.to_le_bytes());
        bytes[36..38].copy_from_slice(&8u16.to_le_bytes());
        bytes[38..40].copy_from_slice(&16u16.to_le_bytes());
        bytes
    }
}
