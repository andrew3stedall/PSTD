mod readpst_diff;

use readpst_diff::manifest::FixtureManifest;
use readpst_diff::normalize::{
    normalize_pstd_archive, normalize_readpst_directory, NormalizationLimits,
};
use readpst_diff::report::{build_differential_report, reports_are_deterministic, write_report};
use readpst_diff::runner::{run_isolated, CommandSpec, RunLimits};
use std::env;
use tempfile::tempdir;

#[test]
fn approved_unicode_fixture_runs_through_isolated_comparison_contract() {
    let fixture = FixtureManifest::approved_unicode_tika();
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    readpst_diff::runner::validate_fixture_on_disk(&fixture, repository_root)
        .expect("approved fixture must pass provenance and hash admission");

    let sandbox = tempdir().expect("sandbox");
    let readpst_sandbox = sandbox.path().join("readpst-run");
    let pstd_sandbox = sandbox.path().join("pstd-run");
    let readpst_output = readpst_sandbox.join("output");
    let pstd_output = pstd_sandbox.join("output");
    let readpst = run_isolated(
        &CommandSpec::new(
            "readpst-contract-stub",
            "pinned-contract-test",
            vec![
                "sh".to_string(),
                "-c".to_string(),
                "printf 'Subject: Hello\\n\\nBody' > message.eml".to_string(),
            ],
            &readpst_sandbox,
            &readpst_output,
        ),
        &RunLimits::default(),
    )
    .expect("isolated readpst contract process");
    let pstd = run_isolated(
        &CommandSpec::new(
            "pstd-contract-stub",
            "canonical-contract-test",
            vec![
                "sh".to_string(),
                "-c".to_string(),
                r#"mkdir -p data; printf '%s\n' '{"message_key":"m1","subject":"Hello","status":"present"}' > data/messages.jsonl; tar -cf canonical.tar data/messages.jsonl"#.to_string(),
            ],
            &pstd_sandbox,
            &pstd_output,
        ),
        &RunLimits::default(),
    )
    .expect("isolated PSTD contract process");

    let readpst_normalized =
        normalize_readpst_directory(&readpst_output, "readpst", &NormalizationLimits::default())
            .expect("normalize readpst contract output");
    let pstd_normalized =
        normalize_pstd_archive(&pstd_output, "pstd", &NormalizationLimits::default())
            .expect("normalize PSTD contract output");
    let report = build_differential_report(
        &fixture,
        &readpst,
        &pstd,
        readpst_normalized,
        pstd_normalized,
        1,
        "utf-8",
        "eml",
        true,
    )
    .expect("build differential report");
    assert!(report.comparison.validate().is_ok());
    assert!(report.evidence.validate().is_ok());
    assert!(reports_are_deterministic(&report, &report).expect("deterministic report"));
    let report_path =
        write_report(sandbox.path(), "evidence/comparison.json", &report).expect("write report");
    assert!(report_path.is_file());
}

#[test]
fn configured_readpst_approved_fixture_differential() {
    let Some(readpst_bin) = env::var_os("PSTD_READPST_BIN") else {
        eprintln!(
            "PSTD_READPST_BIN is not set; dedicated pinned-oracle workflow provisions this test"
        );
        return;
    };
    let fixture = FixtureManifest::approved_unicode_tika();
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = readpst_diff::runner::validate_fixture_on_disk(&fixture, repository_root)
        .expect("approved fixture must pass provenance and hash admission");
    let sandbox = tempdir().expect("sandbox");
    let readpst_version =
        env::var("PSTD_READPST_VERSION").unwrap_or_else(|_| "pinned-source-build".to_string());
    let pstd_bin = env!("CARGO_BIN_EXE_pstd").to_string();
    let fixture_path = fixture_path.to_string_lossy().into_owned();
    let readpst_bin = readpst_bin.to_string_lossy().into_owned();
    let run_pair = |suffix: &str| {
        let readpst_sandbox = sandbox.path().join(format!("readpst-{suffix}"));
        let pstd_sandbox = sandbox.path().join(format!("pstd-{suffix}"));
        let readpst_output = readpst_sandbox.join("output");
        let pstd_output = pstd_sandbox.join("output");
        let readpst = run_isolated(
            &CommandSpec::new(
                "readpst",
                readpst_version.clone(),
                vec![
                    readpst_bin.clone(),
                    "-e".to_string(),
                    "-8".to_string(),
                    "-o".to_string(),
                    readpst_output.to_string_lossy().into_owned(),
                    fixture_path.clone(),
                ],
                &readpst_sandbox,
                &readpst_output,
            ),
            &RunLimits::default(),
        )
        .expect("run pinned readpst");
        let pstd = run_isolated(
            &CommandSpec::new(
                "pstd",
                env!("CARGO_PKG_VERSION"),
                vec![
                    pstd_bin.clone(),
                    "extract".to_string(),
                    "--input".to_string(),
                    fixture_path.clone(),
                    "--output".to_string(),
                    pstd_output.to_string_lossy().into_owned(),
                    "--overwrite".to_string(),
                ],
                &pstd_sandbox,
                &pstd_output,
            ),
            &RunLimits::default(),
        )
        .expect("run PSTD");
        let readpst_normalized = normalize_readpst_directory(
            &readpst_output,
            "readpst",
            &NormalizationLimits::default(),
        )
        .expect("normalize readpst output");
        let pstd_normalized =
            normalize_pstd_archive(&pstd_output, "pstd", &NormalizationLimits::default())
                .expect("normalize PSTD output");
        (readpst, pstd, readpst_normalized, pstd_normalized)
    };

    let (readpst_a, pstd_a, readpst_normalized_a, pstd_normalized_a) = run_pair("a");
    let (readpst_b, pstd_b, readpst_normalized_b, pstd_normalized_b) = run_pair("b");
    assert!(readpst_a.execution.exit_status.is_some());
    assert!(pstd_a.execution.exit_status.is_some());
    let deterministic_repeat =
        readpst_normalized_a == readpst_normalized_b && pstd_normalized_a == pstd_normalized_b;
    if !deterministic_repeat {
        eprintln!(
            "readpst repeat summary: {:?}",
            (
                &readpst_normalized_a.status,
                readpst_normalized_a
                    .records
                    .iter()
                    .map(|record| (
                        &record.kind,
                        &record.identity,
                        &record.status,
                        &record.payload_hashes
                    ))
                    .collect::<Vec<_>>(),
                &readpst_normalized_a.artifacts,
                &readpst_normalized_b.status,
                readpst_normalized_b
                    .records
                    .iter()
                    .map(|record| (
                        &record.kind,
                        &record.identity,
                        &record.status,
                        &record.payload_hashes
                    ))
                    .collect::<Vec<_>>(),
                &readpst_normalized_b.artifacts
            )
        );
        eprintln!(
            "pstd repeat summary: {:?}",
            (
                &pstd_normalized_a.status,
                pstd_normalized_a
                    .records
                    .iter()
                    .map(|record| (
                        &record.kind,
                        &record.identity,
                        &record.status,
                        &record.payload_hashes
                    ))
                    .collect::<Vec<_>>(),
                &pstd_normalized_a.artifacts,
                &pstd_normalized_b.status,
                pstd_normalized_b
                    .records
                    .iter()
                    .map(|record| (
                        &record.kind,
                        &record.identity,
                        &record.status,
                        &record.payload_hashes
                    ))
                    .collect::<Vec<_>>(),
                &pstd_normalized_b.artifacts
            )
        );
    }
    let report = build_differential_report(
        &fixture,
        &readpst_a,
        &pstd_a,
        readpst_normalized_a,
        pstd_normalized_a,
        1,
        "utf-8",
        "eml",
        deterministic_repeat,
    )
    .expect("build configured differential report");
    let report_b = build_differential_report(
        &fixture,
        &readpst_b,
        &pstd_b,
        readpst_normalized_b,
        pstd_normalized_b,
        1,
        "utf-8",
        "eml",
        deterministic_repeat,
    )
    .expect("build repeated differential report");
    let evidence_root = env::var_os("PSTD_DIFFERENTIAL_EVIDENCE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| sandbox.path().join("evidence"));
    let report_path = write_report(&evidence_root, "comparison.json", &report)
        .expect("write configured differential report");
    assert!(report_path.is_file());
    assert!(
        deterministic_repeat,
        "repeated normalized outputs are not deterministic; evidence was written to {}",
        report_path.display()
    );
    assert!(reports_are_deterministic(&report, &report_b).expect("compare repeated reports"));
}
