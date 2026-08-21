mod readpst_diff;

use std::fs;
use std::path::Path;

use readpst_diff::manifest::{
    approved_unicode_baseline_report, sha256_hex, stable_json, ComparisonRun, CryptMethod,
    EvidenceLevel, EvidenceStatus, FixtureManifest, InputFamily, OutcomeRecord, ParityStatus,
    READPST_SOURCE_REVISION, ToolExecution,
};

#[test]
fn approved_unicode_fixture_is_manifested_with_verified_hash() {
    let fixture = FixtureManifest::approved_unicode_tika();
    fixture.validate().unwrap();

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(&fixture.local_path);
    let bytes = fs::read(path).expect("approved Unicode fixture must be present");
    assert_eq!(bytes.len() as u64, fixture.size_bytes);
    assert_eq!(sha256_hex(&bytes), fixture.sha256);
}

#[test]
fn parity_status_and_negative_evidence_are_explicit() {
    let parity_values = [
        ParityStatus::Implemented,
        ParityStatus::Partial,
        ParityStatus::Gap,
        ParityStatus::UnsupportedByReadpst,
        ParityStatus::Filtered,
        ParityStatus::Unavailable,
        ParityStatus::Failed,
    ];
    let encoded = String::from_utf8(stable_json(&parity_values).unwrap()).unwrap();
    for expected in [
        "implemented",
        "partial",
        "gap",
        "unsupported_by_readpst",
        "filtered",
        "unavailable",
        "failed",
    ] {
        assert!(encoded.contains(expected), "missing status {expected}");
    }

    let cases = [
        EvidenceStatus::Skipped,
        EvidenceStatus::Unavailable,
        EvidenceStatus::Malformed,
        EvidenceStatus::Ambiguous,
        EvidenceStatus::Unsupported,
        EvidenceStatus::Failed,
    ];
    let encoded = String::from_utf8(stable_json(&cases).unwrap()).unwrap();
    for expected in [
        "skipped",
        "unavailable",
        "malformed",
        "ambiguous",
        "unsupported",
        "failed",
    ] {
        assert!(encoded.contains(expected), "missing evidence status {expected}");
    }
}

#[test]
fn unicode_baseline_report_is_complete_and_deterministic() {
    let report = approved_unicode_baseline_report();
    report.validate().unwrap();

    let first = stable_json(&report).unwrap();
    let second = stable_json(&report).unwrap();
    assert_eq!(first, second);
    assert_eq!(report.evidence_level, EvidenceLevel::E2);
    assert_eq!(report.parity_status, ParityStatus::Partial);
    assert_eq!(report.inventory.as_ref().unwrap().folders, 8);
    assert!(report.deterministic_repeat);
}

#[test]
fn comparison_contract_requires_pinned_source_and_e4_for_implemented() {
    let run = comparison_run();
    run.validate().unwrap();

    let mut promoted = run.clone();
    promoted.parity_status = ParityStatus::Implemented;
    promoted.evidence_level = EvidenceLevel::E3;
    let error = promoted.validate().unwrap_err();
    assert!(error.contains("E4"));

    let mut drifted = run;
    drifted.readpst_source_revision = "upstream-drift".to_string();
    let error = drifted.validate().unwrap_err();
    assert!(error.contains(READPST_SOURCE_REVISION));
}

#[test]
fn malformed_and_ambiguous_records_cannot_be_serialized_as_success() {
    let malformed = OutcomeRecord {
        case_id: "malformed_derivative".to_string(),
        scope: "pst".to_string(),
        expected: EvidenceStatus::Malformed,
        observed: EvidenceStatus::Malformed,
        reason_code: "invalid_header".to_string(),
        detail: Some("Controlled short-header derivative.".to_string()),
    };
    let ambiguous = OutcomeRecord {
        case_id: "ambiguous_reference".to_string(),
        scope: "attachment".to_string(),
        expected: EvidenceStatus::Ambiguous,
        observed: EvidenceStatus::Ambiguous,
        reason_code: "multiple_id2_targets".to_string(),
        detail: None,
    };

    malformed.validate().unwrap();
    ambiguous.validate().unwrap();
    let encoded = String::from_utf8(stable_json(&[malformed, ambiguous]).unwrap()).unwrap();
    assert!(encoded.contains("\"malformed\""));
    assert!(encoded.contains("\"ambiguous\""));
    assert!(!encoded.contains("\"present\""));
}

fn comparison_run() -> ComparisonRun {
    let fixture = FixtureManifest::approved_unicode_tika();
    let stdout_sha256 = sha256_hex(b"stdout");
    let stderr_sha256 = sha256_hex(b"stderr");
    let readpst = ToolExecution {
        tool: "readpst".to_string(),
        version: READPST_SOURCE_REVISION.to_string(),
        command: vec!["readpst".to_string(), "-e".to_string()],
        exit_status: Some(0),
        stdout_sha256: Some(stdout_sha256.clone()),
        stderr_sha256: Some(stderr_sha256.clone()),
        output_root: "readpst-output".to_string(),
        status: EvidenceStatus::Present,
    };
    let pstd = ToolExecution {
        tool: "pstd".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        command: vec![
            "pstd".to_string(),
            "extract".to_string(),
            "--profile".to_string(),
            "canonical".to_string(),
        ],
        exit_status: Some(0),
        stdout_sha256: Some(stdout_sha256),
        stderr_sha256: Some(stderr_sha256),
        output_root: "pstd-output".to_string(),
        status: EvidenceStatus::Present,
    };

    ComparisonRun {
        comparison_id: "comparison_unicode_tika_e2".to_string(),
        fixture_id: fixture.fixture_id,
        fixture_sha256: fixture.sha256,
        readpst_source_revision: READPST_SOURCE_REVISION.to_string(),
        input_family: InputFamily::Unicode64,
        crypt_method: CryptMethod::None,
        charset_policy: "readpst-default".to_string(),
        output_profile: "canonical".to_string(),
        worker_count: 1,
        evidence_level: EvidenceLevel::E2,
        parity_status: ParityStatus::Partial,
        readpst,
        pstd,
        outcomes: vec![OutcomeRecord {
            case_id: "unicode_tika_positive".to_string(),
            scope: "fixture".to_string(),
            expected: EvidenceStatus::Present,
            observed: EvidenceStatus::Present,
            reason_code: "canonical_baseline".to_string(),
            detail: None,
        }],
    }
}
