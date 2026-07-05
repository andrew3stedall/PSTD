use std::collections::HashMap;

use crate::error::{PstdError, PstdResult, StatusRecord};
use crate::output::ids;
use crate::output::metadata::{
    AttachmentRecord, BodyRecord, FolderRecord, ManifestRecord, MessageRecord,
    MessageReferenceRecord, RecipientRecord,
};
use crate::pst::attachment_table::attachment_payloads_from_subnode_blocks;
use crate::pst::attachments::{unavailable_attachment_record, AttachmentPayload};
use crate::pst::bbt::BbtIndex;
use crate::pst::compatibility::{
    decoder_backlog_from_triage_records, decoder_backlog_review_summary,
    decoder_issue_candidates_from_backlog, select_decoder_candidates_for_implementation,
    triage_observed_attachment_layouts, CompatibilityTriageRecord, DecoderBacklogItem,
    DecoderBacklogReviewSummary, DecoderCandidateSelection, DecoderIssueCandidate,
};
use crate::pst::folder_tree::{
    folder_from_nbt_candidate, is_folder_candidate, root_folder_from_header, FolderInventoryRecord,
};
use crate::pst::header::PstHeader;
use crate::pst::limits::ParserLimits;
use crate::pst::message_metadata::{message_from_properties, status_row};
use crate::pst::message_table::{classify_message_candidate, discover_message_tables};
use crate::pst::messages::{
    body_coverage_report, body_payloads_from_properties, unavailable_body_record, BodyPayload,
};
use crate::pst::nbt::{NbtEntry, NbtIndex};
use crate::pst::node_payload::load_node_property_context;
use crate::pst::reader::PstByteReader;
use crate::pst::subnodes::{
    load_recursive_subnode_blocks, subnode_decode_plans, subnode_references_from_index,
    SubnodeReference,
};

#[derive(Debug, Clone)]
pub struct MetadataExtractionOutput {
    pub folders: Vec<FolderRecord>,
    pub folder_inventory: Vec<FolderInventoryRecord>,
    pub messages: Vec<MessageRecord>,
    pub recipients: Vec<RecipientRecord>,
    pub message_references: Vec<MessageReferenceRecord>,
    pub bodies: Vec<BodyRecord>,
    pub body_payloads: Vec<BodyPayload>,
    pub attachments: Vec<AttachmentRecord>,
    pub attachment_payloads: Vec<AttachmentPayload>,
    pub compatibility_triage: Vec<CompatibilityTriageRecord>,
    pub decoder_backlog: Vec<DecoderBacklogItem>,
    pub decoder_backlog_review: Vec<DecoderBacklogReviewSummary>,
    pub decoder_issue_candidates: Vec<DecoderIssueCandidate>,
    pub decoder_candidate_selection: Vec<DecoderCandidateSelection>,
    pub manifest: Vec<ManifestRecord>,
    pub issues: Vec<StatusRecord>,
    pub folders_discovered: u64,
    pub messages_discovered: u64,
    pub messages_extracted: u64,
    pub status: String,
}

#[derive(Debug, Clone)]
struct FolderDiscoveryOutput {
    folders: Vec<FolderRecord>,
    inventory: Vec<FolderInventoryRecord>,
    folder_key_by_node_identity: HashMap<String, String>,
    issues: Vec<StatusRecord>,
    candidate_count: usize,
    property_loaded_count: usize,
    property_unavailable_count: usize,
    status: String,
}

pub fn extract_metadata(
    input_path: &str,
    run_id: &str,
    pst_id: &str,
) -> PstdResult<MetadataExtractionOutput> {
    let reader = PstByteReader::open(input_path)?;
    let header = PstHeader::parse(&reader)?;
    let limits = ParserLimits::default();
    let bbt = BbtIndex::load_root_with_limits(&reader, header.roots.bbt_root, limits)?;
    let nbt = NbtIndex::load_root_with_limits(&reader, header.roots.nbt_root, limits)?;

    let (mut root_folder, mut root_inventory) = root_folder_from_header(pst_id, &header);
    let mut messages = Vec::new();
    let mut issues = Vec::new();
    let recipients: Vec<RecipientRecord> = Vec::new();
    let message_references: Vec<MessageReferenceRecord> = Vec::new();
    let mut bodies: Vec<BodyRecord> = Vec::new();
    let mut body_payloads: Vec<BodyPayload> = Vec::new();
    let mut attachments: Vec<AttachmentRecord> = Vec::new();
    let mut attachment_payloads: Vec<AttachmentPayload> = Vec::new();
    let mut compatibility_triage: Vec<CompatibilityTriageRecord> = Vec::new();
    let mut subnode_decode_attempts = 0usize;
    let mut subnode_decoded_blocks = 0usize;
    let mut subnode_child_references = 0usize;
    let mut subnode_recursive_child_decodes = 0usize;
    let mut subnode_unsupported_layouts = 0usize;
    let mut attachment_table_parse_errors = 0usize;
    let mut fixture_backed_decoder_hits = 0usize;
    let mut compatibility_follow_up_count = 0usize;
    let mut pq6_property_loaded_messages = 0usize;
    let mut pq6_property_unavailable_messages = 0usize;
    let mut pq6_selected_property_count = 0usize;
    let mut pq6_unknown_property_count = 0usize;
    let mut pq6_decode_error_count = 0usize;
    let mut pq6_body_supported_property_messages = 0usize;
    let mut pq6_body_payload_messages = 0usize;
    let mut pq6_body_payload_records = 0usize;
    let mut pq6_body_fallback_records = 0usize;
    let mut pq6_text_body_property_messages = 0usize;
    let mut pq6_html_body_property_messages = 0usize;
    let mut pq6_rtf_body_property_messages = 0usize;

    let subnode_report = subnode_references_from_index(&nbt);
    let subnode_plans = subnode_decode_plans(&subnode_report.references, limits);

    let metadata_status = if nbt.entries.is_empty() {
        "metadata_root_only".to_string()
    } else {
        "metadata_candidates_from_message_nodes".to_string()
    };

    let folder_discovery = discover_folder_hierarchy(
        &reader,
        &bbt,
        &nbt,
        limits,
        run_id,
        pst_id,
        &root_folder.folder_key,
    );
    issues.extend(folder_discovery.issues.clone());
    let discovered_child_folders = folder_discovery.folders.len() as u64;
    root_folder.child_folder_count = Some(discovered_child_folders);
    root_inventory.child_folder_count = Some(discovered_child_folders);
    root_folder.status = format!("{}; {}", root_folder.status, folder_discovery.status);
    root_inventory.inventory_status = format!(
        "{}; {}",
        root_inventory.inventory_status, folder_discovery.status
    );
    let mut folders = Vec::with_capacity(1 + folder_discovery.folders.len());
    folders.push(root_folder.clone());
    folders.extend(folder_discovery.folders.clone());
    let mut folder_inventory = Vec::with_capacity(1 + folder_discovery.inventory.len());
    folder_inventory.push(root_inventory.clone());
    folder_inventory.extend(folder_discovery.inventory.clone());

    let message_table_discovery =
        discover_message_tables(&nbt, &folder_discovery.folder_key_by_node_identity);
    let message_membership_status = message_table_discovery.message_membership_status();

    if !nbt.entries.is_empty() {
        let non_message_entries = nbt
            .entries
            .len()
            .saturating_sub(message_table_discovery.message_candidate_count());
        if non_message_entries > 0 {
            issues.push(StatusRecord::info(
                run_id,
                "pq5_non_message_nbt_entries_excluded",
                format!(
                    "Excluded {non_message_entries} decoded NBT entries from message output because they are not normal or associated message nodes."
                ),
            ));
        }
        issues.push(StatusRecord::info(
            run_id,
            "pq5_message_membership_status",
            format!(
                "Message table discovery status: {}; {}",
                message_table_discovery.status, message_membership_status
            ),
        ));
    }

    if nbt.entries.is_empty() {
        messages.push(status_row(
            run_id,
            pst_id,
            &root_folder.folder_key,
            &root_folder.folder_path,
            &metadata_status,
        ));
        issues.push(StatusRecord::info(
            run_id,
            "m3_metadata_root_only",
            "Folder root was emitted, but node index entries were unavailable for message metadata.",
        ));
    } else {
        for entry in message_table_discovery.message_candidates.iter().take(1000) {
            let message_key = ids::message_key(pst_id, &node_identity(entry));
            let candidate_status = classify_message_candidate(entry)
                .map(|candidate| {
                    format!(
                        "{}; {}",
                        candidate.membership_status, message_membership_status
                    )
                })
                .unwrap_or_else(|| message_membership_status.clone());
            match load_node_property_context(&reader, &bbt, entry, limits) {
                Ok(loaded) => {
                    pq6_property_loaded_messages += 1;
                    pq6_selected_property_count += loaded.property_report.selected_property_count;
                    pq6_unknown_property_count += loaded.property_report.unknown_property_count;
                    pq6_decode_error_count += loaded.property_report.decode_error_count;

                    let mut message = message_from_properties(
                        run_id,
                        pst_id,
                        &root_folder.folder_key,
                        &root_folder.folder_path,
                        entry.node_id,
                        &loaded.properties,
                    );
                    let loaded_body_payloads =
                        body_payloads_from_properties(&message.message_key, &loaded.properties);
                    let body_report = body_coverage_report(&loaded.properties, &loaded_body_payloads);
                    if body_report.supported_body_property_count > 0 {
                        pq6_body_supported_property_messages += 1;
                    }
                    if body_report.text_property_present {
                        pq6_text_body_property_messages += 1;
                    }
                    if body_report.html_property_present {
                        pq6_html_body_property_messages += 1;
                    }
                    if body_report.rtf_property_present {
                        pq6_rtf_body_property_messages += 1;
                    }
                    pq6_body_payload_records += body_report.extracted_payload_count;
                    pq6_body_fallback_records += body_report.fallback_record_count;

                    if loaded_body_payloads.is_empty() {
                        message.body_status = body_report.status.clone();
                        bodies.push(unavailable_body_record(
                            &message.message_key,
                            "text",
                            &body_report.status,
                        ));
                    } else {
                        pq6_body_payload_messages += 1;
                        message.has_text_body = loaded_body_payloads
                            .iter()
                            .any(|payload| payload.record.body_type == "text");
                        message.has_html_body = loaded_body_payloads
                            .iter()
                            .any(|payload| payload.record.body_type == "html");
                        message.body_status = body_report.status;
                        message.extraction_status = "metadata_and_payload".to_string();
                        for payload in loaded_body_payloads {
                            bodies.push(payload.record.clone());
                            body_payloads.push(payload);
                        }
                    }

                    if message.has_attachments {
                        if let Some(reference) =
                            subnode_reference_for_entry(&subnode_report.references, entry)
                        {
                            subnode_decode_attempts += 1;
                            let loaded_subnodes =
                                load_recursive_subnode_blocks(&reader, &bbt, reference, 1, limits);
                            subnode_decoded_blocks += loaded_subnodes.report.decoded_block_count;
                            subnode_child_references +=
                                loaded_subnodes.report.recursive_child_reference_count;
                            subnode_recursive_child_decodes +=
                                loaded_subnodes.report.recursive_child_decode_count;
                            subnode_unsupported_layouts +=
                                loaded_subnodes.layout_report.unsupported_layout_count;
                            let (
                                mut loaded_attachments,
                                mut unavailable_attachment_records,
                                attachment_report,
                            ) = attachment_payloads_from_subnode_blocks(
                                &message.message_key,
                                &loaded_subnodes.payloads,
                            );
                            attachment_table_parse_errors += attachment_report.parse_error_count;
                            let triage_report = triage_observed_attachment_layouts(
                                &loaded_subnodes.layout_report,
                                &attachment_report,
                            );
                            fixture_backed_decoder_hits +=
                                triage_report.fixture_backed_decoder_count;
                            compatibility_follow_up_count += triage_report.follow_up_issue_count;
                            let triage_status = triage_report.status.clone();
                            compatibility_triage.push(CompatibilityTriageRecord::from_report(
                                run_id,
                                pst_id,
                                &message.message_key,
                                message.message_node_id.clone(),
                                triage_report,
                            ));

                            let attachment_record_count =
                                loaded_attachments.len() + unavailable_attachment_records.len();

                            if loaded_attachments.is_empty() {
                                let status = format!(
                                    "{}; {}; {}; {}",
                                    loaded_subnodes.report.status,
                                    loaded_subnodes.layout_report.status,
                                    attachment_report.status,
                                    triage_status
                                );
                                message.attachment_status = status.clone();
                                message.attachment_count = attachment_record_count as u64;
                                if unavailable_attachment_records.is_empty() {
                                    attachments.push(unavailable_attachment_record(
                                        &message.message_key,
                                        0,
                                        None,
                                        &status,
                                    ));
                                } else {
                                    attachments.append(&mut unavailable_attachment_records);
                                }
                                issues.push(StatusRecord::info(
                                    run_id,
                                    "m16_attachment_layout_triage",
                                    format!(
                                        "Attachment compatibility triage for node_{:x}: {status}",
                                        entry.node_id.0
                                    ),
                                ));
                            } else {
                                message.attachment_count = attachment_record_count as u64;
                                message.attachment_status = format!(
                                    "{}; {}; {}",
                                    loaded_subnodes.report.status,
                                    attachment_report.status,
                                    triage_status
                                );
                                message.extraction_status = "metadata_and_payload".to_string();
                                for payload in loaded_attachments.drain(..) {
                                    attachments.push(payload.record.clone());
                                    attachment_payloads.push(payload);
                                }
                                attachments.append(&mut unavailable_attachment_records);
                            }
                        } else {
                            message.attachment_status =
                                "attachment_subnode_reference_absent".to_string();
                            attachments.push(unavailable_attachment_record(
                                &message.message_key,
                                0,
                                None,
                                "attachment_subnode_reference_absent",
                            ));
                        }
                    } else {
                        message.attachment_status =
                            "attachment_payload_property_absent".to_string();
                    }

                    message.metadata_status = format!(
                        "property_context_loaded; property_count={}; selected_properties={}; unknown_properties={}; decode_errors={}; pq5_status={}",
                        loaded.property_report.parsed_property_count,
                        loaded.property_report.selected_property_count,
                        loaded.property_report.unknown_property_count,
                        loaded.property_report.decode_error_count,
                        candidate_status
                    );
                    messages.push(message);
                }
                Err(reason) => {
                    pq6_property_unavailable_messages += 1;
                    pq6_body_fallback_records += 1;
                    messages.push(candidate_message(
                        run_id,
                        pst_id,
                        &root_folder.folder_key,
                        &root_folder.folder_path,
                        entry,
                        &candidate_status,
                    ));
                    bodies.push(unavailable_body_record(
                        &message_key,
                        "text",
                        "node_property_context_unavailable",
                    ));
                    issues.push(StatusRecord::info(
                        run_id,
                        "m11_node_payload_unavailable",
                        format!(
                            "Node payload status for node_{:x}: {reason}",
                            entry.node_id.0
                        ),
                    ));
                }
            }
        }
    }

    let pq6_status = format!(
        "property_body_coverage; property_loaded_messages={pq6_property_loaded_messages}; property_unavailable_messages={pq6_property_unavailable_messages}; selected_properties={pq6_selected_property_count}; unknown_properties={pq6_unknown_property_count}; decode_errors={pq6_decode_error_count}; body_supported_property_messages={pq6_body_supported_property_messages}; body_payload_messages={pq6_body_payload_messages}; body_payload_records={pq6_body_payload_records}; body_fallback_records={pq6_body_fallback_records}; text_body_property_messages={pq6_text_body_property_messages}; html_body_property_messages={pq6_html_body_property_messages}; rtf_body_property_messages={pq6_rtf_body_property_messages}"
    );
    if message_table_discovery.message_candidate_count() > 0 {
        issues.push(StatusRecord::info(
            run_id,
            "pq6_property_body_coverage",
            format!("PQ6 property/body coverage status: {pq6_status}"),
        ));
    }

    let decoder_backlog = decoder_backlog_from_triage_records(&compatibility_triage);
    let decoder_backlog_review = vec![decoder_backlog_review_summary(
        run_id,
        pst_id,
        &decoder_backlog,
    )];
    let decoder_issue_candidates =
        decoder_issue_candidates_from_backlog(run_id, pst_id, &decoder_backlog);
    let decoder_candidate_selection =
        select_decoder_candidates_for_implementation(run_id, pst_id, &decoder_issue_candidates);

    let mut manifest = base_manifest(
        run_id,
        pst_id,
        &root_folder.folder_key,
        &metadata_status,
        issues.len() as u64,
    );
    for payload in &body_payloads {
        manifest.push(ManifestRecord {
            run_id: run_id.to_string(),
            pst_id: pst_id.to_string(),
            message_key: Some(payload.record.message_key.clone()),
            folder_key: None,
            artefact_type: format!("body_{}", payload.record.body_type),
            archive_path: payload.record.archive_path.clone(),
            sha256: Some(payload.record.sha256.clone()),
            size_bytes: Some(payload.record.size_bytes),
            status: payload.record.status.clone(),
            issue_count: 0,
        });
    }
    for payload in &attachment_payloads {
        manifest.push(ManifestRecord {
            run_id: run_id.to_string(),
            pst_id: pst_id.to_string(),
            message_key: Some(payload.record.message_key.clone()),
            folder_key: None,
            artefact_type: "attachment".to_string(),
            archive_path: payload.record.archive_path.clone(),
            sha256: Some(payload.record.sha256.clone()),
            size_bytes: Some(payload.record.size_bytes),
            status: payload.record.extraction_status.clone(),
            issue_count: 0,
        });
    }

    let folders_discovered = folders.len() as u64;
    let messages_discovered = message_table_discovery.message_candidate_count() as u64;
    let messages_extracted = messages.len() as u64;
    let backlog_review_status = decoder_backlog_review[0].review_status.clone();
    let selected_candidate_count = decoder_candidate_selection
        .iter()
        .filter(|selection| selection.selection_status == "selected_for_next_planning")
        .count();
    let status = format!(
        "{}; bbt_status={}; nbt_status={}; pq4_status={}; pq4_folder_candidates={}; pq4_folder_property_loaded={}; pq4_folder_property_unavailable={}; pq5_status={}; pq5_message_candidates={}; pq5_table_candidates={}; pq5_linked_tables={}; pq5_unlinked_tables={}; pq6_status={}; folders_discovered={}; m4_status=recipient_threading_available; m5_status=body_attachment_outputs_available; m10_status=payload_wiring_available; m11_status=extraction_path_integration; m12_status=attachment_subnode_integration; m14_status=recursive_subnode_layout_exploration; m15_status=compatibility_triage_available; m16_status=fixture_backed_decoder_expansion; m17_status=decoder_backlog_reporting; m18_status=decoder_backlog_review_workflow; m19_status=focused_candidate_selection; m23_status=attachment_metadata_fidelity; body_payloads={}; attachment_payloads={}; attachment_records={}; subnode_references={}; subnode_decode_plans={}; subnode_decode_attempts={}; subnode_decoded_blocks={}; subnode_child_references={}; subnode_recursive_child_decodes={}; subnode_unsupported_layouts={}; attachment_table_parse_errors={}; fixture_backed_decoder_hits={}; compatibility_triage_records={}; compatibility_follow_ups={}; decoder_backlog_items={}; decoder_issue_candidates={}; decoder_candidate_selection={}; selected_decoder_candidates={}; decoder_backlog_review_status={}",
        metadata_status,
        bbt.status,
        nbt.status,
        folder_discovery.status,
        folder_discovery.candidate_count,
        folder_discovery.property_loaded_count,
        folder_discovery.property_unavailable_count,
        message_membership_status,
        message_table_discovery.message_candidate_count(),
        message_table_discovery.table_candidates.len(),
        message_table_discovery.linked_table_count,
        message_table_discovery.unlinked_table_count,
        pq6_status,
        folders_discovered,
        body_payloads.len(),
        attachment_payloads.len(),
        attachments.len(),
        subnode_report.subnode_reference_count,
        subnode_plans.len(),
        subnode_decode_attempts,
        subnode_decoded_blocks,
        subnode_child_references,
        subnode_recursive_child_decodes,
        subnode_unsupported_layouts,
        attachment_table_parse_errors,
        fixture_backed_decoder_hits,
        compatibility_triage.len(),
        compatibility_follow_up_count,
        decoder_backlog.len(),
        decoder_issue_candidates.len(),
        decoder_candidate_selection.len(),
        selected_candidate_count,
        backlog_review_status
    );

    Ok(MetadataExtractionOutput {
        folders,
        folder_inventory,
        messages,
        recipients,
        message_references,
        bodies,
        body_payloads,
        attachments,
        attachment_payloads,
        compatibility_triage,
        decoder_backlog,
        decoder_backlog_review,
        decoder_issue_candidates,
        decoder_candidate_selection,
        manifest,
        issues,
        folders_discovered,
        messages_discovered,
        messages_extracted,
        status,
    })
}

fn discover_folder_hierarchy(
    reader: &PstByteReader,
    bbt: &BbtIndex,
    nbt: &NbtIndex,
    limits: ParserLimits,
    run_id: &str,
    pst_id: &str,
    root_folder_key: &str,
) -> FolderDiscoveryOutput {
    let mut folders = Vec::new();
    let mut inventory = Vec::new();
    let mut folder_key_by_node_identity = HashMap::new();
    let mut issues = Vec::new();
    let mut candidate_count = 0usize;
    let mut property_loaded_count = 0usize;
    let mut property_unavailable_count = 0usize;

    for entry in nbt
        .entries
        .iter()
        .filter(|entry| is_folder_candidate(entry))
        .take(1000)
    {
        candidate_count += 1;
        match load_node_property_context(reader, bbt, entry, limits) {
            Ok(loaded) => {
                property_loaded_count += 1;
                let (folder, record) = folder_from_nbt_candidate(
                    pst_id,
                    entry,
                    Some(root_folder_key.to_string()),
                    Some(&loaded.properties),
                );
                if let Some(node_identity) = &folder.folder_node_id {
                    folder_key_by_node_identity
                        .insert(node_identity.clone(), folder.folder_key.clone());
                }
                folders.push(folder);
                inventory.push(record);
            }
            Err(reason) => {
                property_unavailable_count += 1;
                let (folder, record) = folder_from_nbt_candidate(
                    pst_id,
                    entry,
                    Some(root_folder_key.to_string()),
                    None,
                );
                if let Some(node_identity) = &folder.folder_node_id {
                    folder_key_by_node_identity
                        .insert(node_identity.clone(), folder.folder_key.clone());
                }
                folders.push(folder);
                inventory.push(record);
                issues.push(StatusRecord::info(
                    run_id,
                    "pq4_folder_property_context_unavailable",
                    format!(
                        "Folder candidate property context unavailable for node_{:x}: {reason}",
                        entry.node_id.0
                    ),
                ));
            }
        }
    }

    let status = if candidate_count == 0 {
        "root_only_no_decoded_folder_candidates".to_string()
    } else {
        format!(
            "decoded_folder_candidates; candidates={candidate_count}; property_loaded={property_loaded_count}; property_unavailable={property_unavailable_count}"
        )
    };

    FolderDiscoveryOutput {
        folders,
        inventory,
        folder_key_by_node_identity,
        issues,
        candidate_count,
        property_loaded_count,
        property_unavailable_count,
        status,
    }
}

fn node_identity(entry: &NbtEntry) -> String {
    format!("node_{:x}", entry.node_id.0)
}

fn subnode_reference_for_entry<'a>(
    references: &'a [SubnodeReference],
    entry: &NbtEntry,
) -> Option<&'a SubnodeReference> {
    references
        .iter()
        .find(|reference| reference.node_id == entry.node_id)
}

fn candidate_message(
    run_id: &str,
    pst_id: &str,
    folder_key: &str,
    folder_path: &str,
    entry: &NbtEntry,
    membership_status: &str,
) -> MessageRecord {
    let message_identity = node_identity(entry);
    MessageRecord {
        run_id: run_id.to_string(),
        pst_id: pst_id.to_string(),
        folder_key: folder_key.to_string(),
        message_key: ids::message_key(pst_id, &message_identity),
        message_node_id: Some(message_identity),
        folder_path: folder_path.to_string(),
        item_type: "message_metadata_candidate".to_string(),
        subject: None,
        sender_name: None,
        sender_email: None,
        sender_raw_address: None,
        sender_address_type: None,
        sent_at: None,
        received_at: None,
        created_at: None,
        modified_at: None,
        transport_message_headers: None,
        internet_message_id: None,
        in_reply_to_id: None,
        conversation_index: None,
        conversation_topic: None,
        normalized_subject: None,
        has_text_body: false,
        has_html_body: false,
        has_attachments: false,
        attachment_count: 0,
        metadata_status: format!(
            "node_property_context_unavailable; pq5_status={membership_status}"
        ),
        threading_status: "threading_metadata_not_attempted".to_string(),
        body_status: "node_property_context_unavailable".to_string(),
        attachment_status: "node_property_context_unavailable".to_string(),
        extraction_status: "metadata_only_candidate".to_string(),
    }
}

fn base_manifest(
    run_id: &str,
    pst_id: &str,
    root_folder_key: &str,
    metadata_status: &str,
    issue_count: u64,
) -> Vec<ManifestRecord> {
    vec![
        ManifestRecord {
            run_id: run_id.to_string(),
            pst_id: pst_id.to_string(),
            message_key: None,
            folder_key: Some(root_folder_key.to_string()),
            artefact_type: "folders".to_string(),
            archive_path: "data/folders.jsonl".to_string(),
            sha256: None,
            size_bytes: None,
            status: metadata_status.to_string(),
            issue_count,
        },
        ManifestRecord {
            run_id: run_id.to_string(),
            pst_id: pst_id.to_string(),
            message_key: None,
            folder_key: Some(root_folder_key.to_string()),
            artefact_type: "messages".to_string(),
            archive_path: "data/messages.jsonl".to_string(),
            sha256: None,
            size_bytes: None,
            status: metadata_status.to_string(),
            issue_count,
        },
        ManifestRecord {
            run_id: run_id.to_string(),
            pst_id: pst_id.to_string(),
            message_key: None,
            folder_key: None,
            artefact_type: "recipients".to_string(),
            archive_path: "data/recipients.jsonl".to_string(),
            sha256: None,
            size_bytes: None,
            status: "m4_recipient_output_available".to_string(),
            issue_count: 0,
        },
        ManifestRecord {
            run_id: run_id.to_string(),
            pst_id: pst_id.to_string(),
            message_key: None,
            folder_key: None,
            artefact_type: "message_references".to_string(),
            archive_path: "data/message_references.jsonl".to_string(),
            sha256: None,
            size_bytes: None,
            status: "m4_reference_output_available".to_string(),
            issue_count: 0,
        },
        ManifestRecord {
            run_id: run_id.to_string(),
            pst_id: pst_id.to_string(),
            message_key: None,
            folder_key: None,
            artefact_type: "bodies".to_string(),
            archive_path: "data/bodies.jsonl".to_string(),
            sha256: None,
            size_bytes: None,
            status: "m11_body_output_integrated".to_string(),
            issue_count: 0,
        },
        ManifestRecord {
            run_id: run_id.to_string(),
            pst_id: pst_id.to_string(),
            message_key: None,
            folder_key: None,
            artefact_type: "attachments".to_string(),
            archive_path: "data/attachments.jsonl".to_string(),
            sha256: None,
            size_bytes: None,
            status: "m23_attachment_metadata_fidelity_output_available".to_string(),
            issue_count: 0,
        },
        ManifestRecord {
            run_id: run_id.to_string(),
            pst_id: pst_id.to_string(),
            message_key: None,
            folder_key: None,
            artefact_type: "compatibility_triage".to_string(),
            archive_path: "data/compatibility_triage.jsonl".to_string(),
            sha256: None,
            size_bytes: None,
            status: "m16_compatibility_triage_output_available".to_string(),
            issue_count: 0,
        },
        ManifestRecord {
            run_id: run_id.to_string(),
            pst_id: pst_id.to_string(),
            message_key: None,
            folder_key: None,
            artefact_type: "decoder_backlog".to_string(),
            archive_path: "data/decoder_backlog.jsonl".to_string(),
            sha256: None,
            size_bytes: None,
            status: "m17_decoder_backlog_output_available".to_string(),
            issue_count: 0,
        },
        ManifestRecord {
            run_id: run_id.to_string(),
            pst_id: pst_id.to_string(),
            message_key: None,
            folder_key: None,
            artefact_type: "decoder_backlog_review".to_string(),
            archive_path: "data/decoder_backlog_review.jsonl".to_string(),
            sha256: None,
            size_bytes: None,
            status: "m18_decoder_backlog_review_output_available".to_string(),
            issue_count: 0,
        },
        ManifestRecord {
            run_id: run_id.to_string(),
            pst_id: pst_id.to_string(),
            message_key: None,
            folder_key: None,
            artefact_type: "decoder_issue_candidates".to_string(),
            archive_path: "data/decoder_issue_candidates.jsonl".to_string(),
            sha256: None,
            size_bytes: None,
            status: "m18_decoder_issue_candidates_output_available".to_string(),
            issue_count: 0,
        },
        ManifestRecord {
            run_id: run_id.to_string(),
            pst_id: pst_id.to_string(),
            message_key: None,
            folder_key: None,
            artefact_type: "decoder_candidate_selection".to_string(),
            archive_path: "data/decoder_candidate_selection.jsonl".to_string(),
            sha256: None,
            size_bytes: None,
            status: "m19_decoder_candidate_selection_output_available".to_string(),
            issue_count: 0,
        },
    ]
}

pub fn fallback_metadata(
    run_id: &str,
    pst_id: &str,
    error: &PstdError,
) -> MetadataExtractionOutput {
    let folder_key = ids::folder_key(pst_id, "metadata-fallback-root");
    let folder = FolderRecord {
        pst_id: pst_id.to_string(),
        folder_key: folder_key.clone(),
        parent_folder_key: None,
        folder_path: "/".to_string(),
        folder_name: "root".to_string(),
        folder_node_id: None,
        item_count_total: None,
        child_folder_count: None,
        status: "metadata_unavailable".to_string(),
    };
    let inventory = FolderInventoryRecord {
        pst_id: pst_id.to_string(),
        folder_key: folder_key.clone(),
        folder_path: "/".to_string(),
        item_count_total: None,
        item_count_email: None,
        item_count_unknown: None,
        child_folder_count: None,
        inventory_status: "metadata_unavailable".to_string(),
    };
    MetadataExtractionOutput {
        folders: vec![folder],
        folder_inventory: vec![inventory],
        messages: vec![status_row(
            run_id,
            pst_id,
            &folder_key,
            "/",
            "metadata_unavailable",
        )],
        recipients: Vec::new(),
        message_references: Vec::new(),
        bodies: Vec::new(),
        body_payloads: Vec::new(),
        attachments: Vec::new(),
        attachment_payloads: Vec::new(),
        compatibility_triage: Vec::new(),
        decoder_backlog: Vec::new(),
        decoder_backlog_review: Vec::new(),
        decoder_issue_candidates: Vec::new(),
        decoder_candidate_selection: Vec::new(),
        manifest: Vec::new(),
        issues: vec![StatusRecord::info(
            run_id,
            "m3_metadata_unavailable",
            format!("Metadata extraction unavailable: {error}"),
        )],
        folders_discovered: 0,
        messages_discovered: 0,
        messages_extracted: 0,
        status: "metadata_unavailable".to_string(),
    }
}
