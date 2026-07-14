use crate::pst::payload::PayloadBlock;
use crate::pst::subnodes::SubnodeReference;
use crate::pst::tc_reporting::{report_subnode_table_heaps, TcSubnodeProbeReport};
use crate::pst::tc_run_reporting::TcRunProbeCollector;

/// Converts one successful recursive subnode load into an attributed table-context
/// probe, records it in the run collector, and returns the same typed report to the
/// message-level caller.
///
/// Keeping this operation in one function ensures attachment and non-attachment
/// extraction paths use identical attribution and reporting behaviour. Returning
/// the report lets the extraction path retain typed evidence instead of parsing the
/// aggregate diagnostic string later.
pub fn record_subnode_payload_probe(
    collector: &mut TcRunProbeCollector,
    reference: &SubnodeReference,
    payloads: &[PayloadBlock],
) -> TcSubnodeProbeReport {
    let probe = report_subnode_table_heaps(reference, payloads);
    collector.record(probe.clone());
    probe
}

#[cfg(test)]
mod tests {
    use super::record_subnode_payload_probe;
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset, NodeId};
    use crate::pst::subnodes::SubnodeReference;
    use crate::pst::tc_run_reporting::TcRunProbeCollector;

    #[test]
    fn records_non_table_payloads_without_false_table_counts() {
        let mut collector = TcRunProbeCollector::new();
        let reference = reference(0x122, 0x244);

        let probe =
            record_subnode_payload_probe(&mut collector, &reference, &[payload(0x74, vec![0; 16])]);

        assert_eq!(probe.root_node_id, 0x122);
        assert_eq!(probe.root_subnode_block_id, 0x244);
        assert_eq!(probe.table_heaps.table_heap_count, 0);

        let report = collector.finish();
        assert_eq!(report.probe_count, 1);
        assert_eq!(report.decoded_payload_count, 1);
        assert_eq!(report.probes_with_table_heaps, 0);
        assert_eq!(report.status, "pq44_no_table_heaps");
    }

    #[test]
    fn preserves_node_and_subnode_attribution_for_failed_table_heaps() {
        let mut collector = TcRunProbeCollector::new();
        let reference = reference(0x122, 0x244);
        let mut bytes = vec![0; 16];
        bytes[2] = 0xec;
        bytes[3] = 0x7c;

        let probe = record_subnode_payload_probe(&mut collector, &reference, &[payload(0x74, bytes)]);

        assert_eq!(probe.root_node_id, 0x122);
        assert_eq!(probe.root_subnode_block_id, 0x244);
        assert_eq!(probe.table_heaps.failed_table_heap_count, 1);

        let report = collector.finish();
        assert_eq!(report.probes_with_table_heaps, 1);
        assert_eq!(report.failed_table_heap_count, 1);
        let status = report.progress_status();
        assert!(status.contains("pq43_root_node_id=0x122"));
        assert!(status.contains("pq43_root_subnode_bid=0x244"));
    }

    #[test]
    fn returned_probe_matches_the_collector_aggregate() {
        let mut collector = TcRunProbeCollector::new();
        let reference = reference(0x122, 0x244);
        let mut bytes = vec![0; 16];
        bytes[2] = 0xec;
        bytes[3] = 0x7c;

        let probe = record_subnode_payload_probe(&mut collector, &reference, &[payload(0x74, bytes)]);
        let aggregate = collector.finish();

        assert_eq!(probe.decoded_payload_count, aggregate.decoded_payload_count);
        assert_eq!(
            probe.table_heaps.table_heap_count,
            aggregate.resolved_table_heap_count + aggregate.failed_table_heap_count
        );
        assert_eq!(
            probe.table_heaps.failed_table_heap_count,
            aggregate.failed_table_heap_count
        );
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
