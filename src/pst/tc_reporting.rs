use crate::pst::payload::PayloadBlock;
use crate::pst::subnodes::SubnodeReference;
use crate::pst::tc_complete_recipient_projection::{
    project_complete_recipient_records, project_complete_recipient_records_from_rows,
    TcCompleteRecipientProjectionReport,
};
use crate::pst::tc_descriptor_evidence::{
    build_descriptor_bitmap_evidence_from_columns, format_descriptor_bitmap_evidence,
};
use crate::pst::tc_fixed_width_diagnostic::{build_fixed_width_diagnostic, TcFixedWidthDiagnostic};
use crate::pst::tc_fixed_width_projection::{
    project_fixed_width_row_evidence, project_fixed_width_row_evidence_from_rows,
    TcFixedWidthProjectionReport, TC_FIXED_WIDTH_EVIDENCE_UNAVAILABLE,
};
use crate::pst::heap::HeapOnNode;
use crate::pst::tc_heap::resolve_tcinfo_from_heap;
use crate::pst::tc_recipient_identity_diagnostic::{
    build_recipient_identity_diagnostic, unavailable_recipient_identity_diagnostic,
    TcRecipientIdentityDiagnostic,
};
use crate::pst::tc_recipient_identity_projection::{
    project_recipient_identity_strings, project_recipient_identity_strings_from_rows,
};
use crate::pst::tc_subnode_rows::{resolve_heap_row_storage, resolve_subnode_row_storage};
use crate::pst::tcinfo::TcColumnDescriptor;

const HEAP_SIGNATURE: u8 = 0xec;
const HEAP_CLIENT_TABLE_CONTEXT: u8 = 0x7c;
const SLBLOCK_TYPE: u8 = 0x02;
const SLBLOCK_LEAF_LEVEL: u8 = 0x00;
const UNICODE_SLBLOCK_HEADER_BYTES: usize = 8;
const UNICODE_SLENTRY_BYTES: usize = 24;
const NID_TYPE_MASK: u32 = 0x1f;
const NID_TYPE_RECIPIENT_TABLE: u32 = 0x12;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TcHeapDiagnostic {
    pub block_id: u64,
    pub payload_byte_len: usize,
    pub resolved: bool,
    pub column_count: usize,
    pub row_reference_count: usize,
    pub row_references_in_bounds: usize,
    pub row_references_out_of_bounds: usize,
    pub rows_require_subnode_resolution: bool,
    pub rows_nid: u32,
    pub subnode_row_match_count: usize,
    pub resolved_row_payload_count: usize,
    pub row_data_byte_len: usize,
    pub row_reference_values: Vec<u32>,
    pub row_spans: Vec<usize>,
    pub inferred_row_width: usize,
    pub tcinfo_data_region_boundaries: [u16; 4],
    pub max_column_extent: usize,
    pub bitmap_byte_len: usize,
    pub bitmap_end: usize,
    pub bitmap_rows_analyzed: usize,
    pub bitmap_set_counts: Vec<usize>,
    pub bitmap_unset_counts: Vec<usize>,
    pub bitmap_masks: Vec<String>,
    pub bitmap_status: String,
    pub descriptor_evidence: String,
    pub descriptor_evidence_status: String,
    pub row_layout_extents_valid: bool,
    pub row_layout_status: String,
    pub fixed_width: TcFixedWidthDiagnostic,
    pub recipient_identity: TcRecipientIdentityDiagnostic,
    pub complete_recipients: Option<TcCompleteRecipientProjectionReport>,
    pub status: String,
    pub error: Option<String>,
}

impl TcHeapDiagnostic {
    fn status_fragment(&self) -> String {
        let error = self.error.as_deref().unwrap_or("none").replace(';', ",");
        let row_reference_values = self
            .row_reference_values
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(":");
        let row_spans = self
            .row_spans
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(":");
        let bitmap_set_counts = self
            .bitmap_set_counts
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(":");
        let bitmap_unset_counts = self
            .bitmap_unset_counts
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(":");
        let bitmap_masks = self.bitmap_masks.join(":");
        format!(
            "bid=0x{:x},bytes={},resolved={},columns={},row_refs={},in_bounds={},out_of_bounds={},subnode_rows={},rows_nid=0x{:x},row_matches={},row_payloads={},row_bytes={},row_reference_values={},row_spans={},row_width={},tcinfo_regions={}:{}:{}:{},max_column_extent={},bitmap_bytes={},bitmap_end={},bitmap_rows={},bitmap_set_counts={},bitmap_unset_counts={},bitmap_masks={},bitmap_status={},descriptor_evidence={},descriptor_evidence_status={},row_layout_valid={},row_layout_status={},status={},error={},{},{}",
            self.block_id,
            self.payload_byte_len,
            usize::from(self.resolved),
            self.column_count,
            self.row_reference_count,
            self.row_references_in_bounds,
            self.row_references_out_of_bounds,
            usize::from(self.rows_require_subnode_resolution),
            self.rows_nid,
            self.subnode_row_match_count,
            self.resolved_row_payload_count,
            self.row_data_byte_len,
            row_reference_values,
            row_spans,
            self.inferred_row_width,
            self.tcinfo_data_region_boundaries[0],
            self.tcinfo_data_region_boundaries[1],
            self.tcinfo_data_region_boundaries[2],
            self.tcinfo_data_region_boundaries[3],
            self.max_column_extent,
            self.bitmap_byte_len,
            self.bitmap_end,
            self.bitmap_rows_analyzed,
            bitmap_set_counts,
            bitmap_unset_counts,
            bitmap_masks,
            self.bitmap_status.replace(';', ","),
            self.descriptor_evidence,
            self.descriptor_evidence_status.replace(';', ","),
            usize::from(self.row_layout_extents_valid),
            self.row_layout_status.replace(';', ","),
            self.status.replace(';', ","),
            error,
            self.fixed_width.status_fragment(),
            self.recipient_identity.status_fragment(),
        )
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TcHeapAggregateReport {
    pub payload_count: usize,
    pub table_heap_count: usize,
    pub resolved_table_heap_count: usize,
    pub failed_table_heap_count: usize,
    pub total_columns: usize,
    pub total_row_references: usize,
    pub total_row_references_in_bounds: usize,
    pub total_row_references_out_of_bounds: usize,
    pub subnode_backed_row_heap_count: usize,
    pub diagnostics: Vec<TcHeapDiagnostic>,
    pub status: String,
}

impl TcHeapAggregateReport {
    pub fn progress_status(&self) -> String {
        let diagnostic_fragment = if self.diagnostics.is_empty() {
            "none".to_string()
        } else {
            self.diagnostics
                .iter()
                .map(TcHeapDiagnostic::status_fragment)
                .collect::<Vec<_>>()
                .join("|")
        };
        format!(
            "pq42_status={}; pq42_payloads={}; pq42_table_heaps={}; pq42_resolved_table_heaps={}; pq42_failed_table_heaps={}; pq42_columns={}; pq42_row_references={}; pq42_row_references_in_bounds={}; pq42_row_references_out_of_bounds={}; pq42_subnode_backed_row_heaps={}; pq42_diagnostics={diagnostic_fragment}",
            self.status,
            self.payload_count,
            self.table_heap_count,
            self.resolved_table_heap_count,
            self.failed_table_heap_count,
            self.total_columns,
            self.total_row_references,
            self.total_row_references_in_bounds,
            self.total_row_references_out_of_bounds,
            self.subnode_backed_row_heap_count,
        )
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TcSubnodeProbeReport {
    pub root_node_id: u64,
    pub root_subnode_block_id: u64,
    pub decoded_payload_count: usize,
    pub direct_recipient_table_bids: Vec<u64>,
    pub table_heaps: TcHeapAggregateReport,
    pub status: String,
}

impl TcSubnodeProbeReport {
    pub fn progress_status(&self) -> String {
        format!(
            "pq43_status={}; pq43_root_node_id=0x{:x}; pq43_root_subnode_bid=0x{:x}; pq43_decoded_payloads={}; pq43_direct_recipient_table_bids={}; {}",
            self.status,
            self.root_node_id,
            self.root_subnode_block_id,
            self.decoded_payload_count,
            self.direct_recipient_table_bids
                .iter()
                .map(|bid| format!("0x{bid:x}"))
                .collect::<Vec<_>>()
                .join(":"),
            self.table_heaps.progress_status()
        )
    }
}

pub fn report_subnode_table_heaps(
    reference: &SubnodeReference,
    payloads: &[PayloadBlock],
) -> TcSubnodeProbeReport {
    let direct_recipient_table_bids =
        direct_recipient_table_bids(reference.subnode_block_id.0, payloads);
    let table_heaps = report_table_heaps(payloads);
    let status = if payloads.is_empty() {
        "pq43_subnode_payloads_empty"
    } else if table_heaps.table_heap_count == 0 {
        "pq43_no_table_heaps_detected"
    } else if table_heaps.failed_table_heap_count == 0 {
        "pq43_table_heaps_resolved"
    } else if table_heaps.resolved_table_heap_count > 0 {
        "pq43_table_heaps_partially_resolved"
    } else {
        "pq43_table_heaps_unresolved"
    };

    TcSubnodeProbeReport {
        root_node_id: reference.node_id.0,
        root_subnode_block_id: reference.subnode_block_id.0,
        decoded_payload_count: payloads.len(),
        direct_recipient_table_bids,
        table_heaps,
        status: status.to_string(),
    }
}

fn direct_recipient_table_bids(root_block_id: u64, payloads: &[PayloadBlock]) -> Vec<u64> {
    let Some(root) = payloads
        .iter()
        .find(|payload| payload.block_id.0 == root_block_id)
    else {
        return Vec::new();
    };
    let bytes = &root.bytes;
    if bytes.len() < UNICODE_SLBLOCK_HEADER_BYTES
        || bytes[0] != SLBLOCK_TYPE
        || bytes[1] != SLBLOCK_LEAF_LEVEL
        || bytes[4..8] != [0, 0, 0, 0]
    {
        return Vec::new();
    }

    let entry_count = u16::from_le_bytes([bytes[2], bytes[3]]) as usize;
    let Some(required_len) = entry_count
        .checked_mul(UNICODE_SLENTRY_BYTES)
        .and_then(|entries| entries.checked_add(UNICODE_SLBLOCK_HEADER_BYTES))
    else {
        return Vec::new();
    };
    if required_len > bytes.len() {
        return Vec::new();
    }

    (0..entry_count)
        .filter_map(|index| {
            let start = UNICODE_SLBLOCK_HEADER_BYTES + index * UNICODE_SLENTRY_BYTES;
            let nid = u32::from_le_bytes([
                bytes[start],
                bytes[start + 1],
                bytes[start + 2],
                bytes[start + 3],
            ]);
            let mut bid_bytes = [0u8; 8];
            bid_bytes.copy_from_slice(&bytes[start + 8..start + 16]);
            let data_bid = u64::from_le_bytes(bid_bytes);
            (nid & NID_TYPE_MASK == NID_TYPE_RECIPIENT_TABLE && data_bid != 0)
                .then_some(data_bid)
        })
        .collect()
}

pub fn report_table_heaps(payloads: &[PayloadBlock]) -> TcHeapAggregateReport {
    let diagnostics = payloads
        .iter()
        .filter(|payload| {
            payload.bytes.len() >= 4
                && payload.bytes[2] == HEAP_SIGNATURE
                && payload.bytes[3] == HEAP_CLIENT_TABLE_CONTEXT
        })
        .map(
            |payload| match resolve_tcinfo_from_heap(&payload.bytes, payload.block_ref.offset.0) {
                Ok(report) => {
                    let row_references = report
                        .row_index_report
                        .as_ref()
                        .map(|row_index| {
                            row_index
                                .entries
                                .iter()
                                .map(|entry| entry.row_reference)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    let heap_row_data = (!report.rows_requires_subnode_resolution)
                        .then(|| {
                            HeapOnNode::parse(&payload.bytes, payload.block_ref.offset.0)
                                .ok()
                                .and_then(|heap| {
                                    heap.allocation_by_hid(
                                        &payload.bytes,
                                        report.rows_hnid,
                                        payload.block_ref.offset.0,
                                    )
                                    .ok()
                                })
                        })
                        .flatten();
                    let row_resolution = if report.rows_requires_subnode_resolution {
                        Some(resolve_subnode_row_storage(
                            payloads,
                            report.rows_hnid,
                            &row_references,
                            report.column_count,
                            report.bitmap_end.saturating_sub(report.bitmap_byte_len),
                            report.bitmap_end,
                        ))
                    } else {
                        heap_row_data.map(|row_data| {
                            resolve_heap_row_storage(
                                report.rows_hnid,
                                row_data,
                                &row_references,
                                report.column_count,
                                report.bitmap_end.saturating_sub(report.bitmap_byte_len),
                                report.bitmap_end,
                            )
                        })
                    };
                    let inferred_row_width = row_resolution
                        .as_ref()
                        .map_or(0, |rows| rows.inferred_row_width);
                    let (row_layout_status, row_layout_extents_valid) = validate_row_layout_extents(
                        inferred_row_width,
                        report.data_region_boundaries,
                        report.max_column_extent,
                        report.bitmap_end,
                    );
                    let bitmap_masks = row_resolution
                        .as_ref()
                        .map_or(&[][..], |rows| rows.bitmap_masks.as_slice());
                    let (descriptor_evidence, descriptor_evidence_status) =
                        build_descriptor_evidence_status(&report.column_descriptors, bitmap_masks);
                    let fixed_width = row_resolution.as_ref().map_or_else(
                        unavailable_fixed_width_diagnostic,
                        |rows| {
                            let projection = if let Some(row_data) = heap_row_data {
                                project_fixed_width_row_evidence_from_rows(
                                    &[row_data],
                                    "tc_heap_row_payload_resolved".to_string(),
                                    rows,
                                    &report.column_descriptors,
                                    bitmap_masks,
                                    report.data_region_boundaries[3] as usize,
                                )
                            } else {
                                project_fixed_width_row_evidence(
                                    payloads,
                                    report.rows_hnid,
                                    rows,
                                    &report.column_descriptors,
                                    bitmap_masks,
                                    report.data_region_boundaries[3] as usize,
                                )
                            };
                            build_fixed_width_diagnostic(projection)
                        },
                    );
                    let recipient_identity = row_resolution.as_ref().map_or_else(
                        unavailable_recipient_identity_diagnostic,
                        |rows| {
                            let projection = if let Some(row_data) = heap_row_data {
                                project_recipient_identity_strings_from_rows(
                                    &[row_data],
                                    "tc_heap_row_payload_resolved".to_string(),
                                    rows,
                                    &report.column_descriptors,
                                    bitmap_masks,
                                    &payload.bytes,
                                    payload.block_ref.offset.0,
                                    report.data_region_boundaries[3] as usize,
                                )
                            } else {
                                project_recipient_identity_strings(
                                    payloads,
                                    report.rows_hnid,
                                    rows,
                                    &report.column_descriptors,
                                    bitmap_masks,
                                    &payload.bytes,
                                    payload.block_ref.offset.0,
                                    report.data_region_boundaries[3] as usize,
                                )
                            };
                            build_recipient_identity_diagnostic(projection)
                        },
                    );
                    let complete_recipients = row_resolution.as_ref().map(|rows| {
                        if let Some(row_data) = heap_row_data {
                            project_complete_recipient_records_from_rows(
                                &[row_data],
                                "tc_heap_row_payload_resolved".to_string(),
                                rows,
                                &report.column_descriptors,
                                bitmap_masks,
                                &payload.bytes,
                                payload.block_ref.offset.0,
                                report.data_region_boundaries[3] as usize,
                                &fixed_width,
                            )
                        } else {
                            project_complete_recipient_records(
                                payloads,
                                report.rows_hnid,
                                rows,
                                &report.column_descriptors,
                                bitmap_masks,
                                &payload.bytes,
                                payload.block_ref.offset.0,
                                report.data_region_boundaries[3] as usize,
                                &fixed_width,
                            )
                        }
                    });
                    let base_status = row_resolution
                        .as_ref()
                        .map_or(report.status, |rows| rows.status.clone());
                    let status = complete_recipients
                        .as_ref()
                        .map(|projection| format!("{base_status};{}", projection.status_fragment()))
                        .unwrap_or(base_status);
                    TcHeapDiagnostic {
                        block_id: payload.block_id.0,
                        payload_byte_len: payload.bytes.len(),
                        resolved: true,
                        column_count: report.column_count,
                        row_reference_count: report.row_reference_count,
                        row_references_in_bounds: row_resolution
                            .as_ref()
                            .map_or(report.row_references_in_bounds, |rows| {
                                rows.row_references_in_bounds
                            }),
                        row_references_out_of_bounds: row_resolution
                            .as_ref()
                            .map_or(report.row_references_out_of_bounds, |rows| {
                                rows.row_references_out_of_bounds
                            }),
                        rows_require_subnode_resolution: report.rows_requires_subnode_resolution,
                        rows_nid: report.rows_hnid,
                        subnode_row_match_count: if report.rows_requires_subnode_resolution {
                            row_resolution
                                .as_ref()
                                .map_or(0, |rows| rows.matching_entry_count)
                        } else {
                            0
                        },
                        resolved_row_payload_count: row_resolution
                            .as_ref()
                            .map_or(0, |rows| rows.resolved_payload_count),
                        row_data_byte_len: row_resolution
                            .as_ref()
                            .map_or(report.row_data_byte_len, |rows| rows.row_data_byte_len),
                        row_reference_values: row_resolution.as_ref().map_or_else(
                            || row_references.clone(),
                            |rows| rows.row_references.clone(),
                        ),
                        row_spans: row_resolution
                            .as_ref()
                            .map_or_else(Vec::new, |rows| rows.row_spans.clone()),
                        inferred_row_width,
                        tcinfo_data_region_boundaries: report.data_region_boundaries,
                        max_column_extent: report.max_column_extent,
                        bitmap_byte_len: report.bitmap_byte_len,
                        bitmap_end: report.bitmap_end,
                        bitmap_rows_analyzed: row_resolution
                            .as_ref()
                            .map_or(0, |rows| rows.bitmap_rows_analyzed),
                        bitmap_set_counts: row_resolution
                            .as_ref()
                            .map_or_else(Vec::new, |rows| rows.bitmap_set_counts.clone()),
                        bitmap_unset_counts: row_resolution
                            .as_ref()
                            .map_or_else(Vec::new, |rows| rows.bitmap_unset_counts.clone()),
                        bitmap_masks: row_resolution
                            .as_ref()
                            .map_or_else(Vec::new, |rows| rows.bitmap_masks.clone()),
                        bitmap_status: row_resolution.as_ref().map_or_else(
                            || "tc_row_bitmap_payload_unavailable".to_string(),
                            |rows| rows.bitmap_status.clone(),
                        ),
                        descriptor_evidence,
                        descriptor_evidence_status,
                        row_layout_extents_valid,
                        row_layout_status,
                        fixed_width,
                        recipient_identity,
                        complete_recipients,
                        status,
                        error: report.row_index_error,
                    }
                }
                Err(reason) => TcHeapDiagnostic {
                    block_id: payload.block_id.0,
                    payload_byte_len: payload.bytes.len(),
                    resolved: false,
                    column_count: 0,
                    row_reference_count: 0,
                    row_references_in_bounds: 0,
                    row_references_out_of_bounds: 0,
                    rows_require_subnode_resolution: false,
                    rows_nid: 0,
                    subnode_row_match_count: 0,
                    resolved_row_payload_count: 0,
                    row_data_byte_len: 0,
                    row_reference_values: Vec::new(),
                    row_spans: Vec::new(),
                    inferred_row_width: 0,
                    tcinfo_data_region_boundaries: [0; 4],
                    max_column_extent: 0,
                    bitmap_byte_len: 0,
                    bitmap_end: 0,
                    bitmap_rows_analyzed: 0,
                    bitmap_set_counts: Vec::new(),
                    bitmap_unset_counts: Vec::new(),
                    bitmap_masks: Vec::new(),
                    bitmap_status: "tc_row_bitmap_payload_unavailable".to_string(),
                    descriptor_evidence: "none".to_string(),
                    descriptor_evidence_status: "tc_descriptor_evidence_unavailable".to_string(),
                    row_layout_extents_valid: false,
                    row_layout_status: "tc_row_layout_width_unavailable".to_string(),
                    fixed_width: unavailable_fixed_width_diagnostic(),
                    recipient_identity: unavailable_recipient_identity_diagnostic(),
                    complete_recipients: None,
                    status: "tc_heap_resolution_failed".to_string(),
                    error: Some(reason.to_string()),
                },
            },
        )
        .collect::<Vec<_>>();

    let table_heap_count = diagnostics.len();
    let resolved_table_heap_count = diagnostics.iter().filter(|item| item.resolved).count();
    let failed_table_heap_count = table_heap_count.saturating_sub(resolved_table_heap_count);
    let total_columns = diagnostics.iter().map(|item| item.column_count).sum();
    let total_row_references = diagnostics
        .iter()
        .map(|item| item.row_reference_count)
        .sum();
    let total_row_references_in_bounds = diagnostics
        .iter()
        .map(|item| item.row_references_in_bounds)
        .sum();
    let total_row_references_out_of_bounds = diagnostics
        .iter()
        .map(|item| item.row_references_out_of_bounds)
        .sum();
    let subnode_backed_row_heap_count = diagnostics
        .iter()
        .filter(|item| item.rows_require_subnode_resolution)
        .count();
    let status = if table_heap_count == 0 {
        "tc_heap_report_empty"
    } else if failed_table_heap_count == 0 {
        "tc_heap_report_complete"
    } else if resolved_table_heap_count > 0 {
        "tc_heap_report_partial"
    } else {
        "tc_heap_report_failed"
    };

    TcHeapAggregateReport {
        payload_count: payloads.len(),
        table_heap_count,
        resolved_table_heap_count,
        failed_table_heap_count,
        total_columns,
        total_row_references,
        total_row_references_in_bounds,
        total_row_references_out_of_bounds,
        subnode_backed_row_heap_count,
        diagnostics,
        status: status.to_string(),
    }
}

fn unavailable_fixed_width_diagnostic() -> TcFixedWidthDiagnostic {
    build_fixed_width_diagnostic(TcFixedWidthProjectionReport {
        candidate_status: "tc_row_payload_candidates_nid_missing".to_string(),
        transport_status: "tc_row_transport_unavailable".to_string(),
        evidence_status: TC_FIXED_WIDTH_EVIDENCE_UNAVAILABLE.to_string(),
        evidence: None,
        failure_reason: None,
    })
}

fn build_descriptor_evidence_status(
    columns: &[TcColumnDescriptor],
    bitmap_masks: &[String],
) -> (String, String) {
    if columns.is_empty() || bitmap_masks.is_empty() {
        return (
            "none".to_string(),
            "tc_descriptor_evidence_unavailable".to_string(),
        );
    }

    match build_descriptor_bitmap_evidence_from_columns(columns, bitmap_masks) {
        Ok(evidence) => (
            format_descriptor_bitmap_evidence(&evidence),
            "tc_descriptor_evidence_validated".to_string(),
        ),
        Err(_) => (
            "none".to_string(),
            "tc_descriptor_evidence_construction_failed".to_string(),
        ),
    }
}

fn validate_row_layout_extents(
    row_width: usize,
    data_region_boundaries: [u16; 4],
    max_column_extent: usize,
    bitmap_end: usize,
) -> (String, bool) {
    if row_width == 0 {
        return ("tc_row_layout_width_unavailable".to_string(), false);
    }
    let final_region_boundary = data_region_boundaries[3] as usize;
    if final_region_boundary > row_width || max_column_extent > row_width || bitmap_end > row_width
    {
        return ("tc_row_layout_extents_out_of_bounds".to_string(), false);
    }
    ("tc_row_layout_extents_valid".to_string(), true)
}

#[cfg(test)]
mod tests {
    use super::report_subnode_table_heaps;
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset, NodeId};
    use crate::pst::subnodes::SubnodeReference;
    use crate::pst::tc_message_recipient_selection::{
        select_message_recipient_projection, MESSAGE_RECIPIENT_PROJECTION_SELECTED,
    };

    #[test]
    fn projects_only_the_direct_heap_backed_recipient_table() {
        let reference = SubnodeReference {
            node_id: NodeId(0x2000e4),
            subnode_block_id: BlockId(0x6c6),
            status: "test".to_string(),
        };
        let payloads = vec![
            payload(0x6c6, direct_recipient_slblock(0x692, 0x600)),
            payload(0x600, heap_backed_recipient_table()),
        ];

        let probe = report_subnode_table_heaps(&reference, &payloads);

        assert_eq!(probe.direct_recipient_table_bids, [0x600]);
        let diagnostic = &probe.table_heaps.diagnostics[0];
        assert!(!diagnostic.rows_require_subnode_resolution);
        assert_eq!(diagnostic.rows_nid, 0x80);
        assert_eq!(diagnostic.inferred_row_width, 13);
        assert_eq!(diagnostic.bitmap_masks, ["111"]);
        assert_eq!(diagnostic.fixed_width.semantic_values, ["to"]);

        let selection = select_message_recipient_projection(&probe);
        assert_eq!(selection.status, MESSAGE_RECIPIENT_PROJECTION_SELECTED);
        let records = selection.complete_records.unwrap();
        assert_eq!(records.records.len(), 1);
        assert_eq!(records.records[0].display_name, "Recipient");
        assert_eq!(records.records[0].address, "recipient@example.com");
        assert_eq!(records.records[0].address_kind, "smtp_address");
    }

    fn direct_recipient_slblock(nid: u32, data_bid: u64) -> Vec<u8> {
        let mut bytes = vec![0u8; 32];
        bytes[0] = 0x02;
        bytes[2..4].copy_from_slice(&1u16.to_le_bytes());
        bytes[8..12].copy_from_slice(&nid.to_le_bytes());
        bytes[16..24].copy_from_slice(&data_bid.to_le_bytes());
        bytes
    }

    fn heap_backed_recipient_table() -> Vec<u8> {
        let mut bth_header = vec![0xb5, 4, 4, 0];
        bth_header.extend_from_slice(&0x60u32.to_le_bytes());

        let mut tcinfo = vec![0u8; 46];
        tcinfo[0] = 0x7c;
        tcinfo[1] = 3;
        for (offset, boundary) in [(2, 12u16), (4, 12), (6, 12), (8, 13)] {
            tcinfo[offset..offset + 2].copy_from_slice(&boundary.to_le_bytes());
        }
        tcinfo[10..14].copy_from_slice(&0x20u32.to_le_bytes());
        tcinfo[14..18].copy_from_slice(&0x80u32.to_le_bytes());
        for (index, (tag, data_offset, data_size)) in [
            (0x0c15_0003u32, 0u16, 4u8),
            (0x3001_001fu32, 4u16, 4u8),
            (0x39fe_001fu32, 8u16, 4u8),
        ]
        .into_iter()
        .enumerate()
        {
            let start = 22 + index * 8;
            tcinfo[start..start + 4].copy_from_slice(&tag.to_le_bytes());
            tcinfo[start + 4..start + 6].copy_from_slice(&data_offset.to_le_bytes());
            tcinfo[start + 6] = data_size;
            tcinfo[start + 7] = index as u8;
        }

        let mut row_index = Vec::new();
        row_index.extend_from_slice(&1u32.to_le_bytes());
        row_index.extend_from_slice(&0u32.to_le_bytes());

        let mut row = vec![0u8; 13];
        row[0..4].copy_from_slice(&1u32.to_le_bytes());
        row[4..8].copy_from_slice(&0xa0u32.to_le_bytes());
        row[8..12].copy_from_slice(&0xc0u32.to_le_bytes());
        row[12] = 0b0000_0111;

        let display_name = utf16("Recipient");
        let smtp_address = utf16("recipient@example.com");
        heap(vec![
            bth_header,
            tcinfo,
            row_index,
            row,
            display_name,
            smtp_address,
        ])
    }

    fn heap(allocations: Vec<Vec<u8>>) -> Vec<u8> {
        let mut offsets = vec![8u16];
        for allocation in &allocations {
            let next = usize::from(*offsets.last().unwrap()) + allocation.len();
            offsets.push(u16::try_from(next).unwrap());
        }
        let page_map_offset = *offsets.last().unwrap();
        let page_map_len = 4 + offsets.len() * 2;
        let mut bytes = vec![0u8; usize::from(page_map_offset) + page_map_len];
        bytes[0..2].copy_from_slice(&page_map_offset.to_le_bytes());
        bytes[2] = 0xec;
        bytes[3] = 0x7c;
        bytes[4..8].copy_from_slice(&0x40u32.to_le_bytes());
        for (index, allocation) in allocations.iter().enumerate() {
            let start = usize::from(offsets[index]);
            bytes[start..start + allocation.len()].copy_from_slice(allocation);
        }
        let page_map = usize::from(page_map_offset);
        bytes[page_map..page_map + 2]
            .copy_from_slice(&(allocations.len() as u16).to_le_bytes());
        for (index, offset) in offsets.iter().enumerate() {
            let start = page_map + 4 + index * 2;
            bytes[start..start + 2].copy_from_slice(&offset.to_le_bytes());
        }
        bytes
    }

    fn utf16(value: &str) -> Vec<u8> {
        value
            .encode_utf16()
            .chain(std::iter::once(0))
            .flat_map(u16::to_le_bytes)
            .collect()
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
}
