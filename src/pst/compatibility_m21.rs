#[allow(dead_code)]
#[path = "compatibility.rs"]
mod legacy;

use crate::pst::attachment_table::AttachmentSubnodeWiringReport;
use crate::pst::subnodes::SubnodeLayoutReport;

pub use legacy::{
    decoder_backlog_from_triage_records, decoder_backlog_review_summary,
    decoder_issue_candidates_from_backlog, select_decoder_candidates_for_implementation,
    CompatibilityTriageRecord, DecoderBacklogItem, DecoderBacklogReviewSummary,
    DecoderCandidateSelection, DecoderIssueCandidate, LayoutCompatibilityCase,
    ObservedLayoutTriageReport,
};

pub fn triage_observed_attachment_layouts(
    layout_report: &SubnodeLayoutReport,
    attachment_report: &AttachmentSubnodeWiringReport,
) -> ObservedLayoutTriageReport {
    let mut cases = Vec::new();
    let mut supported_layout_count = 0usize;
    let mut partial_layout_count = 0usize;
    let mut unsupported_layout_count = 0usize;
    let compact_decoder_count = attachment_report
        .table_statuses
        .iter()
        .filter(|status| status.starts_with("compact_attachment_table_"))
        .count();
    let utf16_compact_decoder_count = attachment_report
        .table_statuses
        .iter()
        .filter(|status| status.starts_with("utf16_compact_attachment_table_"))
        .count();
    let fixture_backed_decoder_count = compact_decoder_count + utf16_compact_decoder_count;

    if layout_report.table_layout_count > 0 {
        supported_layout_count += layout_report.table_layout_count;
        cases.push(LayoutCompatibilityCase {
            category: "table_context_layout".to_string(),
            observed_count: layout_report.table_layout_count,
            severity: "supported".to_string(),
            recommended_follow_up: "Keep fixture coverage for this table layout.".to_string(),
            status: "layout_supported".to_string(),
        });
    }

    if compact_decoder_count > 0 {
        supported_layout_count += compact_decoder_count;
        cases.push(LayoutCompatibilityCase {
            category: "compact_attachment_table_layout".to_string(),
            observed_count: compact_decoder_count,
            severity: "supported".to_string(),
            recommended_follow_up:
                "Keep compact attachment table regression coverage before extending this decoder."
                    .to_string(),
            status: "fixture_backed_decoder_supported".to_string(),
        });
    }

    if utf16_compact_decoder_count > 0 {
        supported_layout_count += utf16_compact_decoder_count;
        cases.push(LayoutCompatibilityCase {
            category: "utf16_compact_attachment_table_layout".to_string(),
            observed_count: utf16_compact_decoder_count,
            severity: "supported".to_string(),
            recommended_follow_up:
                "Keep UTF-16 compact attachment table regression coverage before extending this decoder."
                    .to_string(),
            status: "fixture_backed_decoder_supported".to_string(),
        });
    }

    if layout_report.child_reference_layout_count > 0 {
        supported_layout_count += layout_report.child_reference_layout_count;
        cases.push(LayoutCompatibilityCase {
            category: "known_child_reference_layout".to_string(),
            observed_count: layout_report.child_reference_layout_count,
            severity: "supported".to_string(),
            recommended_follow_up:
                "Keep recursive depth and duplicate-guard coverage for this layout.".to_string(),
            status: "layout_supported".to_string(),
        });
    }

    if layout_report.unsupported_layout_count > 0 {
        unsupported_layout_count += layout_report.unsupported_layout_count;
        cases.push(LayoutCompatibilityCase {
            category: "unsupported_subnode_layout".to_string(),
            observed_count: layout_report.unsupported_layout_count,
            severity: "needs_parser_work".to_string(),
            recommended_follow_up: "Capture a focused fixture fingerprint and add a decoder test before expanding parsing."
                .to_string(),
            status: "layout_needs_triage".to_string(),
        });
    }

    if attachment_report.parse_error_count > 0 {
        unsupported_layout_count += attachment_report.parse_error_count;
        cases.push(LayoutCompatibilityCase {
            category: "unparseable_attachment_table".to_string(),
            observed_count: attachment_report.parse_error_count,
            severity: "needs_parser_work".to_string(),
            recommended_follow_up: "Record parse-error offsets and reasons, then add a fixture-backed table parser test."
                .to_string(),
            status: "table_needs_triage".to_string(),
        });
    }

    if attachment_report.missing_payload_count > 0 {
        partial_layout_count += attachment_report.missing_payload_count;
        cases.push(LayoutCompatibilityCase {
            category: "attachment_rows_without_payloads".to_string(),
            observed_count: attachment_report.missing_payload_count,
            severity: "partial".to_string(),
            recommended_follow_up:
                "Confirm whether payload bytes are absent, indirect, or stored in a child subnode."
                    .to_string(),
            status: "payload_mapping_needs_triage".to_string(),
        });
    }

    let follow_up_issue_count = cases
        .iter()
        .filter(|case| case.severity != "supported")
        .count();

    let status = if layout_report.block_count == 0 && attachment_report.subnode_block_count == 0 {
        "observed_layouts_empty"
    } else if unsupported_layout_count > 0 {
        "observed_layouts_need_parser_triage"
    } else if partial_layout_count > 0 {
        "observed_layouts_need_payload_triage"
    } else {
        "observed_layouts_supported"
    };

    ObservedLayoutTriageReport {
        observed_layout_count: layout_report.block_count,
        supported_layout_count,
        partial_layout_count,
        unsupported_layout_count,
        fixture_backed_decoder_count,
        attachment_table_parse_error_count: attachment_report.parse_error_count,
        missing_payload_count: attachment_report.missing_payload_count,
        follow_up_issue_count,
        cases,
        status: status.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::triage_observed_attachment_layouts;
    use crate::pst::attachment_table::AttachmentSubnodeWiringReport;
    use crate::pst::primitives::{BlockId, ByteOffset};
    use crate::pst::subnodes::{SubnodeBlockLayout, SubnodeLayoutReport};

    #[test]
    fn triages_utf16_compact_decoder_hits_as_fixture_backed_evidence() {
        let layout_report = layout_report(1, 0, 0, 0);
        let attachment_report = attachment_report(
            1,
            0,
            0,
            vec!["utf16_compact_attachment_table_payloads_wired".to_string()],
        );

        let triage = triage_observed_attachment_layouts(&layout_report, &attachment_report);

        assert_eq!(triage.status, "observed_layouts_supported");
        assert_eq!(triage.fixture_backed_decoder_count, 1);
        assert_eq!(triage.supported_layout_count, 1);
        assert_eq!(triage.follow_up_issue_count, 0);
        assert!(triage.cases.iter().any(|case| {
            case.category == "utf16_compact_attachment_table_layout"
                && case.status == "fixture_backed_decoder_supported"
                && case.severity == "supported"
        }));
    }

    #[test]
    fn triages_compact_and_utf16_decoder_hits_separately() {
        let layout_report = layout_report(2, 0, 0, 0);
        let attachment_report = attachment_report(
            2,
            0,
            0,
            vec![
                "compact_attachment_table_payloads_wired".to_string(),
                "utf16_compact_attachment_table_payloads_wired".to_string(),
            ],
        );

        let triage = triage_observed_attachment_layouts(&layout_report, &attachment_report);

        assert_eq!(triage.status, "observed_layouts_supported");
        assert_eq!(triage.fixture_backed_decoder_count, 2);
        assert_eq!(triage.supported_layout_count, 2);
        assert_eq!(triage.cases.len(), 2);
        assert!(triage
            .cases
            .iter()
            .any(|case| case.category == "compact_attachment_table_layout"));
        assert!(triage
            .cases
            .iter()
            .any(|case| case.category == "utf16_compact_attachment_table_layout"));
    }

    fn layout_report(
        block_count: usize,
        table_layout_count: usize,
        child_reference_layout_count: usize,
        unsupported_layout_count: usize,
    ) -> SubnodeLayoutReport {
        SubnodeLayoutReport {
            block_count,
            table_layout_count,
            child_reference_layout_count,
            unsupported_layout_count,
            child_reference_count: child_reference_layout_count,
            layouts: (0..block_count)
                .map(|index| SubnodeBlockLayout {
                    block_id: BlockId(index as u64),
                    offset: ByteOffset(index as u64).0,
                    size: 16,
                    byte_len: 16,
                    layout_kind: "test".to_string(),
                    child_block_ids: Vec::new(),
                    status: "test".to_string(),
                })
                .collect(),
            status: "test".to_string(),
        }
    }

    fn attachment_report(
        subnode_block_count: usize,
        parse_error_count: usize,
        missing_payload_count: usize,
        table_statuses: Vec<String>,
    ) -> AttachmentSubnodeWiringReport {
        AttachmentSubnodeWiringReport {
            subnode_block_count,
            parsed_table_count: subnode_block_count.saturating_sub(parse_error_count),
            parse_error_count,
            row_count: subnode_block_count,
            payload_count: subnode_block_count.saturating_sub(missing_payload_count),
            missing_payload_count,
            parse_error_offsets: Vec::new(),
            parse_error_reasons: Vec::new(),
            table_statuses,
            status: "test".to_string(),
        }
    }
}
