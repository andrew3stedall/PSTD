use std::fs;
use std::time::Instant;

use sha2::{Digest, Sha256};

use crate::config::ExtractConfig;
use crate::engine::metadata::{extract_metadata, fallback_metadata};
use crate::error::{PstdError, PstdResult};
use crate::output::ids;
use crate::output::jsonl_writer::JsonlBuffer;
use crate::output::summary::ExtractionSummary;
use crate::output::tar_writer::TarShardWriter;
use crate::progress::{ProgressEvent, ProgressEventType};

pub fn run_extract(config: ExtractConfig) -> PstdResult<ExtractionSummary> {
    validate_config(&config)?;
    fs::create_dir_all(&config.output)?;
    fs::create_dir_all(config.output.join("archives"))?;
    fs::create_dir_all(config.output.join("logs"))?;

    let started = chrono::Utc::now();
    let timer = Instant::now();
    let input_display = config.input.display().to_string();
    let run_id = ids::run_id(&input_display);
    let pst_id = ids::pst_id(&input_display);

    write_root_progress(
        &config,
        &ProgressEvent::new(
            &run_id,
            ProgressEventType::RunStarted,
            "pstd metadata extraction started",
        ),
    )?;

    let metadata = match extract_metadata(&input_display, &run_id, &pst_id) {
        Ok(value) => value,
        Err(reason) if config.continue_on_error => fallback_metadata(&run_id, &pst_id, &reason),
        Err(reason) => return Err(reason),
    };

    let mut folders = JsonlBuffer::new();
    for record in &metadata.folders {
        folders.write_record(record)?;
    }

    let mut folder_inventory = JsonlBuffer::new();
    for record in &metadata.folder_inventory {
        folder_inventory.write_record(record)?;
    }

    let mut messages = JsonlBuffer::new();
    for record in &metadata.messages {
        messages.write_record(record)?;
    }

    let mut recipients = JsonlBuffer::new();
    for record in &metadata.recipients {
        recipients.write_record(record)?;
    }

    let mut message_references = JsonlBuffer::new();
    for record in &metadata.message_references {
        message_references.write_record(record)?;
    }

    let mut bodies = JsonlBuffer::new();
    for record in &metadata.bodies {
        bodies.write_record(record)?;
    }

    let mut attachments = JsonlBuffer::new();
    for record in &metadata.attachments {
        attachments.write_record(record)?;
    }

    let mut manifest = JsonlBuffer::new();
    for record in &metadata.manifest {
        manifest.write_record(record)?;
    }

    let mut issues = JsonlBuffer::new();
    for record in &metadata.issues {
        issues.write_record(record)?;
    }

    let run_config_json = serde_json::to_vec_pretty(&serde_json::json!({
        "tool": "pstd",
        "archive_contract_version": 1,
        "input": input_display,
        "manifest_only": config.manifest_only,
        "profile": config.profile,
        "metadata_status": metadata.status,
    }))?;

    let archives_dir = config.output.join("archives");
    let mut tar = TarShardWriter::new(&archives_dir, &pst_id, config.tar_shard_size_bytes())?;
    tar.append_bytes(&["_pstfast", "run_config.json"], &run_config_json)?;
    tar.append_bytes(&["data", "folders.jsonl"], &folders.into_bytes())?;
    tar.append_bytes(&["data", "messages.jsonl"], &messages.into_bytes())?;
    tar.append_bytes(&["data", "recipients.jsonl"], &recipients.into_bytes())?;
    tar.append_bytes(
        &["data", "message_references.jsonl"],
        &message_references.into_bytes(),
    )?;
    tar.append_bytes(&["data", "bodies.jsonl"], &bodies.into_bytes())?;
    tar.append_bytes(&["data", "attachments.jsonl"], &attachments.into_bytes())?;
    for payload in &metadata.body_payloads {
        append_archive_payload(&mut tar, &payload.record.archive_path, &payload.bytes)?;
    }
    for payload in &metadata.attachment_payloads {
        append_archive_payload(&mut tar, &payload.record.archive_path, &payload.bytes)?;
    }
    tar.append_bytes(
        &["_pstfast", "folder_inventory.jsonl"],
        &folder_inventory.into_bytes(),
    )?;
    tar.append_bytes(&["_pstfast", "errors.jsonl"], &issues.into_bytes())?;
    tar.append_bytes(&["_pstfast", "manifest.jsonl"], &manifest.into_bytes())?;

    let finished = chrono::Utc::now();
    let mut summary = ExtractionSummary {
        run_id: run_id.clone(),
        pst_id: pst_id.clone(),
        source_pst_path: config.input.display().to_string(),
        started_at: started.to_rfc3339(),
        finished_at: Some(finished.to_rfc3339()),
        duration_seconds: Some(timer.elapsed().as_secs_f64()),
        folders_discovered: metadata.folders_discovered,
        messages_discovered: metadata.messages_discovered,
        messages_extracted: metadata.messages_extracted,
        messages_not_extracted: metadata
            .messages_discovered
            .saturating_sub(metadata.messages_extracted),
        attachments_extracted: metadata.attachment_payloads.len() as u64,
        attachments_not_extracted: metadata
            .attachments
            .len()
            .saturating_sub(metadata.attachment_payloads.len())
            as u64,
        bytes_read: 0,
        bytes_written: 0,
        tar_shards_written: 0,
        status: metadata.status.clone(),
    };

    tar.append_bytes(
        &["_pstfast", "summary.json"],
        &serde_json::to_vec_pretty(&summary)?,
    )?;
    let shards = tar.finish()?;
    summary.tar_shards_written = shards.len() as u64;
    summary.bytes_written = shards.iter().map(|s| s.bytes_written_estimate).sum();
    fs::write(
        config.output.join("run_summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;

    write_root_progress(
        &config,
        &ProgressEvent::new(
            &run_id,
            ProgressEventType::RunFinished,
            "pstd metadata extraction finished",
        ),
    )?;
    Ok(summary)
}

fn validate_config(config: &ExtractConfig) -> PstdResult<()> {
    if config.archive_format != "tar" {
        return Err(PstdError::InvalidConfig(
            "archive-format must be tar".to_string(),
        ));
    }
    if config.data_format != "jsonl" {
        return Err(PstdError::InvalidConfig(
            "data-format must be jsonl".to_string(),
        ));
    }
    Ok(())
}

fn append_archive_payload(
    tar: &mut TarShardWriter,
    archive_path: &str,
    bytes: &[u8],
) -> PstdResult<()> {
    let parts = archive_path.split('/').collect::<Vec<_>>();
    tar.append_bytes(&parts, bytes)
}

fn write_root_progress(config: &ExtractConfig, event: &ProgressEvent) -> PstdResult<()> {
    fs::create_dir_all(&config.output)?;
    let path = config.output.join("progress.jsonl");
    let line = event.to_json_line()?;
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(line.as_bytes())?;
    Ok(())
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
