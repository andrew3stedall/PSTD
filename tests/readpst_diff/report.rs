use super::compare::{findings_to_outcomes, ComparisonClass, ComparisonSummary};
use super::manifest::{
    stable_json, ArtifactDigest, ComparisonRun, EvidenceLevel, EvidenceReport, FixtureManifest,
    OutcomeRecord, ParityStatus, READPST_SOURCE_REVISION,
};
use super::normalize::NormalizedOutput;
use super::runner::RunResult;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct DifferentialReport {
    pub comparison: ComparisonRun,
    pub summary: ComparisonSummary,
    pub evidence: EvidenceReport,
    pub deterministic_repeat: bool,
    pub notes: Vec<String>,
}

pub fn build_differential_report(
    fixture: &FixtureManifest,
    readpst: &RunResult,
    pstd: &RunResult,
    readpst_output: NormalizedOutput,
    pstd_output: NormalizedOutput,
    worker_count: u32,
    charset_policy: impl Into<String>,
    output_profile: impl Into<String>,
    deterministic_repeat: bool,
) -> Result<DifferentialReport, String> {
    let charset_policy = charset_policy.into();
    let output_profile = output_profile.into();
    fixture.validate()?;
    readpst.execution.validate()?;
    pstd.execution.validate()?;
    readpst_output.validate()?;
    pstd_output.validate()?;
    let summary = super::compare::compare_outputs(&readpst_output, &pstd_output);
    let outcomes = findings_to_outcomes(&summary)
        .into_iter()
        .map(
            |(case_id, expected, observed, reason_code, detail)| OutcomeRecord {
                case_id,
                scope: "differential".to_string(),
                expected,
                observed,
                reason_code,
                detail: Some(detail),
            },
        )
        .collect::<Vec<_>>();
    let parity_status = parity_status(summary.class);
    let comparison = ComparisonRun {
        comparison_id: format!("{}-{}", fixture.fixture_id, output_profile.into()),
        fixture_id: fixture.fixture_id.clone(),
        fixture_sha256: fixture.sha256.clone(),
        readpst_source_revision: READPST_SOURCE_REVISION.to_string(),
        input_family: fixture.input_family,
        crypt_method: fixture.crypt_method,
        charset_policy: charset_policy.into(),
        output_profile: output_profile.into(),
        worker_count,
        evidence_level: EvidenceLevel::E2,
        parity_status,
        readpst: readpst.execution.clone(),
        pstd: pstd.execution.clone(),
        outcomes: outcomes.clone(),
    };
    comparison.validate()?;

    let mut artifacts = prefixed_artifacts("readpst", &readpst.artifacts)?;
    artifacts.extend(prefixed_artifacts("pstd", &pstd.artifacts)?);
    let evidence = EvidenceReport {
        report_id: format!("{}-evidence", comparison.comparison_id),
        fixture_id: fixture.fixture_id.clone(),
        fixture_sha256: fixture.sha256.clone(),
        evidence_level: EvidenceLevel::E2,
        parity_status,
        inventory: None,
        outcomes,
        artifacts,
        deterministic_repeat,
        notes: vec![
            "One approved fixture differential is E2 evidence, not a final parity claim.".to_string(),
            format!("comparison_class={:?}", summary.class),
        ],
    };
    evidence.validate()?;

    let mut notes = vec![
        format!("readpst_stdout_sha256={}", readpst.execution.stdout_sha256.clone().unwrap_or_default()),
        format!("readpst_stderr_sha256={}", readpst.execution.stderr_sha256.clone().unwrap_or_default()),
        format!("pstd_stdout_sha256={}", pstd.execution.stdout_sha256.clone().unwrap_or_default()),
        format!("pstd_stderr_sha256={}", pstd.execution.stderr_sha256.clone().unwrap_or_default()),
    ];
    if !readpst.escaped_paths.is_empty() || !pstd.escaped_paths.is_empty() {
        notes.push("path_escape_detected".to_string());
    }
    if readpst.timed_out || pstd.timed_out {
        notes.push("timeout_detected".to_string());
    }
    if readpst.output_limited || pstd.output_limited {
        notes.push("output_limit_detected".to_string());
    }
    notes.sort();

    Ok(DifferentialReport {
        comparison,
        summary,
        evidence,
        deterministic_repeat,
        notes,
    })
}

pub fn write_report(
    report_root: &Path,
    relative_path: &str,
    report: &DifferentialReport,
) -> Result<PathBuf, String> {
    validate_relative_path(relative_path)?;
    fs::create_dir_all(report_root)
        .map_err(|error| format!("report_root_create_failed:{error}"))?;
    let path = report_root.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("report_parent_create_failed:{error}"))?;
    }
    let bytes = stable_json(report)?;
    let temporary = path.with_extension("tmp");
    let mut file = fs::File::create(&temporary)
        .map_err(|error| format!("report_temp_create_failed:{error}"))?;
    file.write_all(&bytes)
        .map_err(|error| format!("report_temp_write_failed:{error}"))?;
    file.sync_all()
        .map_err(|error| format!("report_temp_sync_failed:{error}"))?;
    fs::rename(&temporary, &path)
        .map_err(|error| format!("report_publish_failed:{error}"))?;
    Ok(path)
}

pub fn reports_are_deterministic(
    first: &DifferentialReport,
    second: &DifferentialReport,
) -> Result<bool, String> {
    Ok(stable_json(first)? == stable_json(second)?)
}

fn parity_status(class: ComparisonClass) -> ParityStatus {
    match class {
        ComparisonClass::Parity | ComparisonClass::PstdExtension => ParityStatus::Partial,
        ComparisonClass::Unsupported => ParityStatus::UnsupportedByReadpst,
        ComparisonClass::Failure => ParityStatus::Failed,
    }
}

fn prefixed_artifacts(
    prefix: &str,
    artifacts: &[ArtifactDigest],
) -> Result<Vec<ArtifactDigest>, String> {
    artifacts
        .iter()
        .map(|artifact| {
            let mut prefixed = artifact.clone();
            prefixed.path = format!("{prefix}/{}", artifact.path);
            prefixed.validate()?;
            Ok(prefixed)
        })
        .collect()
}

fn validate_relative_path(value: &str) -> Result<(), String> {
    let path = Path::new(value);
    if value.trim().is_empty() || path.is_absolute() {
        return Err("report path must be relative and non-empty".to_string());
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err("report path must not contain traversal components".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::readpst_diff::manifest::{EvidenceStatus, ToolExecution};
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    fn fixture() -> FixtureManifest {
        FixtureManifest::approved_unicode_tika()
    }

    fn execution(tool: &str, root: &str) -> ToolExecution {
        ToolExecution {
            tool: tool.to_string(),
            version: "test-1".to_string(),
            command: vec![tool.to_string(), "fixture".to_string()],
            exit_status: Some(0),
            stdout_sha256: Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string()),
            stderr_sha256: Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string()),
            output_root: root.to_string(),
            status: EvidenceStatus::Present,
        }
    }

    fn run(tool: &str, root: &str) -> RunResult {
        RunResult {
            execution: execution(tool, root),
            stdout: Vec::new(),
            stderr: Vec::new(),
            artifacts: Vec::new(),
            escaped_paths: Vec::new(),
            timed_out: false,
            output_limited: false,
        }
    }

    fn output(tool: &str) -> NormalizedOutput {
        let mut fields = BTreeMap::new();
        fields.insert("subject".to_string(), "Hello".to_string());
        NormalizedOutput {
            tool: tool.to_string(),
            status: EvidenceStatus::Present,
            records: vec![crate::readpst_diff::normalize::NormalizedRecord {
                kind: "message".to_string(),
                identity: "m1".to_string(),
                status: EvidenceStatus::Present,
                fields,
                payload_hashes: Vec::new(),
                children: Vec::new(),
            }],
            artifacts: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn builds_valid_compact_e2_report_and_keeps_partial_status() {
        let report = build_differential_report(
            &fixture(),
            &run("readpst", "readpst"),
            &run("pstd", "pstd"),
            output("readpst"),
            output("pstd"),
            1,
            "utf-8",
            "eml",
            true,
        )
        .expect("report");
        assert_eq!(report.comparison.evidence_level, EvidenceLevel::E2);
        assert_eq!(report.comparison.parity_status, ParityStatus::Partial);
        assert!(report.evidence.validate().is_ok());
    }

    #[test]
    fn report_write_is_confined_and_repeatable() {
        let report = build_differential_report(
            &fixture(),
            &run("readpst", "readpst"),
            &run("pstd", "pstd"),
            output("readpst"),
            output("pstd"),
            1,
            "utf-8",
            "eml",
            true,
        )
        .expect("report");
        let root = tempdir().expect("tempdir");
        let path = write_report(root.path(), "reports/differential.json", &report).expect("write");
        assert!(path.is_file());
        assert!(reports_are_deterministic(&report, &report).expect("determinism"));
        assert!(write_report(root.path(), "../escape.json", &report).is_err());
    }
}
