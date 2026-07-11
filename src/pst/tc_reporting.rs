use crate::pst::payload::PayloadBlock;
use crate::pst::subnodes::SubnodeReference;
use crate::pst::tc_heap::resolve_tcinfo_from_heap;
use crate::pst::tc_subnode_rows::resolve_subnode_row_storage;

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
    pub status: String,
    pub error: Option<String>,
}

impl TcHeapDiagnostic {
    fn status_fragment(&self) -> String {
        let error = self.error.as_deref().unwrap_or("none").replace(';', ",");
        format!(
            "bid=0x{:x},bytes={},resolved={},columns={},row_refs={},in_bounds={},out_of_bounds={},subnode_rows={},rows_nid=0x{:x},row_matches={},row_payloads={},row_bytes={},status={},error={}",
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
            self.status.replace(';', ","),
            error
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
                        resolve_subnode_row_storage(payloads, report.rows_hnid, &row_references)
                    });
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

#[cfg(test)]
mod tests {
    use super::{report_subnode_table_heaps, report_table_heaps};
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset, NodeId};
    use crate::pst::subnodes::SubnodeReference;

    #[test]
    fn ignores_non_table_payloads() {
        let report = report_table_heaps(&[payload(1, vec![0; 16])]);
        assert_eq!(report.payload_count, 1);
        assert_eq!(report.table_heap_count, 0);
        assert_eq!(report.status, "tc_heap_report_empty");
        assert!(report.progress_status().contains("pq42_table_heaps=0"));
        assert!(report.progress_status().contains("pq42_diagnostics=none"));
    }

    #[test]
    fn preserves_exact_failure_evidence_for_table_heaps() {
        let mut bytes = vec![0; 16];
        bytes[2] = 0xec;
        bytes[3] = 0x7c;
        let report = report_table_heaps(&[payload(0x74, bytes)]);

        assert_eq!(report.table_heap_count, 1);
        assert_eq!(report.resolved_table_heap_count, 0);
        assert_eq!(report.failed_table_heap_count, 1);
        assert_eq!(report.diagnostics[0].block_id, 0x74);
        assert!(report.diagnostics[0].error.is_some());
        assert_eq!(report.status, "tc_heap_report_failed");
        let status = report.progress_status();
        assert!(status.contains("pq42_failed_table_heaps=1"));
        assert!(status.contains("bid=0x74,bytes=16,resolved=0"));
        assert!(!status.contains(";error="));
    }

    #[test]
    fn binds_table_heap_evidence_to_the_subnode_probe_identity() {
        let reference = SubnodeReference {
            node_id: NodeId(0x122),
            subnode_block_id: BlockId(0x244),
            status: "test".to_string(),
        };
        let mut bytes = vec![0; 16];
        bytes[2] = 0xec;
        bytes[3] = 0x7c;
        let report = report_subnode_table_heaps(&reference, &[payload(0x74, bytes)]);

        assert_eq!(report.root_node_id, 0x122);
        assert_eq!(report.root_subnode_block_id, 0x244);
        assert_eq!(report.decoded_payload_count, 1);
        assert_eq!(report.table_heaps.failed_table_heap_count, 1);
        assert_eq!(report.status, "pq43_table_heaps_unresolved");
        let status = report.progress_status();
        assert!(status.contains("pq43_root_node_id=0x122"));
        assert!(status.contains("pq43_root_subnode_bid=0x244"));
        assert!(status.contains("pq42_failed_table_heaps=1"));
    }

    #[test]
    fn distinguishes_decoded_non_table_payloads_from_empty_probes() {
        let reference = SubnodeReference {
            node_id: NodeId(1),
            subnode_block_id: BlockId(2),
            status: "test".to_string(),
        };

        let empty = report_subnode_table_heaps(&reference, &[]);
        assert_eq!(empty.status, "pq43_subnode_payloads_empty");

        let non_table = report_subnode_table_heaps(&reference, &[payload(3, vec![0; 16])]);
        assert_eq!(non_table.status, "pq43_no_table_heaps_detected");
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
