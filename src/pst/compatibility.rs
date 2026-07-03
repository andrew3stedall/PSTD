use std::collections::{BTreeMap, BTreeSet};

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecoderBacklogReviewSummary {
    pub run_id: String,
    pub pst_id: String,
    pub total_items: usize,
    pub high_priority_count: usize,
    pub medium_priority_count: usize,
    pub low_priority_count: usize,
    pub decoder_work_count: usize,
    pub payload_mapping_count: usize,
    pub unique_candidate_count: usize,
    pub top_candidate_key: Option<String>,
    pub review_status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecoderIssueCandidate {
    pub run_id: String,
    pub pst_id: String,
    pub decoder_candidate_key: String,
    pub category: String,
    pub priority: String,
    pub backlog_status: String,
    pub affected_message_count: usize,
    pub observed_total: usize,
    pub source_item_count: usize,
    pub recommended_title: String,
    pub recommended_action: String,
    pub checklist: Vec<String>,
    pub issue_status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecoderCandidateSelection {
    pub run_id: String,
    pub pst_id: String,
    pub selection_rank: usize,
    pub decoder_candidate_key: String,
    pub category: String,
    pub priority: String,
    pub affected_message_count: usize,
    pub observed_total: usize,
    pub source_item_count: usize,
    pub selection_status: String,
    pub selection_reason: String,
    pub recommended_next_step: String,
    pub implementation_scope: String,
    pub test_expectation: String,
    pub fallback_requirement: String,
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

pub fn decoder_backlog_review_summary(
    run_id: &str,
    pst_id: &str,
    items: &[DecoderBacklogItem],
) -> DecoderBacklogReviewSummary {
    let mut unique_candidate_keys = BTreeSet::new();
    let mut high_priority_count = 0usize;
    let mut medium_priority_count = 0usize;
    let mut low_priority_count = 0usize;
    let mut decoder_work_count = 0usize;
    let mut payload_mapping_count = 0usize;

    for item in items {
        unique_candidate_keys.insert(item.decoder_candidate_key.clone());
        match item.priority.as_str() {
            "high" => high_priority_count += 1,
            "medium" => medium_priority_count += 1,
            _ => low_priority_count += 1,
        }
        if item.backlog_status == "payload_mapping_backlog_open" {
            payload_mapping_count += 1;
        } else {
            decoder_work_count += 1;
        }
    }

    let top_candidate_key = items
        .iter()
        .min_by_key(|item| {
            (
                priority_rank(&item.priority),
                item.decoder_candidate_key.as_str(),
            )
        })
        .map(|item| item.decoder_candidate_key.clone());

    let review_status = if items.is_empty() {
        "decoder_backlog_review_empty"
    } else if high_priority_count > 0 {
        "decoder_backlog_review_high_priority_open"
    } else if medium_priority_count > 0 {
        "decoder_backlog_review_medium_priority_open"
    } else {
        "decoder_backlog_review_low_priority_open"
    };

    DecoderBacklogReviewSummary {
        run_id: run_id.to_string(),
        pst_id: pst_id.to_string(),
        total_items: items.len(),
        high_priority_count,
        medium_priority_count,
        low_priority_count,
        decoder_work_count,
        payload_mapping_count,
        unique_candidate_count: unique_candidate_keys.len(),
        top_candidate_key,
        review_status: review_status.to_string(),
    }
}

pub fn decoder_issue_candidates_from_backlog(
    run_id: &str,
    pst_id: &str,
    items: &[DecoderBacklogItem],
) -> Vec<DecoderIssueCandidate> {
    let mut grouped: BTreeMap<String, Vec<&DecoderBacklogItem>> = BTreeMap::new();
    for item in items {
        grouped
            .entry(item.decoder_candidate_key.clone())
            .or_default()
            .push(item);
    }

    let mut candidates = grouped
        .into_iter()
        .map(|(candidate_key, grouped_items)| {
            let first = grouped_items[0];
            let affected_message_count = grouped_items
                .iter()
                .map(|item| item.message_key.as_str())
                .collect::<BTreeSet<_>>()
                .len();
            let observed_total = grouped_items
                .iter()
                .map(|item| item.observed_count)
                .sum::<usize>();
            DecoderIssueCandidate {
                run_id: run_id.to_string(),
                pst_id: pst_id.to_string(),
                decoder_candidate_key: candidate_key,
                category: first.category.clone(),
                priority: first.priority.clone(),
                backlog_status: first.backlog_status.clone(),
                affected_message_count,
                observed_total,
                source_item_count: grouped_items.len(),
                recommended_title: recommended_issue_title(first),
                recommended_action: first.recommended_action.clone(),
                checklist: review_checklist(first),
                issue_status: "issue_candidate_ready_for_review".to_string(),
            }
        })
        .collect::<Vec<_>>();

    candidates.sort_by_key(|candidate| {
        (
            priority_rank(&candidate.priority),
            candidate.decoder_candidate_key.clone(),
        )
    });
    candidates
}

pub fn select_decoder_candidates_for_implementation(
    run_id: &str,
    pst_id: &str,
    candidates: &[DecoderIssueCandidate],
) -> Vec<DecoderCandidateSelection> {
    let mut ordered = candidates.to_vec();
    ordered.sort_by_key(|candidate| {
        (
            priority_rank(&candidate.priority),
            std::cmp::Reverse(candidate.observed_total),
            std::cmp::Reverse(candidate.affected_message_count),
            candidate.decoder_candidate_key.clone(),
        )
    });

    ordered
        .iter()
        .enumerate()
        .map(|(index, candidate)| candidate_selection(run_id, pst_id, index + 1, candidate))
        .collect()
}

fn candidate_selection(
    run_id: &str,
    pst_id: &str,
    selection_rank: usize,
    candidate: &DecoderIssueCandidate,
) -> DecoderCandidateSelection {
    DecoderCandidateSelection {
        run_id: run_id.to_string(),
        pst_id: pst_id.to_string(),
        selection_rank,
        decoder_candidate_key: candidate.decoder_candidate_key.clone(),
        category: candidate.category.clone(),
        priority: candidate.priority.clone(),
        affected_message_count: candidate.affected_message_count,
        observed_total: candidate.observed_total,
        source_item_count: candidate.source_item_count,
        selection_status: selection_status(candidate).to_string(),
        selection_reason: selection_reason(candidate),
        recommended_next_step: candidate.recommended_action.clone(),
        implementation_scope: implementation_scope(candidate),
        test_expectation: test_expectation(candidate),
        fallback_requirement: fallback_requirement(candidate),
    }
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

fn priority_rank(priority: &str) -> u8 {
    match priority {
        "high" => 0,
        "medium" => 1,
        _ => 2,
    }
}

fn selection_status(candidate: &DecoderIssueCandidate) -> &'static str {
    match candidate.priority.as_str() {
        "high" => "selected_for_next_planning",
        "medium" => "hold_for_high_priority_review",
        _ => "hold_for_more_evidence",
    }
}

fn selection_reason(candidate: &DecoderIssueCandidate) -> String {
    format!(
        "priority={}; affected_messages={}; observed_total={}; source_items={}",
        candidate.priority,
        candidate.affected_message_count,
        candidate.observed_total,
        candidate.source_item_count
    )
}

fn implementation_scope(candidate: &DecoderIssueCandidate) -> String {
    match candidate.category.as_str() {
        "unsupported_subnode_layout" =>
            "Add one narrow subnode layout parser path guarded by a focused regression test."
                .to_string(),
        "unparseable_attachment_table" =>
            "Add one attachment table parser path for the observed table shape and preserve parse-error reporting."
                .to_string(),
        "attachment_rows_without_payloads" =>
            "Add one payload mapping path for rows that already parse but do not yet produce payload bytes."
                .to_string(),
        _ => "Add one narrow compatibility improvement for this candidate key.".to_string(),
    }
}

fn test_expectation(candidate: &DecoderIssueCandidate) -> String {
    match candidate.category.as_str() {
        "unsupported_subnode_layout" =>
            "Add a synthetic or reviewed fixture-backed subnode layout test before enabling support."
                .to_string(),
        "unparseable_attachment_table" =>
            "Add a table parser regression test covering the observed parse status.".to_string(),
        "attachment_rows_without_payloads" =>
            "Add a payload mapping regression test proving bytes or explicit unavailable status.".to_string(),
        _ => "Add a regression test that proves the candidate behaviour and fallback path."
            .to_string(),
    }
}

fn fallback_requirement(candidate: &DecoderIssueCandidate) -> String {
    match candidate.backlog_status.as_str() {
        "payload_mapping_backlog_open" =>
            "Rows that still cannot map payload bytes must keep payload mapping fallback status."
                .to_string(),
        _ => "Rows that still cannot be parsed must keep explicit unsupported fallback status.".to_string(),
    }
}

fn recommended_issue_title(item: &DecoderBacklogItem) -> String {
    format!(
        "[decoder-backlog] {} ({})",
        item.category.replace('_', " "),
        item.priority
    )
}

fn review_checklist(item: &DecoderBacklogItem) -> Vec<String> {
    let mut checklist = vec![
        "Review the source decoder_backlog.jsonl rows.".to_string(),
        "Confirm the category, priority, and affected message count.".to_string(),
        "Add a focused regression test before changing parser behaviour.".to_string(),
        "Preserve fallback status for unsupported rows.".to_string(),
    ];

    match item.category.as_str() {
        "unsupported_subnode_layout" => checklist
            .push("Document the observed subnode layout before adding a decoder.".to_string()),
        "unparseable_attachment_table" => {
            checklist.push("Record parse-error offsets and reasons in the issue body.".to_string())
        }
        "attachment_rows_without_payloads" => checklist.push(
            "Trace whether payload bytes are direct, indirect, or in a child subnode.".to_string(),
        ),
        _ => checklist.push("Record the reviewed evidence before implementation.".to_string()),
    }

    checklist
}

#[cfg(test)]
mod tests {
    use super::{
        decoder_backlog_from_triage_records, decoder_backlog_review_summary,
        decoder_issue_candidates_from_backlog, select_decoder_candidates_for_implementation,
        triage_observed_attachment_layouts, CompatibilityTriageRecord,
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
        let backlog = sample_backlog();

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
        let record =
            CompatibilityTriageRecord::from_report("run_123", "pst_123", "msg_123", None, report);

        let backlog = decoder_backlog_from_triage_records(&[record]);

        assert!(backlog.is_empty());
    }

    #[test]
    fn summarizes_decoder_backlog_for_review() {
        let backlog = sample_backlog();

        let summary = decoder_backlog_review_summary("run_123", "pst_123", &backlog);

        assert_eq!(summary.total_items, 3);
        assert_eq!(summary.high_priority_count, 2);
        assert_eq!(summary.medium_priority_count, 1);
        assert_eq!(summary.low_priority_count, 0);
        assert_eq!(summary.decoder_work_count, 2);
        assert_eq!(summary.payload_mapping_count, 1);
        assert_eq!(summary.unique_candidate_count, 3);
        assert_eq!(
            summary.review_status,
            "decoder_backlog_review_high_priority_open"
        );
    }

    #[test]
    fn summarizes_empty_backlog_for_review() {
        let summary = decoder_backlog_review_summary("run_123", "pst_123", &[]);

        assert_eq!(summary.total_items, 0);
        assert_eq!(summary.top_candidate_key, None);
        assert_eq!(summary.review_status, "decoder_backlog_review_empty");
    }

    #[test]
    fn builds_issue_candidates_from_backlog() {
        let backlog = sample_backlog();

        let candidates = decoder_issue_candidates_from_backlog("run_123", "pst_123", &backlog);

        assert_eq!(candidates.len(), 3);
        assert_eq!(candidates[0].priority, "high");
        assert_eq!(
            candidates[0].issue_status,
            "issue_candidate_ready_for_review"
        );
        assert!(candidates[0]
            .checklist
            .iter()
            .any(|item| item.contains("regression test")));
        assert!(candidates
            .iter()
            .any(|candidate| candidate.backlog_status == "payload_mapping_backlog_open"));
    }

    #[test]
    fn selects_candidates_for_focused_planning() {
        let backlog = sample_backlog();
        let candidates = decoder_issue_candidates_from_backlog("run_123", "pst_123", &backlog);

        let selections =
            select_decoder_candidates_for_implementation("run_123", "pst_123", &candidates);

        assert_eq!(selections.len(), 3);
        assert_eq!(selections[0].selection_rank, 1);
        assert_eq!(selections[0].priority, "high");
        assert_eq!(selections[0].selection_status, "selected_for_next_planning");
        assert!(selections[0].test_expectation.contains("test"));
        assert!(selections[0].fallback_requirement.contains("fallback"));
        assert!(selections
            .iter()
            .any(|selection| selection.selection_status == "hold_for_high_priority_review"));
    }

    #[test]
    fn selects_no_candidates_for_empty_review() {
        let selections = select_decoder_candidates_for_implementation("run_123", "pst_123", &[]);

        assert!(selections.is_empty());
    }

    fn sample_backlog() -> Vec<super::DecoderBacklogItem> {
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
        decoder_backlog_from_triage_records(&[record])
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
