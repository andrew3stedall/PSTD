use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::config::ExtractConfig;
use crate::engine::runner::run_extract;
use crate::error::PstdResult;

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
    pub pst_completed: u64,
    pub pst_failed: u64,
    pub pst_skipped: u64,
    pub continue_on_error: bool,
    pub status: String,
    pub items: Vec<BatchItemSummary>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchItemSummary {
    pub pst_path: String,
    pub pst_output: String,
    pub status: String,
    pub run_id: Option<String>,
    pub pst_id: Option<String>,
    pub message: Option<String>,
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
    let checkpoint_path = config.output.join("batch_checkpoint.jsonl");
    let mut items = Vec::new();

    for pst_path in pst_paths {
        let item_output = config.output.join(batch_output_dir_name(&pst_path));
        let checkpoint = run_batch_item(&config, &pst_path, &item_output);
        append_checkpoint(&checkpoint_path, &checkpoint)?;

        let failed = checkpoint.status == "failed";
        items.push(checkpoint);
        if failed && !config.continue_on_error {
            break;
        }
    }

    let finished = chrono::Utc::now();
    let pst_total = items.len() as u64;
    let pst_completed = items
        .iter()
        .filter(|item| item.status == "completed")
        .count() as u64;
    let pst_failed = items.iter().filter(|item| item.status == "failed").count() as u64;
    let pst_skipped = items
        .iter()
        .filter(|item| item.status == "skipped_completed")
        .count() as u64;
    let status = if pst_failed == 0 {
        "completed".to_string()
    } else if config.continue_on_error {
        "completed_with_failures".to_string()
    } else {
        "failed".to_string()
    };

    let summary = BatchSummary {
        batch_id,
        started_at: started.to_rfc3339(),
        finished_at: Some(finished.to_rfc3339()),
        duration_seconds: Some(timer.elapsed().as_secs_f64()),
        input_root: config.input.display().to_string(),
        output_root: config.output.display().to_string(),
        pst_total,
        pst_completed,
        pst_failed,
        pst_skipped,
        continue_on_error: config.continue_on_error,
        status,
        items,
    };

    fs::write(
        config.output.join("batch_summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    Ok(summary)
}

fn run_batch_item(config: &BatchConfig, pst_path: &Path, item_output: &Path) -> BatchItemSummary {
    if is_completed(item_output) && !config.overwrite {
        return BatchItemSummary {
            pst_path: pst_path.display().to_string(),
            pst_output: item_output.display().to_string(),
            status: "skipped_completed".to_string(),
            run_id: None,
            pst_id: None,
            message: Some(
                "existing run_summary.json found; use --overwrite to reprocess".to_string(),
            ),
        };
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
        Ok(summary) => BatchItemSummary {
            pst_path: pst_path.display().to_string(),
            pst_output: item_output.display().to_string(),
            status: "completed".to_string(),
            run_id: Some(summary.run_id),
            pst_id: Some(summary.pst_id),
            message: Some(summary.status),
        },
        Err(error) => BatchItemSummary {
            pst_path: pst_path.display().to_string(),
            pst_output: item_output.display().to_string(),
            status: "failed".to_string(),
            run_id: None,
            pst_id: None,
            message: Some(error.to_string()),
        },
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
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", serde_json::to_string(item)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{batch_output_dir_name, is_pst_file};

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
}
