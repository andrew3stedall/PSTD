use super::manifest::EvidenceStatus;
use super::normalize::{NormalizedOutput, NormalizedRecord};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonClass {
    Parity,
    PstdExtension,
    Unsupported,
    Failure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComparisonFinding {
    pub scope: String,
    pub class: ComparisonClass,
    pub reason_code: String,
    pub detail: String,
    pub left_identity: Option<String>,
    pub right_identity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComparisonSummary {
    pub class: ComparisonClass,
    pub matched_records: u64,
    pub left_records: u64,
    pub right_records: u64,
    pub findings: Vec<ComparisonFinding>,
}

impl ComparisonSummary {
    pub fn deterministic(&mut self) {
        self.findings.sort_by(|left, right| {
            (
                left.scope.as_str(),
                left.reason_code.as_str(),
                left.left_identity.as_deref().unwrap_or_default(),
                left.right_identity.as_deref().unwrap_or_default(),
            )
                .cmp(&(
                    right.scope.as_str(),
                    right.reason_code.as_str(),
                    right.left_identity.as_deref().unwrap_or_default(),
                    right.right_identity.as_deref().unwrap_or_default(),
                ))
        });
    }
}

pub fn compare_outputs(left: &NormalizedOutput, right: &NormalizedOutput) -> ComparisonSummary {
    let left_records = comparable_records(left);
    let right_records = comparable_records(right);
    let mut findings = Vec::new();
    let mut matched = 0_u64;
    let mut used_left = BTreeSet::new();
    let left_by_key = index_records(&left_records);

    for right_record in &right_records {
        let key = semantic_key(right_record);
        let Some(candidates) = left_by_key.get(&key) else {
            findings.push(ComparisonFinding {
                scope: "record".to_string(),
                class: ComparisonClass::PstdExtension,
                reason_code: "pstd_record_not_in_readpst".to_string(),
                detail: format!("no readpst record matched semantic key {key}"),
                left_identity: None,
                right_identity: Some(right_record.identity.clone()),
            });
            continue;
        };
        let candidate = candidates
            .iter()
            .find(|record| !used_left.contains(&record.identity))
            .copied();
        let Some(left_record) = candidate else {
            findings.push(ComparisonFinding {
                scope: "record".to_string(),
                class: ComparisonClass::PstdExtension,
                reason_code: "pstd_duplicate_semantic_record".to_string(),
                detail: format!("all readpst records for semantic key {key} were already matched"),
                left_identity: None,
                right_identity: Some(right_record.identity.clone()),
            });
            continue;
        };
        used_left.insert(left_record.identity.clone());
        matched += 1;
        compare_record(left_record, right_record, &mut findings);
    }

    for record in &left_records {
        if !used_left.contains(&record.identity) {
            findings.push(ComparisonFinding {
                scope: "record".to_string(),
                class: ComparisonClass::Failure,
                reason_code: "readpst_record_missing_from_pstd".to_string(),
                detail: format!("readpst record {} was not represented by PSTD", record.identity),
                left_identity: Some(record.identity.clone()),
                right_identity: None,
            });
        }
    }

    if left.status == EvidenceStatus::Failed || right.status == EvidenceStatus::Failed {
        findings.push(ComparisonFinding {
            scope: "execution".to_string(),
            class: ComparisonClass::Failure,
            reason_code: "tool_execution_failed".to_string(),
            detail: "one or both tool executions failed".to_string(),
            left_identity: None,
            right_identity: None,
        });
    } else if left_records.is_empty() && right_records.is_empty() {
        findings.push(ComparisonFinding {
            scope: "execution".to_string(),
            class: ComparisonClass::Failure,
            reason_code: "both_outputs_empty".to_string(),
            detail: "empty outputs cannot establish semantic parity".to_string(),
            left_identity: None,
            right_identity: None,
        });
    }

    let class = if findings
        .iter()
        .any(|finding| finding.class == ComparisonClass::Failure)
    {
        ComparisonClass::Failure
    } else if findings
        .iter()
        .any(|finding| finding.class == ComparisonClass::PstdExtension)
    {
        ComparisonClass::PstdExtension
    } else if left.status == EvidenceStatus::Unsupported
        || right.status == EvidenceStatus::Unsupported
    {
        ComparisonClass::Unsupported
    } else {
        ComparisonClass::Parity
    };

    let mut summary = ComparisonSummary {
        class,
        matched_records: matched,
        left_records: left_records.len() as u64,
        right_records: right_records.len() as u64,
        findings,
    };
    summary.deterministic();
    summary
}

pub fn findings_to_outcomes(
    summary: &ComparisonSummary,
) -> Vec<(String, EvidenceStatus, EvidenceStatus, String, String)> {
    let mut outcomes = Vec::new();
    if summary.findings.is_empty() {
        outcomes.push((
            "semantic_common_records".to_string(),
            EvidenceStatus::Present,
            EvidenceStatus::Present,
            "semantic_match".to_string(),
            "all normalized records matched".to_string(),
        ));
    } else {
        for (index, finding) in summary.findings.iter().enumerate() {
            let observed = match finding.class {
                ComparisonClass::Parity | ComparisonClass::PstdExtension => EvidenceStatus::Present,
                ComparisonClass::Unsupported => EvidenceStatus::Unsupported,
                ComparisonClass::Failure => EvidenceStatus::Failed,
            };
            outcomes.push((
                format!("finding_{index:04}"),
                EvidenceStatus::Present,
                observed,
                finding.reason_code.clone(),
                finding.detail.clone(),
            ));
        }
    }
    outcomes
}

fn comparable_records(output: &NormalizedOutput) -> Vec<&NormalizedRecord> {
    output
        .records
        .iter()
        .filter(|record| {
            matches!(
                record.kind.as_str(),
                "message" | "messages" | "contact" | "typed_item"
            )
        })
        .collect()
}

fn index_records<'a>(
    records: &[&'a NormalizedRecord],
) -> BTreeMap<String, Vec<&'a NormalizedRecord>> {
    let mut index = BTreeMap::new();
    for record in records {
        index
            .entry(semantic_key(record))
            .or_insert_with(Vec::new)
            .push(*record);
    }
    index
}

fn semantic_key(record: &NormalizedRecord) -> String {
    let kind = logical_kind(&record.kind);
    for field in ["message-id", "subject", "first_line"] {
        if let Some(value) = record.fields.get(field) {
            return format!("{kind}:{}", display_value(value).to_ascii_lowercase());
        }
    }
    format!("{kind}:{}", record.identity)
}

fn logical_kind(kind: &str) -> &str {
    match kind {
        "messages" => "message",
        "folders" => "folder",
        "bodies" => "body",
        "attachments" => "attachment",
        other => other,
    }
}

fn compare_record(
    left: &NormalizedRecord,
    right: &NormalizedRecord,
    findings: &mut Vec<ComparisonFinding>,
) {
    for field in ["subject", "from", "to", "cc", "date", "message-id"] {
        let left_value = left.fields.get(field).map(|value| display_value(value));
        let right_value = right.fields.get(field).map(|value| display_value(value));
        if left_value != right_value && (left_value.is_some() || right_value.is_some()) {
            findings.push(ComparisonFinding {
                scope: format!("record.{field}"),
                class: ComparisonClass::Failure,
                reason_code: "field_mismatch".to_string(),
                detail: format!(
                    "normalized field differs: left={left_value:?} right={right_value:?}"
                ),
                left_identity: Some(left.identity.clone()),
                right_identity: Some(right.identity.clone()),
            });
        }
    }
    if !left.payload_hashes.is_empty()
        && !right.payload_hashes.is_empty()
        && left.payload_hashes != right.payload_hashes
    {
        findings.push(ComparisonFinding {
            scope: "record.payload".to_string(),
            class: ComparisonClass::Failure,
            reason_code: "payload_hash_mismatch".to_string(),
            detail: "normalized payload hashes differ".to_string(),
            left_identity: Some(left.identity.clone()),
            right_identity: Some(right.identity.clone()),
        });
    }
    if left.status != right.status {
        findings.push(ComparisonFinding {
            scope: "record.status".to_string(),
            class: ComparisonClass::Failure,
            reason_code: "status_mismatch".to_string(),
            detail: format!("normalized status differs: left={:?} right={:?}", left.status, right.status),
            left_identity: Some(left.identity.clone()),
            right_identity: Some(right.identity.clone()),
        });
    }
}

fn display_value(value: &str) -> String {
    serde_json::from_str::<String>(value).unwrap_or_else(|_| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(
        kind: &str,
        identity: &str,
        subject: &str,
        status: EvidenceStatus,
    ) -> NormalizedRecord {
        let mut fields = BTreeMap::new();
        fields.insert("subject".to_string(), subject.to_string());
        NormalizedRecord {
            kind: kind.to_string(),
            identity: identity.to_string(),
            status,
            fields,
            payload_hashes: Vec::new(),
            children: Vec::new(),
        }
    }

    fn output(tool: &str, records: Vec<NormalizedRecord>) -> NormalizedOutput {
        NormalizedOutput {
            tool: tool.to_string(),
            status: EvidenceStatus::Present,
            records,
            artifacts: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn matches_semantic_records_without_using_filenames() {
        let left = output("readpst", vec![record("message", "0001.eml", "Hello", EvidenceStatus::Present)]);
        let right = output("pstd", vec![record("messages", "msg_source_1", "\"Hello\"", EvidenceStatus::Present)]);
        let summary = compare_outputs(&left, &right);
        assert_eq!(summary.class, ComparisonClass::Parity);
        assert_eq!(summary.matched_records, 1);
        assert!(summary.findings.is_empty());
    }

    #[test]
    fn records_extensions_and_missing_records_explicitly() {
        let left = output("readpst", vec![record("message", "m1", "Hello", EvidenceStatus::Present)]);
        let right = output(
            "pstd",
            vec![
                record("messages", "m1", "Hello", EvidenceStatus::Present),
                record("messages", "m2", "Extra", EvidenceStatus::Present),
            ],
        );
        let summary = compare_outputs(&left, &right);
        assert_eq!(summary.class, ComparisonClass::PstdExtension);
        assert!(summary
            .findings
            .iter()
            .any(|finding| finding.reason_code == "pstd_record_not_in_readpst"));
    }

    #[test]
    fn mismatches_and_empty_outputs_are_failures() {
        let left = output("readpst", vec![record("message", "m1", "Hello", EvidenceStatus::Present)]);
        let right = output("pstd", vec![record("messages", "m1", "Different", EvidenceStatus::Present)]);
        let summary = compare_outputs(&left, &right);
        assert_eq!(summary.class, ComparisonClass::Failure);
        assert!(summary
            .findings
            .iter()
            .any(|finding| finding.reason_code == "readpst_record_missing_from_pstd"));
        let empty = output("readpst", Vec::new());
        let empty_summary = compare_outputs(&empty, &output("pstd", Vec::new()));
        assert_eq!(empty_summary.class, ComparisonClass::Failure);
    }
}
