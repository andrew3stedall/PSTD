use crate::pst::attachment_table::AttachmentSubnodeWiringReport;
use crate::pst::subnodes::SubnodeLayoutReport;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LayoutCompatibilityCase {
    pub category: String,
    pub observed_count: usize,
    pub severity: String,
    pub recommended_follow_up: String,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObservedLayoutTriageReport {
    pub observed_layout_count: usize,
    pub supported_layout_count: usize,
    pub partial_layout_count: usize,
    pub unsupported_layout_count: usize,
    pub fixture_backed_decoder_count: usize,
    pub attachment_table_parse_error_count: usize,
    pub missing_payload_count: usize,
    pub follow_up_issue_count: usize,
    pub cases: Vec<LayoutCompatibilityCase>,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompatibilityTriageRecord {
    pub run_id: String,
    pub pst_id: String,
    pub message_key: String,
    pub message_node_id: Option<String>,
    pub observed_layout_count: usize,
    pub supported_layout_count: usize,
    pub partial_layout_count: usize,
    pub unsupported_layout_count: usize,
    pub fixture_backed_decoder_count: usize,
    pub attachment_table_parse_error_count: usize,
    pub missing_payload_count: usize,
    pub follow_up_issue_count: usize,
    pub cases: Vec<LayoutCompatibilityCase>,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecoderBacklogItem {
    pub run_id: String,
    pub pst_id: String,
    pub message_key: String,
    pub message_node_id: Option<String>,
    pub decoder_candidate_key: String,
    pub category: String,
    pub priority: String,
    pub severity: String,
    pub observed_count: usize,
    pub source_triage_status: String,
    pub recommended_action: String,
    pub backlog_status: String,
}

impl CompatibilityTriageRecord {
    pub fn from_report(
        run_id: &str,
        pst_id: &str,
        message_key: &str,
        message_node_id: Option<String>,
        report: ObservedLayoutTriageReport,
    ) -> Self {
        Self {
            run_id: run_id.to_string(),
            pst_id: pst_id.to_string(),
            message_key: message_key.to_string(),
            message_node_id,
            observed_layout_count: report.observed_layout_count,
            supported_layout_count: report.supported_layout_count,
            partial_layout_count: report.partial_layout_count,
            unsupported_layout_count: report.unsupported_layout_count,
            fixture_backed_decoder_count: report.fixture_backed_decoder_count,
            attachment_table_parse_error_count: report.attachment_table_parse_error_count,
            missing_payload_count: report.missing_payload_count,
            follow_up_issue_count: report.follow_up_issue_count,
            cases: report.cases,
            status: report.status,
        }
    }
}

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
        fixture_backed_decoder_count: compact_decoder_count,
        attachment_table_parse_error_count: attachment_report.parse_error_count,
        missing_payload_count: attachment_report.missing_payload_count,
        follow_up_issue_count,
        cases,
        status: status.to_string(),
    }
}

pub fn decoder_backlog_from_triage_records(
    records: &[CompatibilityTriageRecord],
) -> Vec<DecoderBacklogItem> {
    let mut items = Vec::new();

    for record in records {
        for case in &record.cases {
            if case.severity == "supported" {
                continue;
            }

            items.push(DecoderBacklogItem {
                run_id: record.run_id.clone(),
                pst_id: record.pst_id.clone(),
                message_key: record.message_key.clone(),
                message_node_id: record.message_node_id.clone(),
                decoder_candidate_key: decoder_candidate_key(&case.category, &case.status),
                category: case.category.clone(),
                priority: backlog_priority(&case.category, &case.severity).to_string(),
                severity: case.severity.clone(),
                observed_count: case.observed_count,
                source_triage_status: record.status.clone(),
                recommended_action: case.recommended_follow_up.clone(),
                backlog_status: backlog_status(&case.severity).to_string(),
            });
        }
    }

    items
}

fn decoder_candidate_key(category: &str, status: &str) -> String {
    format!("{category}:{status}")
}

fn backlog_priority(category: &str, severity: &str) -> &'static str {
    match (category, severity) {
        ("unsupported_subnode_layout", _) => "high",
        ("unparseable_attachment_table", _) => "high",
        ("attachment_rows_without_payloads", _) => "medium",
        (_, "partial") => "medium",
        _ => "low",
    }
}

fn backlog_status(severity: &str) -> &'static str {
    if severity == "partial" {
        "payload_mapping_backlog_open"
    } else {
        "decoder_backlog_open"
    }
}

#[cfg(test)]
mod tests {
    use super::{
        decoder_backlog_from_triage_records, triage_observed_attachment_layouts,
        CompatibilityTriageRecord,
    };
    use crate::pst::attachment_table::AttachmentSubnodeWiringReport;
    use crate::pst::primitives::{BlockId, ByteOffset};
    use crate::pst::subnodes::{SubnodeBlockLayout, SubnodeLayoutReport};

    #[test]
    fn triages_supported_layouts() {
        let layout_report = layout_report(2, 1, 1, 0);
        let attachment_report = attachment_report(2, 0, 0, Vec::new());

        let triage = triage_observed_attachment_layouts(&layout_report, &attachment_report);
        assert_eq!(triage.status, "observed_layouts_supported");
        assert_eq!(triage.supported_layout_count, 2);
        assert_eq!(triage.follow_up_issue_count, 0);
        assert_eq!(triage.cases.len(), 2);
    }

    #[test]
    fn triages_fixture_backed_compact_decoder_hits() {
        let layout_report = layout_report(1, 0, 0, 0);
        let attachment_report = attachment_report(
            1,
            0,
            0,
            vec!["compact_attachment_table_payloads_wired".to_string()],
        );

        let triage = triage_observed_attachment_layouts(&layout_report, &attachment_report);
        assert_eq!(triage.status, "observed_layouts_supported");
        assert_eq!(triage.fixture_backed_decoder_count, 1);
        assert_eq!(triage.supported_layout_count, 1);
        assert!(triage
            .cases
            .iter()
            .any(|case| case.category == "compact_attachment_table_layout"));
    }

    #[test]
    fn triages_unsupported_layouts_and_parse_errors() {
        let layout_report = layout_report(3, 1, 0, 2);
        let attachment_report = attachment_report(3, 1, 0, Vec::new());

        let triage = triage_observed_attachment_layouts(&layout_report, &attachment_report);
        assert_eq!(triage.status, "observed_layouts_need_parser_triage");
        assert_eq!(triage.unsupported_layout_count, 3);
        assert_eq!(triage.attachment_table_parse_error_count, 1);
        assert_eq!(triage.follow_up_issue_count, 2);
        assert!(triage
            .cases
            .iter()
            .any(|case| case.category == "unsupported_subnode_layout"));
        assert!(triage
            .cases
            .iter()
            .any(|case| case.category == "unparseable_attachment_table"));
    }

    #[test]
    fn triages_missing_payloads_as_partial() {
        let layout_report = layout_report(1, 1, 0, 0);
        let attachment_report = attachment_report(1, 0, 2, Vec::new());

        let triage = triage_observed_attachment_layouts(&layout_report, &attachment_report);
        assert_eq!(triage.status, "observed_layouts_need_payload_triage");
        assert_eq!(triage.partial_layout_count, 2);
        assert_eq!(triage.missing_payload_count, 2);
        assert_eq!(triage.follow_up_issue_count, 1);
    }

    #[test]
    fn triages_empty_reports() {
        let layout_report = layout_report(0, 0, 0, 0);
        let attachment_report = attachment_report(0, 0, 0, Vec::new());

        let triage = triage_observed_attachment_layouts(&layout_report, &attachment_report);
        assert_eq!(triage.status, "observed_layouts_empty");
        assert_eq!(triage.cases.len(), 0);
    }

    #[test]
    fn builds_machine_readable_triage_record() {
        let layout_report = layout_report(1, 1, 0, 0);
        let attachment_report = attachment_report(1, 0, 0, Vec::new());
        let report = triage_observed_attachment_layouts(&layout_report, &attachment_report);

        let record = CompatibilityTriageRecord::from_report(
            "run_123",
            "pst_123",
            "msg_123",
            Some("node_1".to_string()),
            report,
        );

        assert_eq!(record.run_id, "run_123");
        assert_eq!(record.pst_id, "pst_123");
        assert_eq!(record.message_key, "msg_123");
        assert_eq!(record.message_node_id.as_deref(), Some("node_1"));
        assert_eq!(record.status, "observed_layouts_supported");
        assert_eq!(record.cases.len(), 1);
    }

    #[test]
    fn builds_decoder_backlog_from_non_supported_cases() {
        let layout_report = layout_report(3, 1, 0, 2);
        let attachment_report = attachment_report(3, 1, 2, Vec::new());
        let report = triage_observed_attachment_layouts(&layout_report, &attachment_report);
        let record = CompatibilityTriageRecord::from_report(
            "run_123",
            "pst_123",
            "msg_123",
            Some("node_1".to_string()),
            report,
        );

        let backlog = decoder_backlog_from_triage_records(&[record]);

        assert_eq!(backlog.len(), 3);
        assert!(backlog
            .iter()
            .any(|item| item.category == "unsupported_subnode_layout"
                && item.priority == "high"
                && item.backlog_status == "decoder_backlog_open"));
        assert!(backlog
            .iter()
            .any(|item| item.category == "unparseable_attachment_table"
                && item.priority == "high"
                && item.backlog_status == "decoder_backlog_open"));
        assert!(backlog
            .iter()
            .any(|item| item.category == "attachment_rows_without_payloads"
                && item.priority == "medium"
                && item.backlog_status == "payload_mapping_backlog_open"));
    }

    #[test]
    fn skips_supported_cases_in_decoder_backlog() {
        let layout_report = layout_report(2, 1, 1, 0);
        let attachment_report = attachment_report(2, 0, 0, Vec::new());
        let report = triage_observed_attachment_layouts(&layout_report, &attachment_report);
        let record = CompatibilityTriageRecord::from_report("run_123", "pst_123", "msg_123", None, report);

        let backlog = decoder_backlog_from_triage_records(&[record]);

        assert!(backlog.is_empty());
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
