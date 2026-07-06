use std::fs;
use std::time::Instant;

use sha2::{Digest, Sha256};

use crate::config::ExtractConfig;
use crate::engine::metadata::{extract_metadata, fallback_metadata};
use crate::error::{PstdError, PstdResult};
use crate::output::ids;
use crate::output::jsonl_writer::JsonlBuffer;
use crate::output::metadata::MessageRecord;
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
    let metadata_status = status_with_property_diagnostics(&metadata.status, &metadata.messages);

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

    let mut compatibility_triage = JsonlBuffer::new();
    for record in &metadata.compatibility_triage {
        compatibility_triage.write_record(record)?;
    }

    let mut decoder_backlog = JsonlBuffer::new();
    for record in &metadata.decoder_backlog {
        decoder_backlog.write_record(record)?;
    }

    let mut decoder_backlog_review = JsonlBuffer::new();
    for record in &metadata.decoder_backlog_review {
        decoder_backlog_review.write_record(record)?;
    }

    let mut decoder_issue_candidates = JsonlBuffer::new();
    for record in &metadata.decoder_issue_candidates {
        decoder_issue_candidates.write_record(record)?;
    }

    let mut decoder_candidate_selection = JsonlBuffer::new();
    for record in &metadata.decoder_candidate_selection {
        decoder_candidate_selection.write_record(record)?;
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
        "metadata_status": metadata_status.clone(),
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
    tar.append_bytes(
        &["data", "compatibility_triage.jsonl"],
        &compatibility_triage.into_bytes(),
    )?;
    tar.append_bytes(
        &["data", "decoder_backlog.jsonl"],
        &decoder_backlog.into_bytes(),
    )?;
    tar.append_bytes(
        &["data", "decoder_backlog_review.jsonl"],
        &decoder_backlog_review.into_bytes(),
    )?;
    tar.append_bytes(
        &["data", "decoder_issue_candidates.jsonl"],
        &decoder_issue_candidates.into_bytes(),
    )?;
    tar.append_bytes(
        &["data", "decoder_candidate_selection.jsonl"],
        &decoder_candidate_selection.into_bytes(),
    )?;
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
        status: metadata_status,
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

fn status_with_property_diagnostics(base_status: &str, messages: &[MessageRecord]) -> String {
    let plausible = messages
        .iter()
        .map(|message| status_counter(&message.extraction_status, "plausible"))
        .sum::<usize>();
    let suspicious = messages
        .iter()
        .map(|message| status_counter(&message.extraction_status, "suspicious"))
        .sum::<usize>();
    let byte_swapped_selected = messages
        .iter()
        .map(|message| status_counter(&message.extraction_status, "byte_swapped_selected"))
        .sum::<usize>();
    let heap_bth_contexts =
        status_contains_count(messages, "pq10_traversal=heap_bth_property_context");
    let legacy_flat_bth_contexts =
        status_contains_count(messages, "pq10_traversal=legacy_flat_bth_property_context");
    let unknown_traversal_contexts = status_contains_count(
        messages,
        "pq10_traversal=property_context_traversal_unknown",
    );
    let pq11_offset_heap_contexts = status_contains_count(
        messages,
        "pq10_traversal=heap_bth_property_context_at_offset_",
    );
    let pq11_candidate_not_found =
        status_contains_count(messages, "pq11_heap_probe=candidate_not_found");
    let pq11_candidate_heap_failed =
        status_contains_count(messages, "pq11_heap_probe=candidate_heap_failed");
    let pq11_candidate_bth_failed =
        status_contains_count(messages, "pq11_heap_probe=candidate_bth_failed");
    let pq12_no_signature =
        status_contains_count(messages, "pq12_boundary=no_signature_in_first_4096");
    let pq12_signature_without_page_map =
        status_contains_count(messages, "pq12_boundary=signature_without_valid_page_map");
    let pq12_candidate_heap_failed =
        status_contains_count(messages, "pq12_boundary=candidate_heap_failed");
    let pq12_candidate_bth_failed =
        status_contains_count(messages, "pq12_boundary=candidate_bth_failed");
    let pq13_subnode_references = status_counter(base_status, "subnode_references");
    let pq13_subnode_decode_plans = status_counter(base_status, "subnode_decode_plans");
    let pq13_subnode_decode_attempts = status_counter(base_status, "subnode_decode_attempts");

    let has_pq9_signal = plausible > 0 || suspicious > 0 || byte_swapped_selected > 0;
    let has_pq10_signal =
        heap_bth_contexts > 0 || legacy_flat_bth_contexts > 0 || unknown_traversal_contexts > 0;
    let has_pq11_signal = pq11_offset_heap_contexts > 0
        || pq11_candidate_not_found > 0
        || pq11_candidate_heap_failed > 0
        || pq11_candidate_bth_failed > 0;
    let has_pq12_signal = pq12_no_signature > 0
        || pq12_signature_without_page_map > 0
        || pq12_candidate_heap_failed > 0
        || pq12_candidate_bth_failed > 0;
    let has_pq13_signal = pq13_subnode_references > 0
        || pq13_subnode_decode_plans > 0
        || pq13_subnode_decode_attempts > 0;
    if !has_pq9_signal
        && !has_pq10_signal
        && !has_pq11_signal
        && !has_pq12_signal
        && !has_pq13_signal
    {
        return base_status.to_string();
    }

    let mut status = base_status.to_string();
    if has_pq9_signal {
        status.push_str(&format!(
            "; pq9_status=tag_shape_visible; pq9_plausible_property_tags={plausible}; pq9_suspicious_property_keys={suspicious}; pq9_byte_swapped_selected={byte_swapped_selected}; pq9_next_blocker={}",
            pq9_next_blocker(plausible, suspicious)
        ));
    }
    if has_pq10_signal {
        status.push_str(&format!(
            "; pq10_status=property_context_traversal_visible; pq10_heap_bth_contexts={heap_bth_contexts}; pq10_legacy_flat_bth_contexts={legacy_flat_bth_contexts}; pq10_unknown_traversal_contexts={unknown_traversal_contexts}; pq10_next_blocker={}",
            pq10_next_blocker(heap_bth_contexts, legacy_flat_bth_contexts, unknown_traversal_contexts)
        ));
    }
    if has_pq11_signal {
        status.push_str(&format!(
            "; pq11_status=heap_probe_visible; pq11_offset_heap_contexts={pq11_offset_heap_contexts}; pq11_candidate_not_found={pq11_candidate_not_found}; pq11_candidate_heap_failed={pq11_candidate_heap_failed}; pq11_candidate_bth_failed={pq11_candidate_bth_failed}; pq11_next_blocker={}",
            pq11_next_blocker(
                pq11_offset_heap_contexts,
                pq11_candidate_not_found,
                pq11_candidate_heap_failed,
                pq11_candidate_bth_failed,
            )
        ));
    }
    if has_pq12_signal {
        status.push_str(&format!(
            "; pq12_status=payload_boundary_visible; pq12_no_signature={pq12_no_signature}; pq12_signature_without_page_map={pq12_signature_without_page_map}; pq12_candidate_heap_failed={pq12_candidate_heap_failed}; pq12_candidate_bth_failed={pq12_candidate_bth_failed}; pq12_next_blocker={}",
            pq12_next_blocker(
                pq12_no_signature,
                pq12_signature_without_page_map,
                pq12_candidate_heap_failed,
                pq12_candidate_bth_failed,
            )
        ));
    }
    if has_pq13_signal {
        status.push_str(&format!(
            "; pq13_status=payload_source_visible; pq13_subnode_references={pq13_subnode_references}; pq13_subnode_decode_plans={pq13_subnode_decode_plans}; pq13_subnode_decode_attempts={pq13_subnode_decode_attempts}; pq13_next_blocker={}",
            pq13_next_blocker(
                pq13_subnode_references,
                pq13_subnode_decode_plans,
                pq13_subnode_decode_attempts,
            )
        ));
    }
    status
}

fn status_counter(status: &str, key: &str) -> usize {
    let colon = format!("{key}:");
    let equals = format!("{key}=");
    status
        .split(&colon)
        .nth(1)
        .or_else(|| status.split(&equals).nth(1))
        .and_then(|tail| tail.split([',', ';']).next())
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0)
}

fn status_contains_count(messages: &[MessageRecord], needle: &str) -> usize {
    messages
        .iter()
        .filter(|message| message.extraction_status.contains(needle))
        .count()
}

fn pq9_next_blocker(plausible: usize, suspicious: usize) -> &'static str {
    if suspicious > plausible {
        "heap_bth_layout_traversal"
    } else if plausible > 0 {
        "selected_mapi_dictionary_expansion"
    } else {
        "property_context_signal_absent"
    }
}

fn pq10_next_blocker(
    heap_bth_contexts: usize,
    legacy_flat_bth_contexts: usize,
    unknown_traversal_contexts: usize,
) -> &'static str {
    if legacy_flat_bth_contexts > 0 {
        "heap_hn_header_or_bth_root_detection"
    } else if unknown_traversal_contexts > 0 {
        "property_context_traversal_status_missing"
    } else if heap_bth_contexts > 0 {
        "heap_bth_index_or_external_hnid_resolution"
    } else {
        "property_context_traversal_signal_absent"
    }
}

fn pq11_next_blocker(
    offset_heap_contexts: usize,
    candidate_not_found: usize,
    candidate_heap_failed: usize,
    candidate_bth_failed: usize,
) -> &'static str {
    if offset_heap_contexts > 0 {
        "selected_property_or_external_hnid_resolution"
    } else if candidate_bth_failed > 0 {
        "heap_bth_root_or_index_decode"
    } else if candidate_heap_failed > 0 {
        "heap_page_map_or_allocation_decode"
    } else if candidate_not_found > 0 {
        "heap_signature_or_block_payload_prefix_detection"
    } else {
        "heap_probe_signal_absent"
    }
}

fn pq12_next_blocker(
    no_signature: usize,
    signature_without_page_map: usize,
    candidate_heap_failed: usize,
    candidate_bth_failed: usize,
) -> &'static str {
    if candidate_bth_failed > 0 {
        "heap_bth_root_or_index_decode"
    } else if candidate_heap_failed > 0 || signature_without_page_map > 0 {
        "payload_prefix_or_heap_page_map_decode"
    } else if no_signature > 0 {
        "payload_block_selection_or_subnode_resolution"
    } else {
        "payload_boundary_signal_absent"
    }
}

fn pq13_next_blocker(
    subnode_references: usize,
    subnode_decode_plans: usize,
    subnode_decode_attempts: usize,
) -> &'static str {
    if subnode_references > subnode_decode_attempts
        || subnode_decode_plans > subnode_decode_attempts
    {
        "message_subnode_payload_selection"
    } else if subnode_decode_attempts > 0 {
        "subnode_payload_interpretation"
    } else if subnode_references == 0 && subnode_decode_plans == 0 {
        "non_subnode_payload_source_selection"
    } else {
        "payload_source_signal_absent"
    }
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

#[cfg(test)]
mod tests {
    use super::{
        pq10_next_blocker, pq11_next_blocker, pq12_next_blocker, pq13_next_blocker,
        pq9_next_blocker, status_counter,
    };

    #[test]
    fn parses_pq9_status_counters() {
        let status = "metadata_only; pq9_tag_shape=plausible:3,suspicious:7,byte_swapped_selected:1; pq9_next_blocker=heap_bth_layout_traversal";
        assert_eq!(status_counter(status, "plausible"), 3);
        assert_eq!(status_counter(status, "suspicious"), 7);
        assert_eq!(status_counter(status, "byte_swapped_selected"), 1);
    }

    #[test]
    fn parses_equals_status_counters() {
        let status = "status; subnode_references=3; subnode_decode_plans=2";
        assert_eq!(status_counter(status, "subnode_references"), 3);
        assert_eq!(status_counter(status, "subnode_decode_plans"), 2);
    }

    #[test]
    fn chooses_pq9_next_blocker() {
        assert_eq!(pq9_next_blocker(2, 7), "heap_bth_layout_traversal");
        assert_eq!(pq9_next_blocker(7, 2), "selected_mapi_dictionary_expansion");
        assert_eq!(pq9_next_blocker(0, 0), "property_context_signal_absent");
    }

    #[test]
    fn chooses_pq10_next_blocker() {
        assert_eq!(
            pq10_next_blocker(0, 1, 0),
            "heap_hn_header_or_bth_root_detection"
        );
        assert_eq!(
            pq10_next_blocker(0, 0, 1),
            "property_context_traversal_status_missing"
        );
        assert_eq!(
            pq10_next_blocker(1, 0, 0),
            "heap_bth_index_or_external_hnid_resolution"
        );
        assert_eq!(
            pq10_next_blocker(0, 0, 0),
            "property_context_traversal_signal_absent"
        );
    }

    #[test]
    fn chooses_pq11_next_blocker() {
        assert_eq!(
            pq11_next_blocker(1, 0, 0, 0),
            "selected_property_or_external_hnid_resolution"
        );
        assert_eq!(
            pq11_next_blocker(0, 0, 0, 1),
            "heap_bth_root_or_index_decode"
        );
        assert_eq!(
            pq11_next_blocker(0, 0, 1, 0),
            "heap_page_map_or_allocation_decode"
        );
        assert_eq!(
            pq11_next_blocker(0, 1, 0, 0),
            "heap_signature_or_block_payload_prefix_detection"
        );
        assert_eq!(pq11_next_blocker(0, 0, 0, 0), "heap_probe_signal_absent");
    }

    #[test]
    fn chooses_pq12_next_blocker() {
        assert_eq!(
            pq12_next_blocker(0, 0, 0, 1),
            "heap_bth_root_or_index_decode"
        );
        assert_eq!(
            pq12_next_blocker(0, 1, 0, 0),
            "payload_prefix_or_heap_page_map_decode"
        );
        assert_eq!(
            pq12_next_blocker(0, 0, 1, 0),
            "payload_prefix_or_heap_page_map_decode"
        );
        assert_eq!(
            pq12_next_blocker(1, 0, 0, 0),
            "payload_block_selection_or_subnode_resolution"
        );
        assert_eq!(
            pq12_next_blocker(0, 0, 0, 0),
            "payload_boundary_signal_absent"
        );
    }

    #[test]
    fn chooses_pq13_next_blocker() {
        assert_eq!(
            pq13_next_blocker(3, 3, 0),
            "message_subnode_payload_selection"
        );
        assert_eq!(pq13_next_blocker(3, 3, 3), "subnode_payload_interpretation");
        assert_eq!(
            pq13_next_blocker(0, 0, 0),
            "non_subnode_payload_source_selection"
        );
    }
}
