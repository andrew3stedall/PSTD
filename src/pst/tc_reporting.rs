use crate::pst::payload::PayloadBlock;
use crate::pst::tc_heap::resolve_tcinfo_from_heap;

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
    pub status: String,
    pub error: Option<String>,
}

impl TcHeapDiagnostic {
    fn status_fragment(&self) -> String {
        let error = self.error.as_deref().unwrap_or("none").replace(';', ",");
        format!(
            "bid=0x{:x},bytes={},resolved={},columns={},row_refs={},in_bounds={},out_of_bounds={},subnode_rows={},status={},error={}",
            self.block_id,
            self.payload_byte_len,
            usize::from(self.resolved),
            self.column_count,
            self.row_reference_count,
            self.row_references_in_bounds,
            self.row_references_out_of_bounds,
            usize::from(self.rows_require_subnode_resolution),
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
                Ok(report) => TcHeapDiagnostic {
                    block_id: payload.block_id.0,
                    payload_byte_len: payload.bytes.len(),
                    resolved: true,
                    column_count: report.column_count,
                    row_reference_count: report.row_reference_count,
                    row_references_in_bounds: report.row_references_in_bounds,
                    row_references_out_of_bounds: report.row_references_out_of_bounds,
                    rows_require_subnode_resolution: report.rows_requires_subnode_resolution,
                    status: report.status,
                    error: report.row_index_error,
                },
                Err(reason) => TcHeapDiagnostic {
                    block_id: payload.block_id.0,
                    payload_byte_len: payload.bytes.len(),
                    resolved: false,
                    column_count: 0,
                    row_reference_count: 0,
                    row_references_in_bounds: 0,
                    row_references_out_of_bounds: 0,
                    rows_require_subnode_resolution: false,
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
    use super::report_table_heaps;
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset};

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
