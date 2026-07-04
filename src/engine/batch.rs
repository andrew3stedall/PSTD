use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::config::ExtractConfig;
use crate::engine::runner::run_extract;
use crate::error::PstdResult;
use crate::output::summary::ExtractionSummary;

#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub input: PathBuf,
    pub output: PathBuf,
    pub recursive: bool,
    pub continue_on_error: bool,
    pub overwrite: bool,
    pub manifest_only: bool,
    pub archive_format: String,
    pub data_format: String,
    pub tar_shard_size_mb: u64,
    pub progress: String,
    pub log_level: String,
    pub profile: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchSummary {
    pub batch_id: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub duration_seconds: Option<f64>,
    pub input_root: String,
    pub output_root: String,
    pub pst_total: u64,
    pub pst_discovered: u64,
    pub pst_attempted: u64,
    pub pst_completed: u64,
    pub pst_partial: u64,
    pub pst_failed: u64,
    pub pst_skipped: u64,
    pub pst_not_run: u64,
    pub continue_on_error: bool,
    pub status: String,
    pub operator_message: String,
    pub checkpoint_path: String,
    pub progress_path: String,
    pub items: Vec<BatchItemSummary>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchItemSummary {
    pub pst_path: String,
    pub pst_output: String,
    pub status: String,
    pub run_id: Option<String>,
    pub pst_id: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub duration_seconds: Option<f64>,
    pub messages_discovered: u64,
    pub messages_extracted: u64,
    pub messages_not_extracted: u64,
    pub attachments_extracted: u64,
    pub attachments_not_extracted: u64,
    pub tar_shards_written: u64,
    pub bytes_written: u64,
    pub output_exists: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct BatchCounters {
    pst_discovered: u64,
    pst_attempted: u64,
    pst_completed: u64,
    pst_partial: u64,
    pst_failed: u64,
    pst_skipped: u64,
    pst_not_run: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchProgressEvent {
    pub batch_id: String,
    pub event_type: String,
    pub timestamp_utc: String,
    pub pst_path: Option<String>,
    pub pst_output: Option<String>,
    pub item_status: Option<String>,
    pub pst_discovered: u64,
    pub pst_attempted: u64,
    pub pst_completed: u64,
    pub pst_partial: u64,
    pub pst_failed: u64,
    pub pst_skipped: u64,
    pub pst_not_run: u64,
    pub message: String,
}

pub fn run_batch(config: BatchConfig) -> PstdResult<BatchSummary> {
    fs::create_dir_all(&config.output)?;
    let started = chrono::Utc::now();
    let timer = Instant::now();
    let batch_id = crate::output::ids::stable_id(
        "batch",
        &[
            &config.input.display().to_string(),
            &started.timestamp_millis().to_string(),
        ],
    );

    let pst_paths = discover_pst_files(&config.input, config.recursive)?;
    let pst_discovered = pst_paths.len() as u64;
    let checkpoint_path = config.output.join("batch_checkpoint.jsonl");
    let progress_path = config.output.join("batch_progress.jsonl");
    let mut items = Vec::new();

    write_batch_progress(
        &progress_path,
        &BatchProgressEvent::new(
            &batch_id,
            "batch_started",
            None,
            None,
            None,
            BatchCounters {
                pst_discovered,
                pst_not_run: pst_discovered,
                ..BatchCounters::default()
            },
            format!("discovered {pst_discovered} PST file(s)"),
        ),
    )?;

    for pst_path in pst_paths {
        let item_output = config.output.join(batch_output_dir_name(&pst_path));
        write_batch_progress(
            &progress_path,
            &BatchProgressEvent::new(
                &batch_id,
                "pst_started",
                Some(&pst_path),
                Some(&item_output),
                None,
                counters_from_items(&items, pst_discovered),
                "PST batch item started",
            ),
        )?;

        let checkpoint = run_batch_item(&config, &pst_path, &item_output);
        append_checkpoint(&checkpoint_path, &checkpoint)?;

        let failed = checkpoint.status == "failed";
        items.push(checkpoint.clone());
        write_batch_progress(
            &progress_path,
            &BatchProgressEvent::new(
                &batch_id,
                "pst_finished",
                Some(&pst_path),
                Some(&item_output),
                Some(&checkpoint.status),
                counters_from_items(&items, pst_discovered),
                checkpoint
                    .message
                    .clone()
                    .unwrap_or_else(|| "PST batch item finished".to_string()),
            ),
        )?;

        if failed && !config.continue_on_error {
            break;
        }
    }

    let finished = chrono::Utc::now();
    let counters = counters_from_items(&items, pst_discovered);
    let status = batch_status(&counters, config.continue_on_error);
    let operator_message = operator_message(&counters, &status);

    let summary = BatchSummary {
        batch_id: batch_id.clone(),
        started_at: started.to_rfc3339(),
        finished_at: Some(finished.to_rfc3339()),
        duration_seconds: Some(timer.elapsed().as_secs_f64()),
        input_root: config.input.display().to_string(),
        output_root: config.output.display().to_string(),
        pst_total: counters.pst_discovered,
        pst_discovered: counters.pst_discovered,
        pst_attempted: counters.pst_attempted,
        pst_completed: counters.pst_completed,
        pst_partial: counters.pst_partial,
        pst_failed: counters.pst_failed,
        pst_skipped: counters.pst_skipped,
        pst_not_run: counters.pst_not_run,
        continue_on_error: config.continue_on_error,
        status,
        operator_message,
        checkpoint_path: checkpoint_path.display().to_string(),
        progress_path: progress_path.display().to_string(),
        items,
    };

    fs::write(
        config.output.join("batch_summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;

    write_batch_progress(
        &progress_path,
        &BatchProgressEvent::new(
            &batch_id,
            "batch_finished",
            None,
            None,
            Some(&summary.status),
            counters,
            summary.operator_message.clone(),
        ),
    )?;

    Ok(summary)
}

fn run_batch_item(config: &BatchConfig, pst_path: &Path, item_output: &Path) -> BatchItemSummary {
    let item_started = chrono::Utc::now();
    let timer = Instant::now();

    if is_completed(item_output) && !config.overwrite {
        return skipped_item_summary(pst_path, item_output, item_started, timer.elapsed().as_secs_f64());
    }

    let extract_config = ExtractConfig {
        input: pst_path.to_path_buf(),
        output: item_output.to_path_buf(),
        continue_on_error: config.continue_on_error,
        overwrite: config.overwrite,
        manifest_only: config.manifest_only,
        archive_format: config.archive_format.clone(),
        data_format: config.data_format.clone(),
        tar_shard_size_mb: config.tar_shard_size_mb,
        progress: config.progress.clone(),
        log_level: config.log_level.clone(),
        profile: config.profile.clone(),
    };

    match run_extract(extract_config) {
        Ok(summary) => batch_item_from_extraction_summary(
            pst_path,
            item_output,
            summary,
            item_started,
            timer.elapsed().as_secs_f64(),
        ),
        Err(error) => BatchItemSummary {
            pst_path: pst_path.display().to_string(),
            pst_output: item_output.display().to_string(),
            status: "failed".to_string(),
            run_id: None,
            pst_id: None,
            started_at: Some(item_started.to_rfc3339()),
            finished_at: Some(chrono::Utc::now().to_rfc3339()),
            duration_seconds: Some(timer.elapsed().as_secs_f64()),
            messages_discovered: 0,
            messages_extracted: 0,
            messages_not_extracted: 0,
            attachments_extracted: 0,
            attachments_not_extracted: 0,
            tar_shards_written: 0,
            bytes_written: 0,
            output_exists: item_output.exists(),
            message: Some(error.to_string()),
        },
    }
}

fn batch_item_from_extraction_summary(
    pst_path: &Path,
    item_output: &Path,
    summary: ExtractionSummary,
    item_started: chrono::DateTime<chrono::Utc>,
    duration_seconds: f64,
) -> BatchItemSummary {
    let status = classify_extraction_summary(&summary).to_string();
    BatchItemSummary {
        pst_path: pst_path.display().to_string(),
        pst_output: item_output.display().to_string(),
        status,
        run_id: Some(summary.run_id),
        pst_id: Some(summary.pst_id),
        started_at: Some(item_started.to_rfc3339()),
        finished_at: Some(chrono::Utc::now().to_rfc3339()),
        duration_seconds: Some(duration_seconds),
        messages_discovered: summary.messages_discovered,
        messages_extracted: summary.messages_extracted,
        messages_not_extracted: summary.messages_not_extracted,
        attachments_extracted: summary.attachments_extracted,
        attachments_not_extracted: summary.attachments_not_extracted,
        tar_shards_written: summary.tar_shards_written,
        bytes_written: summary.bytes_written,
        output_exists: item_output.exists(),
        message: Some(summary.status),
    }
}

fn skipped_item_summary(
    pst_path: &Path,
    item_output: &Path,
    item_started: chrono::DateTime<chrono::Utc>,
    duration_seconds: f64,
) -> BatchItemSummary {
    let existing = read_existing_summary(item_output);
    let message = existing.as_ref().map_or_else(
        || "existing run_summary.json found; use --overwrite to reprocess".to_string(),
        |summary| {
            format!(
                "existing run_summary.json found; previous_status={}; use --overwrite to reprocess",
                summary.status
            )
        },
    );
    BatchItemSummary {
        pst_path: pst_path.display().to_string(),
        pst_output: item_output.display().to_string(),
        status: "skipped_completed".to_string(),
        run_id: existing.as_ref().map(|summary| summary.run_id.clone()),
        pst_id: existing.as_ref().map(|summary| summary.pst_id.clone()),
        started_at: Some(item_started.to_rfc3339()),
        finished_at: Some(chrono::Utc::now().to_rfc3339()),
        duration_seconds: Some(duration_seconds),
        messages_discovered: existing
            .as_ref()
            .map(|summary| summary.messages_discovered)
            .unwrap_or(0),
        messages_extracted: existing
            .as_ref()
            .map(|summary| summary.messages_extracted)
            .unwrap_or(0),
        messages_not_extracted: existing
            .as_ref()
            .map(|summary| summary.messages_not_extracted)
            .unwrap_or(0),
        attachments_extracted: existing
            .as_ref()
            .map(|summary| summary.attachments_extracted)
            .unwrap_or(0),
        attachments_not_extracted: existing
            .as_ref()
            .map(|summary| summary.attachments_not_extracted)
            .unwrap_or(0),
        tar_shards_written: existing
            .as_ref()
            .map(|summary| summary.tar_shards_written)
            .unwrap_or(0),
        bytes_written: existing
            .as_ref()
            .map(|summary| summary.bytes_written)
            .unwrap_or(0),
        output_exists: item_output.exists(),
        message: Some(message),
    }
}

fn read_existing_summary(item_output: &Path) -> Option<ExtractionSummary> {
    let path = item_output.join("run_summary.json");
    let bytes = fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn classify_extraction_summary(summary: &ExtractionSummary) -> &'static str {
    if summary.messages_not_extracted > 0
        || summary.attachments_not_extracted > 0
        || summary.status.contains("metadata_unavailable")
        || summary.status.contains("partial")
    {
        "partial_success"
    } else {
        "completed"
    }
}

fn counters_from_items(items: &[BatchItemSummary], pst_discovered: u64) -> BatchCounters {
    let pst_completed = items
        .iter()
        .filter(|item| item.status == "completed")
        .count() as u64;
    let pst_partial = items
        .iter()
        .filter(|item| item.status == "partial_success")
        .count() as u64;
    let pst_failed = items.iter().filter(|item| item.status == "failed").count() as u64;
    let pst_skipped = items
        .iter()
        .filter(|item| item.status.starts_with("skipped"))
        .count() as u64;
    let pst_attempted = items.len() as u64 - pst_skipped;
    let pst_not_run = pst_discovered.saturating_sub(items.len() as u64);

    BatchCounters {
        pst_discovered,
        pst_attempted,
        pst_completed,
        pst_partial,
        pst_failed,
        pst_skipped,
        pst_not_run,
    }
}

fn batch_status(counters: &BatchCounters, continue_on_error: bool) -> String {
    if counters.pst_discovered == 0 {
        "no_pst_files_found".to_string()
    } else if counters.pst_failed == 0 && counters.pst_partial == 0 && counters.pst_not_run == 0 {
        "completed".to_string()
    } else if counters.pst_failed == 0 && counters.pst_not_run == 0 {
        "completed_with_partial_success".to_string()
    } else if counters.pst_failed > 0 && continue_on_error {
        "completed_with_failures".to_string()
    } else {
        "failed_stopped_early".to_string()
    }
}

fn operator_message(counters: &BatchCounters, status: &str) -> String {
    format!(
        "status={status}; discovered={}; attempted={}; completed={}; partial={}; failed={}; skipped={}; not_run={}",
        counters.pst_discovered,
        counters.pst_attempted,
        counters.pst_completed,
        counters.pst_partial,
        counters.pst_failed,
        counters.pst_skipped,
        counters.pst_not_run
    )
}

impl BatchProgressEvent {
    fn new(
        batch_id: &str,
        event_type: &str,
        pst_path: Option<&Path>,
        pst_output: Option<&Path>,
        item_status: Option<&str>,
        counters: BatchCounters,
        message: impl Into<String>,
    ) -> Self {
        Self {
            batch_id: batch_id.to_string(),
            event_type: event_type.to_string(),
            timestamp_utc: chrono::Utc::now().to_rfc3339(),
            pst_path: pst_path.map(|path| path.display().to_string()),
            pst_output: pst_output.map(|path| path.display().to_string()),
            item_status: item_status.map(|status| status.to_string()),
            pst_discovered: counters.pst_discovered,
            pst_attempted: counters.pst_attempted,
            pst_completed: counters.pst_completed,
            pst_partial: counters.pst_partial,
            pst_failed: counters.pst_failed,
            pst_skipped: counters.pst_skipped,
            pst_not_run: counters.pst_not_run,
            message: message.into(),
        }
    }
}

pub fn discover_pst_files(input: &Path, recursive: bool) -> PstdResult<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if input.is_file() {
        if is_pst_file(input) {
            paths.push(input.to_path_buf());
        }
    } else if input.is_dir() {
        collect_pst_files(input, recursive, &mut paths)?;
    }
    paths.sort();
    Ok(paths)
}

fn collect_pst_files(dir: &Path, recursive: bool, paths: &mut Vec<PathBuf>) -> PstdResult<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && recursive {
            collect_pst_files(&path, recursive, paths)?;
        } else if path.is_file() && is_pst_file(&path) {
            paths.push(path);
        }
    }
    Ok(())
}

pub fn is_pst_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("pst"))
        .unwrap_or(false)
}

pub fn batch_output_dir_name(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("pst");
    let safe = safe_dir_name(stem);
    let path_display = path.display().to_string();
    let suffix = &crate::output::ids::stable_id("pstout", &[&path_display])[7..];
    format!("{safe}_{suffix}")
}

fn safe_dir_name(value: &str) -> String {
    let safe = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();

    if safe.is_empty() {
        "pst".to_string()
    } else {
        safe
    }
}

fn is_completed(output: &Path) -> bool {
    output.join("run_summary.json").is_file()
}

fn append_checkpoint(path: &Path, item: &BatchItemSummary) -> PstdResult<()> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", serde_json::to_string(item)?)?;
    Ok(())
}

fn write_batch_progress(path: &Path, event: &BatchProgressEvent) -> PstdResult<()> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", serde_json::to_string(event)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        batch_output_dir_name, batch_status, classify_extraction_summary, counters_from_items,
        is_pst_file, operator_message, BatchItemSummary,
    };
    use crate::output::summary::ExtractionSummary;

    #[test]
    fn detects_pst_extensions_case_insensitively() {
        assert!(is_pst_file(Path::new("archive.pst")));
        assert!(is_pst_file(Path::new("archive.PST")));
        assert!(!is_pst_file(Path::new("archive.txt")));
    }

    #[test]
    fn creates_safe_unique_batch_output_names() {
        let output = batch_output_dir_name(Path::new("/tmp/My Archive!.pst"));
        assert!(output.starts_with("My_Archive_"));
        assert!(output.len() > "My_Archive_".len());
    }

    #[test]
    fn classifies_partial_success_from_extraction_gaps() {
        let summary = extraction_summary(10, 9, 1, 3, 2, "metadata_and_payload");
        assert_eq!(classify_extraction_summary(&summary), "partial_success");

        let summary = extraction_summary(10, 10, 0, 3, 0, "metadata_and_payload");
        assert_eq!(classify_extraction_summary(&summary), "completed");
    }

    #[test]
    fn aggregates_batch_counters_with_not_run_items() {
        let items = vec![
            item("completed"),
            item("partial_success"),
            item("failed"),
            item("skipped_completed"),
        ];
        let counters = counters_from_items(&items, 6);

        assert_eq!(counters.pst_discovered, 6);
        assert_eq!(counters.pst_attempted, 3);
        assert_eq!(counters.pst_completed, 1);
        assert_eq!(counters.pst_partial, 1);
        assert_eq!(counters.pst_failed, 1);
        assert_eq!(counters.pst_skipped, 1);
        assert_eq!(counters.pst_not_run, 2);
        assert_eq!(batch_status(&counters, false), "failed_stopped_early");
        assert!(operator_message(&counters, "failed_stopped_early").contains("not_run=2"));
    }

    #[test]
    fn reports_completed_with_partial_success_when_no_failures() {
        let items = vec![item("completed"), item("partial_success")];
        let counters = counters_from_items(&items, 2);

        assert_eq!(batch_status(&counters, true), "completed_with_partial_success");
    }

    fn item(status: &str) -> BatchItemSummary {
        BatchItemSummary {
            pst_path: format!("/tmp/{status}.pst"),
            pst_output: format!("/tmp/out/{status}"),
            status: status.to_string(),
            run_id: None,
            pst_id: None,
            started_at: None,
            finished_at: None,
            duration_seconds: None,
            messages_discovered: 0,
            messages_extracted: 0,
            messages_not_extracted: 0,
            attachments_extracted: 0,
            attachments_not_extracted: 0,
            tar_shards_written: 0,
            bytes_written: 0,
            output_exists: false,
            message: None,
        }
    }

    fn extraction_summary(
        messages_discovered: u64,
        messages_extracted: u64,
        messages_not_extracted: u64,
        attachments_extracted: u64,
        attachments_not_extracted: u64,
        status: &str,
    ) -> ExtractionSummary {
        ExtractionSummary {
            run_id: "run_123".to_string(),
            pst_id: "pst_123".to_string(),
            source_pst_path: "/tmp/archive.pst".to_string(),
            started_at: "2026-01-01T00:00:00Z".to_string(),
            finished_at: Some("2026-01-01T00:00:01Z".to_string()),
            duration_seconds: Some(1.0),
            folders_discovered: 1,
            messages_discovered,
            messages_extracted,
            messages_not_extracted,
            attachments_extracted,
            attachments_not_extracted,
            bytes_read: 0,
            bytes_written: 0,
            tar_shards_written: 1,
            status: status.to_string(),
        }
    }
}
