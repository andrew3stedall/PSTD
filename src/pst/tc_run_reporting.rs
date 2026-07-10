use crate::pst::tc_reporting::TcSubnodeProbeReport;

const MAX_PROBE_DIAGNOSTICS: usize = 16;

#[derive(Debug, Clone, Default)]
pub struct TcRunProbeCollector {
    probes: Vec<TcSubnodeProbeReport>,
}

impl TcRunProbeCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, probe: TcSubnodeProbeReport) {
        self.probes.push(probe);
    }

    pub fn probe_count(&self) -> usize {
        self.probes.len()
    }

    pub fn finish(self) -> TcRunAggregateReport {
        aggregate_subnode_table_probes(&self.probes)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TcRunAggregateReport {
    pub probe_count: usize,
    pub decoded_probe_count: usize,
    pub decoded_payload_count: usize,
    pub probes_with_table_heaps: usize,
    pub resolved_table_heap_count: usize,
    pub failed_table_heap_count: usize,
    pub total_columns: usize,
    pub total_row_references: usize,
    pub total_row_references_in_bounds: usize,
    pub total_row_references_out_of_bounds: usize,
    pub subnode_backed_row_heap_count: usize,
    pub diagnostic_fragments: Vec<String>,
    pub diagnostics_truncated: bool,
    pub status: String,
}

impl TcRunAggregateReport {
    pub fn progress_status(&self) -> String {
        let diagnostics = if self.diagnostic_fragments.is_empty() {
            "none".to_string()
        } else {
            self.diagnostic_fragments.join("|")
        };

        format!(
            "pq44_status={}; pq44_probes={}; pq44_decoded_probes={}; pq44_decoded_payloads={}; pq44_probes_with_table_heaps={}; pq44_resolved_table_heaps={}; pq44_failed_table_heaps={}; pq44_columns={}; pq44_row_references={}; pq44_row_references_in_bounds={}; pq44_row_references_out_of_bounds={}; pq44_subnode_backed_row_heaps={}; pq44_diagnostics_truncated={}; pq44_diagnostics={diagnostics}",
            self.status,
            self.probe_count,
            self.decoded_probe_count,
            self.decoded_payload_count,
            self.probes_with_table_heaps,
            self.resolved_table_heap_count,
            self.failed_table_heap_count,
            self.total_columns,
            self.total_row_references,
            self.total_row_references_in_bounds,
            self.total_row_references_out_of_bounds,
            self.subnode_backed_row_heap_count,
            usize::from(self.diagnostics_truncated),
        )
    }
}

pub fn aggregate_subnode_table_probes(probes: &[TcSubnodeProbeReport]) -> TcRunAggregateReport {
    let probe_count = probes.len();
    let decoded_probe_count = probes
        .iter()
        .filter(|probe| probe.decoded_payload_count > 0)
        .count();
    let decoded_payload_count = probes.iter().map(|probe| probe.decoded_payload_count).sum();
    let probes_with_table_heaps = probes
        .iter()
        .filter(|probe| probe.table_heaps.table_heap_count > 0)
        .count();
    let resolved_table_heap_count = probes
        .iter()
        .map(|probe| probe.table_heaps.resolved_table_heap_count)
        .sum();
    let failed_table_heap_count = probes
        .iter()
        .map(|probe| probe.table_heaps.failed_table_heap_count)
        .sum();
    let total_columns = probes
        .iter()
        .map(|probe| probe.table_heaps.total_columns)
        .sum();
    let total_row_references = probes
        .iter()
        .map(|probe| probe.table_heaps.total_row_references)
        .sum();
    let total_row_references_in_bounds = probes
        .iter()
        .map(|probe| probe.table_heaps.total_row_references_in_bounds)
        .sum();
    let total_row_references_out_of_bounds = probes
        .iter()
        .map(|probe| probe.table_heaps.total_row_references_out_of_bounds)
        .sum();
    let subnode_backed_row_heap_count = probes
        .iter()
        .map(|probe| probe.table_heaps.subnode_backed_row_heap_count)
        .sum();

    let eligible_diagnostics = probes
        .iter()
        .filter(|probe| probe.table_heaps.table_heap_count > 0)
        .map(TcSubnodeProbeReport::progress_status)
        .collect::<Vec<_>>();
    let diagnostics_truncated = eligible_diagnostics.len() > MAX_PROBE_DIAGNOSTICS;
    let diagnostic_fragments = eligible_diagnostics
        .into_iter()
        .take(MAX_PROBE_DIAGNOSTICS)
        .map(|status| status.replace(';', ","))
        .collect::<Vec<_>>();

    let status = if probe_count == 0 {
        "pq44_no_probes"
    } else if probes_with_table_heaps == 0 {
        "pq44_no_table_heaps"
    } else if failed_table_heap_count == 0 {
        "pq44_table_heaps_resolved"
    } else if resolved_table_heap_count > 0 {
        "pq44_table_heaps_partially_resolved"
    } else {
        "pq44_table_heaps_unresolved"
    };

    TcRunAggregateReport {
        probe_count,
        decoded_probe_count,
        decoded_payload_count,
        probes_with_table_heaps,
        resolved_table_heap_count,
        failed_table_heap_count,
        total_columns,
        total_row_references,
        total_row_references_in_bounds,
        total_row_references_out_of_bounds,
        subnode_backed_row_heap_count,
        diagnostic_fragments,
        diagnostics_truncated,
        status: status.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        aggregate_subnode_table_probes, TcRunProbeCollector, MAX_PROBE_DIAGNOSTICS,
    };
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset, NodeId};
    use crate::pst::subnodes::SubnodeReference;
    use crate::pst::tc_reporting::report_subnode_table_heaps;

    #[test]
    fn reports_empty_and_non_table_probe_runs_distinctly() {
        let empty = aggregate_subnode_table_probes(&[]);
        assert_eq!(empty.status, "pq44_no_probes");
        assert!(empty.progress_status().contains("pq44_diagnostics=none"));

        let reference = reference(1, 2);
        let non_table = report_subnode_table_heaps(&reference, &[payload(3, vec![0; 16])]);
        let report = aggregate_subnode_table_probes(&[non_table]);
        assert_eq!(report.probe_count, 1);
        assert_eq!(report.decoded_probe_count, 1);
        assert_eq!(report.decoded_payload_count, 1);
        assert_eq!(report.probes_with_table_heaps, 0);
        assert_eq!(report.status, "pq44_no_table_heaps");
    }

    #[test]
    fn collector_accepts_probes_from_separate_extraction_paths() {
        let mut collector = TcRunProbeCollector::new();
        let non_table_reference = reference(1, 2);
        collector.record(report_subnode_table_heaps(
            &non_table_reference,
            &[payload(3, vec![0; 16])],
        ));

        let failed_reference = reference(0x122, 0x244);
        let mut bytes = vec![0; 16];
        bytes[2] = 0xec;
        bytes[3] = 0x7c;
        collector.record(report_subnode_table_heaps(
            &failed_reference,
            &[payload(0x74, bytes)],
        ));

        assert_eq!(collector.probe_count(), 2);
        let report = collector.finish();
        assert_eq!(report.probe_count, 2);
        assert_eq!(report.decoded_probe_count, 2);
        assert_eq!(report.probes_with_table_heaps, 1);
        assert_eq!(report.failed_table_heap_count, 1);
        assert!(report.progress_status().contains("pq43_root_node_id=0x122"));
    }

    #[test]
    fn aggregates_failed_table_heap_evidence_with_probe_identity() {
        let reference = reference(0x122, 0x244);
        let mut bytes = vec![0; 16];
        bytes[2] = 0xec;
        bytes[3] = 0x7c;
        let probe = report_subnode_table_heaps(&reference, &[payload(0x74, bytes)]);
        let report = aggregate_subnode_table_probes(&[probe]);

        assert_eq!(report.probes_with_table_heaps, 1);
        assert_eq!(report.failed_table_heap_count, 1);
        assert_eq!(report.status, "pq44_table_heaps_unresolved");
        let status = report.progress_status();
        assert!(status.contains("pq43_root_node_id=0x122"));
        assert!(status.contains("pq43_root_subnode_bid=0x244"));
        assert!(status.contains("pq44_failed_table_heaps=1"));
    }

    #[test]
    fn caps_probe_diagnostics_without_losing_aggregate_counts() {
        let probes = (0..MAX_PROBE_DIAGNOSTICS + 3)
            .map(|index| {
                let reference = reference(index as u64 + 1, index as u64 + 100);
                let mut bytes = vec![0; 16];
                bytes[2] = 0xec;
                bytes[3] = 0x7c;
                report_subnode_table_heaps(&reference, &[payload(index as u64 + 200, bytes)])
            })
            .collect::<Vec<_>>();

        let report = aggregate_subnode_table_probes(&probes);
        assert_eq!(report.probe_count, MAX_PROBE_DIAGNOSTICS + 3);
        assert_eq!(report.failed_table_heap_count, MAX_PROBE_DIAGNOSTICS + 3);
        assert_eq!(report.diagnostic_fragments.len(), MAX_PROBE_DIAGNOSTICS);
        assert!(report.diagnostics_truncated);
        assert!(report
            .progress_status()
            .contains("pq44_diagnostics_truncated=1"));
    }

    fn reference(node_id: u64, block_id: u64) -> SubnodeReference {
        SubnodeReference {
            node_id: NodeId(node_id),
            subnode_block_id: BlockId(block_id),
            status: "test".to_string(),
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
}
