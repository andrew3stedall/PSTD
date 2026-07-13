use crate::pst::payload::PayloadBlock;
use crate::pst::subnodes::SubnodeReference;
use crate::pst::tc_descriptor_evidence::{
    build_descriptor_bitmap_evidence_from_columns, format_descriptor_bitmap_evidence,
};
use crate::pst::tc_fixed_width_diagnostic::{build_fixed_width_diagnostic, TcFixedWidthDiagnostic};
use crate::pst::tc_fixed_width_projection::{
    project_fixed_width_row_evidence, TcFixedWidthProjectionReport,
    TC_FIXED_WIDTH_EVIDENCE_UNAVAILABLE,
};
use crate::pst::tc_heap::resolve_tcinfo_from_heap;
use crate::pst::tc_recipient_identity_diagnostic::{
    build_recipient_identity_diagnostic, unavailable_recipient_identity_diagnostic,
    TcRecipientIdentityDiagnostic,
};
use crate::pst::tc_recipient_identity_projection::project_recipient_identity_strings;
use crate::pst::tc_subnode_rows::resolve_subnode_row_storage;
use crate::pst::tcinfo::TcColumnDescriptor;

const HEAP_SIGNATURE: u8 = 0xec;
const HEAP_CLIENT_TABLE_CONTEXT: u8 = 0x7c;

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
    pub table_heaps: TcHeapAggregateReport,
    pub status: String,
}

impl TcSubnodeProbeReport {
    pub fn progress_status(&self) -> String {
        format!(
            "pq43_status={}; pq43_root_node_id=0x{:x}; pq43_root_subnode_bid=0x{:x}; pq43_decoded_payloads={}; {}",
            self.status,
            self.root_node_id,
            self.root_subnode_block_id,
            self.decoded_payload_count,
            self.table_heaps.progress_status()
        )
    }
}

pub fn report_subnode_table_heaps(
    reference: &SubnodeReference,
    payloads: &[PayloadBlock],
) -> TcSubnodeProbeReport {
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
        table_heaps,
        status: status.to_string(),
    }
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
                    let subnode_rows = report.rows_requires_subnode_resolution.then(|| {
                        resolve_subnode_row_storage(
                            payloads,
                            report.rows_hnid,
                            &row_references,
                            report.column_count,
                            report.bitmap_end.saturating_sub(report.bitmap_byte_len),
                            report.bitmap_end,
                        )
                    });
                    let inferred_row_width = subnode_rows
                        .as_ref()
                        .map_or(0, |rows| rows.inferred_row_width);
                    let (row_layout_status, row_layout_extents_valid) = validate_row_layout_extents(
                        inferred_row_width,
                        report.data_region_boundaries,
                        report.max_column_extent,
                        report.bitmap_end,
                    );
                    let bitmap_masks = subnode_rows
                        .as_ref()
                        .map_or(&[][..], |rows| rows.bitmap_masks.as_slice());
                    let (descriptor_evidence, descriptor_evidence_status) =
                        build_descriptor_evidence_status(&report.column_descriptors, bitmap_masks);
                    let fixed_width = subnode_rows.as_ref().map_or_else(
                        unavailable_fixed_width_diagnostic,
                        |rows| {
                            build_fixed_width_diagnostic(project_fixed_width_row_evidence(
                                payloads,
                                report.rows_hnid,
                                rows,
                                &report.column_descriptors,
                                bitmap_masks,
                                report.data_region_boundaries[3] as usize,
                            ))
                        },
                    );
                    let recipient_identity = subnode_rows.as_ref().map_or_else(
                        unavailable_recipient_identity_diagnostic,
                        |rows| {
                            build_recipient_identity_diagnostic(project_recipient_identity_strings(
                                payloads,
                                report.rows_hnid,
                                rows,
                                &report.column_descriptors,
                                bitmap_masks,
                                &payload.bytes,
                                payload.block_ref.offset.0,
                                report.data_region_boundaries[3] as usize,
                            ))
                        },
                    );
                    TcHeapDiagnostic {
                        block_id: payload.block_id.0,
                        payload_byte_len: payload.bytes.len(),
                        resolved: true,
                        column_count: report.column_count,
                        row_reference_count: report.row_reference_count,
                        row_references_in_bounds: subnode_rows
                            .as_ref()
                            .map_or(report.row_references_in_bounds, |rows| {
                                rows.row_references_in_bounds
                            }),
                        row_references_out_of_bounds: subnode_rows
                            .as_ref()
                            .map_or(report.row_references_out_of_bounds, |rows| {
                                rows.row_references_out_of_bounds
                            }),
                        rows_require_subnode_resolution: report.rows_requires_subnode_resolution,
                        rows_nid: report.rows_hnid,
                        subnode_row_match_count: subnode_rows
                            .as_ref()
                            .map_or(0, |rows| rows.matching_entry_count),
                        resolved_row_payload_count: subnode_rows
                            .as_ref()
                            .map_or(0, |rows| rows.resolved_payload_count),
                        row_data_byte_len: subnode_rows
                            .as_ref()
                            .map_or(report.row_data_byte_len, |rows| rows.row_data_byte_len),
                        row_reference_values: subnode_rows.as_ref().map_or_else(
                            || row_references.clone(),
                            |rows| rows.row_references.clone(),
                        ),
                        row_spans: subnode_rows
                            .as_ref()
                            .map_or_else(Vec::new, |rows| rows.row_spans.clone()),
                        inferred_row_width,
                        tcinfo_data_region_boundaries: report.data_region_boundaries,
                        max_column_extent: report.max_column_extent,
                        bitmap_byte_len: report.bitmap_byte_len,
                        bitmap_end: report.bitmap_end,
                        bitmap_rows_analyzed: subnode_rows
                            .as_ref()
                            .map_or(0, |rows| rows.bitmap_rows_analyzed),
                        bitmap_set_counts: subnode_rows
                            .as_ref()
                            .map_or_else(Vec::new, |rows| rows.bitmap_set_counts.clone()),
                        bitmap_unset_counts: subnode_rows
                            .as_ref()
                            .map_or_else(Vec::new, |rows| rows.bitmap_unset_counts.clone()),
                        bitmap_masks: subnode_rows
                            .as_ref()
                            .map_or_else(Vec::new, |rows| rows.bitmap_masks.clone()),
                        bitmap_status: subnode_rows.as_ref().map_or_else(
                            || "tc_row_bitmap_payload_unavailable".to_string(),
                            |rows| rows.bitmap_status.clone(),
                        ),
                        descriptor_evidence,
                        descriptor_evidence_status,
                        row_layout_extents_valid,
                        row_layout_status,
                        fixed_width,
                        recipient_identity,
                        status: subnode_rows
                            .as_ref()
                            .map_or(report.status, |rows| rows.status.clone()),
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
