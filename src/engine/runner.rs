use std::fs;
use std::time::Instant;

use sha2::{Digest, Sha256};

use crate::config::ExtractConfig;
use crate::error::{PstdError, PstdResult, StatusRecord};
use crate::output::ids;
use crate::output::jsonl_writer::JsonlBuffer;
use crate::output::metadata::{FolderRecord, ManifestRecord, MessageRecord};
use crate::output::summary::ExtractionSummary;
use crate::output::tar_writer::TarShardWriter;
use crate::progress::{ProgressEvent, ProgressEventType};

pub fn run_extract(config: ExtractConfig) -> PstdResult<ExtractionSummary> {
    validate_config(&config)?;
    fs::create_dir_all(&config.output)?;
    let archives_dir = config.output.join("archives");
    let logs_dir = config.output.join("logs");
    fs::create_dir_all(&archives_dir)?;
    fs::create_dir_all(&logs_dir)?;

    let started = chrono::Utc::now();
    let timer = Instant::now();
    let input_display = config.input.display().to_string();
    let run_id = ids::run_id(&input_display);
    let pst_id = ids::pst_id(&input_display);

    let start_event = ProgressEvent::new(&run_id, ProgressEventType::RunStarted, "pstd M1 placeholder run started");
    write_root_progress(&config, &start_event)?;

    let mut messages = JsonlBuffer::new();
    let mut folders = JsonlBuffer::new();
    let mut manifest = JsonlBuffer::new();
    let mut issues = JsonlBuffer::new();

    let folder_key = ids::folder_key(&pst_id, "placeholder-root");
    let message_key = ids::message_key(&pst_id, "placeholder-message");

    folders.write_record(&FolderRecord {
        pst_id: pst_id.clone(),
        folder_key: folder_key.clone(),
        parent_folder_key: None,
        folder_path: "/".to_string(),
        folder_name: "root".to_string(),
        folder_node_id: None,
        item_count_total: Some(0),
        child_folder_count: Some(0),
        status: "placeholder".to_string(),
    })?;

    messages.write_record(&MessageRecord {
        run_id: run_id.clone(),
        pst_id: pst_id.clone(),
        folder_key: folder_key.clone(),
        message_key: message_key.clone(),
        message_node_id: None,
        folder_path: "/".to_string(),
        item_type: "placeholder".to_string(),
        subject: Some("PSTD M1 placeholder record".to_string()),
        sender_name: None,
        sender_email: None,
        sender_raw_address: None,
        sender_address_type: None,
        sent_at: None,
        received_at: None,
        created_at: None,
        modified_at: None,
        internet_message_id: None,
        in_reply_to_id: None,
        conversation_index: None,
        conversation_topic: None,
        normalized_subject: None,
        has_text_body: false,
        has_html_body: false,
        has_attachments: false,
        attachment_count: 0,
        metadata_status: "placeholder".to_string(),
        threading_status: "placeholder".to_string(),
        body_status: "not_extracted".to_string(),
        attachment_status: "not_extracted".to_string(),
        extraction_status: "placeholder".to_string(),
    })?;

    issues.write_record(&StatusRecord::info(
        &run_id,
        "m1_placeholder",
        "PST parsing is intentionally deferred to a later milestone",
    ))?;

    let run_config_json = serde_json::to_vec_pretty(&serde_json::json!({
        "tool": "pstd",
        "archive_contract_version": 1,
        "input": input_display,
        "continue_on_error": config.continue_on_error,
        "manifest_only": config.manifest_only,
        "profile": config.profile,
    }))?;

    let mut tar = TarShardWriter::new(&archives_dir, &pst_id, config.tar_shard_size_bytes())?;
    tar.append_bytes(&["_pstfast", "run_config.json"], &run_config_json)?;
    tar.append_bytes(&["data", "folders.jsonl"], &folders.into_bytes())?;
    tar.append_bytes(&["data", "messages.jsonl"], &messages.into_bytes())?;
    tar.append_bytes(&["_pstfast", "errors.jsonl"], &issues.into_bytes())?;

    manifest.write_record(&ManifestRecord {
        run_id: run_id.clone(),
        pst_id: pst_id.clone(),
        message_key: Some(message_key),
        folder_key: Some(folder_key),
        artefact_type: "placeholder_message".to_string(),
        archive_path: "data/messages.jsonl".to_string(),
        sha256: None,
        size_bytes: None,
        status: "placeholder".to_string(),
        issue_count: 1,
    })?;
    tar.append_bytes(&["_pstfast", "manifest.jsonl"], &manifest.into_bytes())?;

    let finished = chrono::Utc::now();
    let duration = timer.elapsed().as_secs_f64();
    let mut summary = ExtractionSummary {
        run_id: run_id.clone(),
        pst_id: pst_id.clone(),
        source_pst_path: config.input.display().to_string(),
        started_at: started.to_rfc3339(),
        finished_at: Some(finished.to_rfc3339()),
        duration_seconds: Some(duration),
        folders_discovered: 1,
        messages_discovered: 0,
        messages_extracted: 0,
        messages_not_extracted: 0,
        attachments_extracted: 0,
        attachments_not_extracted: 0,
        bytes_read: 0,
        bytes_written: 0,
        tar_shards_written: 0,
        status: "placeholder_completed".to_string(),
    };

    let summary_json = serde_json::to_vec_pretty(&summary)?;
    tar.append_bytes(&["_pstfast", "summary.json"], &summary_json)?;
    let shards = tar.finish()?;
    summary.tar_shards_written = shards.len() as u64;
    summary.bytes_written = shards.iter().map(|s| s.bytes_written_estimate).sum();

    let root_summary = serde_json::to_vec_pretty(&summary)?;
    fs::write(config.output.join("run_summary.json"), root_summary)?;

    let finished_event = ProgressEvent::new(&run_id, ProgressEventType::RunFinished, "pstd M1 placeholder run finished");
    write_root_progress(&config, &finished_event)?;

    Ok(summary)
}

fn validate_config(config: &ExtractConfig) -> PstdResult<()> {
    if config.archive_format != "tar" {
        return Err(PstdError::InvalidConfig("M1 supports archive-format=tar only".to_string()));
    }
    if config.data_format != "jsonl" {
        return Err(PstdError::InvalidConfig("M1 supports data-format=jsonl only".to_string()));
    }
    Ok(())
}

fn write_root_progress(config: &ExtractConfig, event: &ProgressEvent) -> PstdResult<()> {
    fs::create_dir_all(&config.output)?;
    let path = config.output.join("progress.jsonl");
    let line = event.to_json_line()?;
    use std::io::Write;
    let mut file = fs::OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(line.as_bytes())?;
    Ok(())
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
