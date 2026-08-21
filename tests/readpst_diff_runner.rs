mod readpst_diff;

use readpst_diff::manifest::FixtureManifest;
use readpst_diff::normalize::{
    normalize_pstd_archive, normalize_readpst_directory, NormalizationLimits,
};
use readpst_diff::report::{
    build_differential_report, reports_are_deterministic, write_report,
};
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
                "mkdir -p data; printf '%s\\n' '{"message_key":"m1","subject":"Hello","status":"present"}' > data/messages.jsonl; tar -cf canonical.tar data/messages.jsonl".to_string(),
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
fn configured_readpst_is_explicitly_opt_in_until_oracle_binary_is_provisioned() {
    if env::var_os("PSTD_READPST_BIN").is_none() {
        eprintln!("PSTD_READPST_BIN is not set; external pinned readpst execution is not claimed by the contract test");
    }
}
