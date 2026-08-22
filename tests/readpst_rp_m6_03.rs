use std::fs;
use std::path::{Path, PathBuf};

use pstd::config::ReadpstPolicy;
use pstd::engine::batch::{discover_pst_files, run_batch, BatchConfig, BatchProgressEvent};

fn batch_config(input: &Path, output: &Path, jobs: u16) -> BatchConfig {
    let readpst = ReadpstPolicy {
        jobs,
        ..ReadpstPolicy::default()
    };
    BatchConfig {
        input: input.to_path_buf(),
        output: output.to_path_buf(),
        recursive: true,
        continue_on_error: true,
        overwrite: true,
        manifest_only: false,
        archive_format: "tar".to_string(),
        data_format: "jsonl".to_string(),
        tar_shard_size_mb: 1,
        progress: "off".to_string(),
        log_level: "error".to_string(),
        profile: "balanced".to_string(),
        readpst,
    }
}

fn stable_item_view(summary: &pstd::engine::batch::BatchSummary) -> Vec<(String, String, bool)> {
    summary
        .items
        .iter()
        .map(|item| {
            let output_name = PathBuf::from(&item.pst_output)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string();
            (output_name, item.status.clone(), item.output_exists)
        })
        .collect()
}

fn stable_progress_view(path: &Path) -> Vec<(String, String, Option<String>, Option<String>, u64, u64, u64, u64, u64, u64, u64)> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<BatchProgressEvent>(line).unwrap())
        .map(|event| {
            (
                serde_json::to_string(&event.event_type).unwrap(),
                event
                    .pst_path
                    .as_deref()
                    .and_then(|path| Path::new(path).file_name())
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                event.item_status,
                Some(event.message),
                event.pst_discovered,
                event.pst_attempted,
                event.pst_completed,
                event.pst_partial,
                event.pst_failed,
                event.pst_skipped,
                event.pst_not_run,
            )
        })
        .collect()
}

#[test]
fn bounded_worker_runs_preserve_sorted_batch_results() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("inputs");
    fs::create_dir_all(input.join("nested")).unwrap();
    for name in ["zeta.pst", "alpha.pst", "nested/middle.pst"] {
        fs::write(input.join(name), b"!BDN\0\0\0\0").unwrap();
    }

    let one = run_batch(batch_config(&input, &temp.path().join("one"), 1)).unwrap();
    let many = run_batch(batch_config(&input, &temp.path().join("many"), 4)).unwrap();

    assert_eq!(one.pst_discovered, 3);
    assert_eq!(many.pst_discovered, 3);
    assert_eq!(stable_item_view(&one), stable_item_view(&many));
    assert_eq!(
        one.items
            .iter()
            .map(|item| {
                PathBuf::from(&item.pst_path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<Vec<_>>(),
        vec![
            "alpha.pst".to_string(),
            "middle.pst".to_string(),
            "zeta.pst".to_string(),
        ]
    );
    assert_eq!(
        stable_progress_view(&temp.path().join("one/batch_progress.jsonl")),
        stable_progress_view(&temp.path().join("many/batch_progress.jsonl"))
    );
}

#[test]
fn recursive_discovery_skips_directory_symlinks_and_rejects_symlink_input() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("inputs");
    fs::create_dir_all(input.join("real")).unwrap();
    fs::write(input.join("real/valid.pst"), b"!BDN").unwrap();
    std::os::unix::fs::symlink(&input, input.join("loop")).unwrap();

    let discovered = discover_pst_files(&input, true).unwrap();
    assert_eq!(discovered.len(), 1);
    assert!(discovered[0].ends_with("real/valid.pst"));

    let link = temp.path().join("input-link.pst");
    std::os::unix::fs::symlink(input.join("real/valid.pst"), &link).unwrap();
    let error = discover_pst_files(&link, false).unwrap_err();
    assert!(error.to_string().contains("symlink input is not admitted"));
}
