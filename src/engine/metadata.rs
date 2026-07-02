use crate::error::{PstdError, PstdResult, StatusRecord};
use crate::output::ids;
use crate::output::metadata::{
    AttachmentRecord, BodyRecord, FolderRecord, ManifestRecord, MessageRecord,
    MessageReferenceRecord, RecipientRecord,
};
use crate::pst::attachment_table::attachment_payloads_from_subnode_blocks;
use crate::pst::attachments::{unavailable_attachment_record, AttachmentPayload};
use crate::pst::bbt::BbtIndex;
use crate::pst::folder_tree::{root_folder_from_header, FolderInventoryRecord};
use crate::pst::header::PstHeader;
use crate::pst::limits::ParserLimits;
use crate::pst::message_metadata::{message_from_properties, status_row};
use crate::pst::messages::{body_payloads_from_properties, unavailable_body_record, BodyPayload};
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
    pub manifest: Vec<ManifestRecord>,
    pub issues: Vec<StatusRecord>,
    pub folders_discovered: u64,
    pub messages_discovered: u64,
    pub messages_extracted: u64,
    pub status: String,
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

    let (root_folder, root_inventory) = root_folder_from_header(pst_id, &header);
    let mut messages = Vec::new();
    let mut issues = Vec::new();
    let recipients: Vec<RecipientRecord> = Vec::new();
    let message_references: Vec<MessageReferenceRecord> = Vec::new();
    let mut bodies: Vec<BodyRecord> = Vec::new();
    let mut body_payloads: Vec<BodyPayload> = Vec::new();
    let mut attachments: Vec<AttachmentRecord> = Vec::new();
    let mut attachment_payloads: Vec<AttachmentPayload> = Vec::new();
    let mut subnode_decode_attempts = 0usize;
    let mut subnode_decoded_blocks = 0usize;
    let mut subnode_child_references = 0usize;
    let mut subnode_recursive_child_decodes = 0usize;
    let mut subnode_unsupported_layouts = 0usize;
    let mut attachment_table_parse_errors = 0usize;

    let subnode_report = subnode_references_from_index(&nbt);
    let subnode_plans = subnode_decode_plans(&subnode_report.references, limits);

    let metadata_status = if nbt.entries.is_empty() {
        "metadata_root_only".to_string()
    } else {
        "metadata_candidates_from_node_index".to_string()
    };

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
        for entry in nbt.entries.iter().take(1000) {
            let message_key = ids::message_key(pst_id, &node_identity(entry));
            match load_node_property_context(&reader, &bbt, entry, limits) {
                Ok(loaded) => {
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
                    if loaded_body_payloads.is_empty() {
                        message.body_status = "body_payload_property_absent".to_string();
                        bodies.push(unavailable_body_record(
                            &message.message_key,
                            "text",
                            "body_payload_property_absent",
                        ));
                    } else {
                        message.has_text_body = loaded_body_payloads
                            .iter()
                            .any(|payload| payload.record.body_type == "text");
                        message.has_html_body = loaded_body_payloads
                            .iter()
                            .any(|payload| payload.record.body_type == "html");
                        message.body_status = "body_payload_extracted".to_string();
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
                            let (mut loaded_attachments, attachment_report) =
                                attachment_payloads_from_subnode_blocks(
                                    &message.message_key,
                                    &loaded_subnodes.payloads,
                                );
                            attachment_table_parse_errors += attachment_report.parse_error_count;

                            if loaded_attachments.is_empty() {
                                let status = format!(
                                    "{}; {}; {}",
                                    loaded_subnodes.report.status,
                                    loaded_subnodes.layout_report.status,
                                    attachment_report.status
                                );
                                message.attachment_status = status.clone();
                                attachments.push(unavailable_attachment_record(
                                    &message.message_key,
                                    0,
                                    None,
                                    &status,
                                ));
                                issues.push(StatusRecord::info(
                                    run_id,
                                    "m14_attachment_layout_unavailable",
                                    format!(
                                        "Attachment subnode layout status for node_{:x}: {status}",
                                        entry.node_id.0
                                    ),
                                ));
                            } else {
                                message.attachment_count = loaded_attachments.len() as u64;
                                message.attachment_status = format!(
                                    "{}; {}",
                                    loaded_subnodes.report.status, attachment_report.status
                                );
                                message.extraction_status = "metadata_and_payload".to_string();
                                for payload in loaded_attachments.drain(..) {
                                    attachments.push(payload.record.clone());
                                    attachment_payloads.push(payload);
                                }
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
                        "property_context_loaded; property_count={}",
                        loaded.property_report.parsed_property_count
                    );
                    messages.push(message);
                }
                Err(reason) => {
                    messages.push(candidate_message(
                        run_id,
                        pst_id,
                        &root_folder.folder_key,
                        &root_folder.folder_path,
                        entry,
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

    let folders_discovered = 1;
    let messages_discovered = nbt.entries.len() as u64;
    let messages_extracted = messages.len() as u64;
    let status = format!(
        "{}; bbt_status={}; nbt_status={}; m4_status=recipient_threading_available; m5_status=body_attachment_outputs_available; m10_status=payload_wiring_available; m11_status=extraction_path_integration; m12_status=attachment_subnode_integration; m14_status=recursive_subnode_layout_exploration; body_payloads={}; attachment_payloads={}; subnode_references={}; subnode_decode_plans={}; subnode_decode_attempts={}; subnode_decoded_blocks={}; subnode_child_references={}; subnode_recursive_child_decodes={}; subnode_unsupported_layouts={}; attachment_table_parse_errors={}",
        metadata_status,
        bbt.status,
        nbt.status,
        body_payloads.len(),
        attachment_payloads.len(),
        subnode_report.subnode_reference_count,
        subnode_plans.len(),
        subnode_decode_attempts,
        subnode_decoded_blocks,
        subnode_child_references,
        subnode_recursive_child_decodes,
        subnode_unsupported_layouts,
        attachment_table_parse_errors
    );

    Ok(MetadataExtractionOutput {
        folders: vec![root_folder],
        folder_inventory: vec![root_inventory],
        messages,
        recipients,
        message_references,
        bodies,
        body_payloads,
        attachments,
        attachment_payloads,
        manifest,
        issues,
        folders_discovered,
        messages_discovered,
        messages_extracted,
        status,
    })
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
        internet_message_id: None,
        in_reply_to_id: None,
        conversation_index: None,
        conversation_topic: None,
        normalized_subject: None,
        has_text_body: false,
        has_html_body: false,
        has_attachments: false,
        attachment_count: 0,
        metadata_status: "node_property_context_unavailable".to_string(),
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
            status: "m14_recursive_subnode_layout_output_available".to_string(),
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
