use crate::error::StatusRecord;
use crate::pst::tc_run_reporting::{TcRunAggregateReport, TcRunProbeCollector};

#[derive(Debug, Clone)]
pub struct TcExtractionProbeSummary {
    pub report: TcRunAggregateReport,
    pub issue: Option<StatusRecord>,
}

impl TcExtractionProbeSummary {
    pub fn progress_status(&self) -> String {
        self.report.progress_status()
    }
}

pub fn finalize_table_probe_collection(
    run_id: &str,
    collector: TcRunProbeCollector,
) -> TcExtractionProbeSummary {
    let report = collector.finish();
    let issue = (report.probes_with_table_heaps > 0).then(|| {
        StatusRecord::info(
            run_id,
            "pq47_table_probe_evidence",
            format!("Table-context probe evidence: {}", report.progress_status()),
        )
    });

    TcExtractionProbeSummary { report, issue }
}

#[cfg(test)]
mod tests {
    use super::finalize_table_probe_collection;
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset, NodeId};
    use crate::pst::subnodes::SubnodeReference;
    use crate::pst::tc_probe_collection::record_subnode_payload_probe;
    use crate::pst::tc_run_reporting::TcRunProbeCollector;

    #[test]
    fn omits_issue_when_no_table_heap_is_observed() {
        let mut collector = TcRunProbeCollector::new();
        record_subnode_payload_probe(&mut collector, &reference(1, 2), &[payload(3, vec![0; 16])]);

        let summary = finalize_table_probe_collection("run-1", collector);
        assert_eq!(summary.report.probe_count, 1);
        assert_eq!(summary.report.probes_with_table_heaps, 0);
        assert!(summary.issue.is_none());
        assert!(summary
            .progress_status()
            .contains("pq44_probes_with_table_heaps=0"));
    }

    #[test]
    fn emits_bounded_issue_when_table_heap_evidence_exists() {
        let mut collector = TcRunProbeCollector::new();
        let mut bytes = vec![0; 16];
        bytes[2] = 0xec;
        bytes[3] = 0x7c;
        record_subnode_payload_probe(
            &mut collector,
            &reference(0x122, 0x244),
            &[payload(0x74, bytes)],
        );

        let summary = finalize_table_probe_collection("run-2", collector);
        assert_eq!(summary.report.failed_table_heap_count, 1);
        let issue = summary.issue.expect("table evidence issue");
        assert_eq!(issue.code, "pq47_table_probe_evidence");
        assert!(issue.message.contains("pq43_root_node_id=0x122"));
        assert!(issue.message.contains("pq43_root_subnode_bid=0x244"));
        assert!(issue.message.contains("pq44_failed_table_heaps=1"));
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
