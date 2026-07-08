use std::collections::{HashSet, VecDeque};

use crate::pst::bbt::BbtIndex;
use crate::pst::limits::ParserLimits;
use crate::pst::nbt::{NbtEntry, NbtIndex};
use crate::pst::payload::{load_payload_block, PayloadBlock};
use crate::pst::primitives::{BlockId, NodeId};
use crate::pst::reader::PstByteReader;
use crate::pst::table_context::TableContext;

const SYNTHETIC_CHILD_REF_MAGIC: &[u8; 4] = b"SNOD";

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
        .filter(|layout| layout.layout_kind == "synthetic_child_reference")
        .count();
    let unsupported_layout_count = layouts
        .iter()
        .filter(|layout| layout.layout_kind == "unsupported")
        .count();
    let child_reference_count = layouts
        .iter()
        .map(|layout| layout.child_block_ids.len())
        .sum::<usize>();
    let base_status = if layouts.is_empty() {
        "subnode_layouts_empty"
    } else if unsupported_layout_count == 0 {
        "subnode_layouts_classified"
    } else if table_layout_count > 0 || child_reference_layout_count > 0 {
        "subnode_layouts_partially_classified"
    } else {
        "subnode_layouts_unsupported"
    };
    let status = format!(
        "{base_status}; subnode_table_declared_columns={}; subnode_table_columns={}; subnode_table_declared_rows={}; subnode_table_rows={}; subnode_table_values={}; subnode_table_omitted_values={}; subnode_table_selected_columns={}; subnode_table_plausible_columns={}; subnode_table_unknown_columns={}; subnode_table_selected_values={}; subnode_table_plausible_values={}; subnode_table_unknown_values={}",
        status_sum(&layouts, "subnode_table_declared_columns"),
        status_sum(&layouts, "subnode_table_columns"),
        status_sum(&layouts, "subnode_table_declared_rows"),
        status_sum(&layouts, "subnode_table_rows"),
        status_sum(&layouts, "subnode_table_values"),
        status_sum(&layouts, "subnode_table_omitted_values"),
        status_sum(&layouts, "subnode_table_selected_columns"),
        status_sum(&layouts, "subnode_table_plausible_columns"),
        status_sum(&layouts, "subnode_table_unknown_columns"),
        status_sum(&layouts, "subnode_table_selected_values"),
        status_sum(&layouts, "subnode_table_plausible_values"),
        status_sum(&layouts, "subnode_table_unknown_values"),
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
    match TableContext::parse_with_report(&payload.bytes, payload.block_ref.offset.0) {
        Ok(report) => {
            let value_count = report
                .context
                .rows
                .iter()
                .map(|row| row.values.len())
                .sum::<usize>();
            SubnodeBlockLayout {
                block_id: payload.block_id,
                offset: payload.block_ref.offset.0,
                size: payload.block_ref.size,
                byte_len: payload.bytes.len(),
                layout_kind: "table_context".to_string(),
                child_block_ids: Vec::new(),
                status: format!(
                    "{}; subnode_table_declared_columns={}; subnode_table_columns={}; subnode_table_declared_rows={}; subnode_table_rows={}; subnode_table_row_width={}; subnode_table_values={}; subnode_table_omitted_values={}; subnode_table_selected_columns={}; subnode_table_plausible_columns={}; subnode_table_unknown_columns={}; subnode_table_selected_values={}; subnode_table_plausible_values={}; subnode_table_unknown_values={}",
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
                ),
            }
        }
        Err(reason) => SubnodeBlockLayout {
            block_id: payload.block_id,
            offset: payload.block_ref.offset.0,
            size: payload.block_ref.size,
            byte_len: payload.bytes.len(),
            layout_kind: "unsupported".to_string(),
            child_block_ids: Vec::new(),
            status: format!("subnode_layout_unsupported; reason={reason}"),
        },
    }
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
        status: "subnode_layouts_empty; subnode_table_declared_columns=0; subnode_table_columns=0; subnode_table_declared_rows=0; subnode_table_rows=0; subnode_table_values=0; subnode_table_omitted_values=0; subnode_table_selected_columns=0; subnode_table_plausible_columns=0; subnode_table_unknown_columns=0; subnode_table_selected_values=0; subnode_table_plausible_values=0; subnode_table_unknown_values=0".to_string(),
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
