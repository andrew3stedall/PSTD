from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


def replace_nth(text: str, old: str, new: str, occurrence: int, label: str) -> str:
    start = -1
    for _ in range(occurrence):
        start = text.find(old, start + 1)
        if start < 0:
            raise SystemExit(f"{label}: occurrence {occurrence} not found")
    return text[:start] + new + text[start + len(old):]


attachment_path = Path("src/pst/attachment_property_context.rs")
attachment = attachment_path.read_text(encoding="utf-8")

attachment = replace_once(
    attachment,
    "use crate::pst::attachments::unavailable_attachment_record_from_properties;\n",
    "use crate::pst::attachments::{\n"
    "    attachment_payload, unavailable_attachment_record_from_properties, AttachmentMetadata,\n"
    "    AttachmentPayload,\n"
    "};\n"
    "use crate::pst::bbt::BbtIndex;\n"
    "use crate::pst::data_tree::load_unicode_xblock_payload;\n"
    "use crate::pst::limits::ParserLimits;\n",
    "attachment imports",
)
attachment = replace_once(
    attachment,
    "use crate::pst::property_context::PropertyContext;\n",
    "use crate::pst::property_context::PropertyContext;\n"
    "use crate::pst::reader::PstByteReader;\n",
    "reader import",
)
attachment = replace_once(
    attachment,
    "    pub rejected_context_count: usize,\n    pub status: String,\n",
    "    pub rejected_context_count: usize,\n"
    "    pub payload_count: usize,\n"
    "    pub payload_bytes: u64,\n"
    "    pub payload_failure_count: usize,\n"
    "    pub status: String,\n",
    "report fields",
)
attachment = replace_once(
    attachment,
    "            rejected_context_count,\n            status: status.to_string(),\n",
    "            rejected_context_count,\n"
    "            payload_count: 0,\n"
    "            payload_bytes: 0,\n"
    "            payload_failure_count: 0,\n"
    "            status: status.to_string(),\n",
    "metadata report initializer",
)

payload_function = r'''
pub fn attachment_payloads_from_property_context_subnodes(
    message_key: &str,
    blocks: &[PayloadBlock],
    reader: &PstByteReader,
    bbt: &BbtIndex,
    limits: ParserLimits,
) -> (
    Vec<AttachmentPayload>,
    Vec<AttachmentRecord>,
    AttachmentPropertyContextReport,
) {
    let mut payloads = Vec::new();
    let mut records = Vec::new();
    let mut property_context_count = 0usize;
    let mut rejected_context_count = 0usize;
    let mut payload_failure_count = 0usize;

    for block in blocks {
        let Ok(heap) = HeapOnNode::parse(&block.bytes, block.block_ref.offset.0) else {
            continue;
        };
        if heap.header.client_signature != HEAP_CLIENT_PROPERTY_CONTEXT {
            continue;
        }
        property_context_count += 1;

        let Ok(bth) =
            BthMap::parse_property_context_from_heap(&heap, &block.bytes, block.block_ref.offset.0)
        else {
            rejected_context_count += 1;
            continue;
        };
        let Ok(report) = PropertyContext::from_bth_with_report(&bth) else {
            rejected_context_count += 1;
            continue;
        };
        let ordinal = payloads.len() + records.len();
        let Some(mut record) =
            filename_attachment_record(message_key, ordinal, &report.context, blocks)
        else {
            rejected_context_count += 1;
            continue;
        };

        let Some((data_nid, data_bid)) = resolved_subnode_data_reference(&report.context, blocks)
        else {
            payload_failure_count += 1;
            records.push(record);
            continue;
        };
        let Some(expected_size) = record.declared_size_bytes else {
            payload_failure_count += 1;
            records.push(record);
            continue;
        };

        match load_unicode_xblock_payload(
            reader,
            bbt,
            crate::pst::primitives::BlockId(data_bid),
            expected_size,
            limits,
        ) {
            Ok(tree) => {
                let metadata = AttachmentMetadata {
                    filename_original: record.filename_original.clone(),
                    content_type: record.content_type.clone(),
                    is_inline: record.is_inline,
                    content_id: record.content_id.clone(),
                    attachment_method: record.attachment_method,
                    declared_size_bytes: record.declared_size_bytes,
                };
                let mut payload = attachment_payload(message_key, ordinal, metadata, tree.bytes);
                payload.record.extraction_status = format!(
                    "attachment_payload_extracted_unicode_xblock; data_nid=0x{data_nid:08x}; data_bid=0x{data_bid:x}; child_blocks={}; zip_signature=504b0304",
                    tree.child_bids.len()
                );
                payloads.push(payload);
            }
            Err(reason) => {
                payload_failure_count += 1;
                record.extraction_status = format!(
                    "{}; data_tree_error={}",
                    record.extraction_status,
                    sanitized_status_reason(&reason.to_string())
                );
                records.push(record);
            }
        }
    }

    let filename_record_count = payloads.len() + records.len();
    let payload_count = payloads.len();
    let payload_bytes = payloads
        .iter()
        .map(|payload| payload.record.size_bytes)
        .sum::<u64>();
    let status = if payload_count > 0 && payload_failure_count == 0 {
        "attachment_property_context_payloads_extracted"
    } else if payload_count > 0 {
        "attachment_property_context_payloads_partially_extracted"
    } else if filename_record_count > 0 {
        "attachment_property_context_payloads_unavailable"
    } else {
        "attachment_property_context_filename_absent"
    };

    (
        payloads,
        records,
        AttachmentPropertyContextReport {
            property_context_count,
            filename_record_count,
            rejected_context_count,
            payload_count,
            payload_bytes,
            payload_failure_count,
            status: status.to_string(),
        },
    )
}

'''
attachment = replace_once(
    attachment,
    "fn filename_attachment_record(\n",
    payload_function + "fn filename_attachment_record(\n",
    "payload function insertion",
)

sanitize_function = r'''
fn sanitized_status_reason(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => ch,
            _ => '_',
        })
        .collect::<String>()
        .trim_matches('_')
        .chars()
        .take(120)
        .collect()
}

'''
attachment = replace_once(
    attachment,
    "#[cfg(test)]\nmod tests {\n",
    sanitize_function + "#[cfg(test)]\nmod tests {\n",
    "sanitizer insertion",
)
attachment_path.write_text(attachment, encoding="utf-8")

engine_path = Path("src/engine/metadata.rs")
engine = engine_path.read_text(encoding="utf-8")
engine = replace_once(
    engine,
    "use crate::pst::attachment_property_context::attachment_records_from_property_context_subnodes;\n",
    "use crate::pst::attachment_property_context::{\n"
    "    attachment_payloads_from_property_context_subnodes,\n"
    "    attachment_records_from_property_context_subnodes,\n"
    "};\n",
    "engine imports",
)
engine = replace_once(
    engine,
    "                    let message_property_has_attachments = message.has_attachments;\n",
    "                    let mut message_property_has_attachments = message.has_attachments;\n",
    "mutable attachment gate",
)
engine = replace_nth(
    engine,
    "                            let (mut property_context_attachments, attachment_property_report) =\n",
    "                            let (property_context_attachments, attachment_property_report) =\n",
    1,
    "first metadata tuple",
)
engine = replace_once(
    engine,
    "                                attachments.append(&mut property_context_attachments);\n",
    "                                message_property_has_attachments = true;\n",
    "defer metadata record emission",
)
second_call = (
    "                            let (mut property_context_attachments, attachment_property_report) =\n"
    "                                attachment_records_from_property_context_subnodes(\n"
    "                                    &message.message_key,\n"
    "                                    &loaded_subnodes.payloads,\n"
    "                                );\n"
)
second_replacement = (
    "                            let (\n"
    "                                mut property_context_payloads,\n"
    "                                mut property_context_attachments,\n"
    "                                attachment_property_report,\n"
    "                            ) = attachment_payloads_from_property_context_subnodes(\n"
    "                                &message.message_key,\n"
    "                                &loaded_subnodes.payloads,\n"
    "                                &reader,\n"
    "                                &bbt,\n"
    "                                limits,\n"
    "                            );\n"
)
engine = replace_once(engine, second_call, second_replacement, "second payload call")
engine = replace_once(
    engine,
    "                            let attachment_record_count = loaded_attachments.len()\n"
    "                                + unavailable_attachment_records.len()\n"
    "                                + property_context_attachments.len();\n",
    "                            let attachment_record_count = loaded_attachments.len()\n"
    "                                + unavailable_attachment_records.len()\n"
    "                                + property_context_payloads.len()\n"
    "                                + property_context_attachments.len();\n",
    "attachment count",
)
engine = replace_once(
    engine,
    "                            if loaded_attachments.is_empty()\n"
    "                                && property_context_attachments.is_empty()\n"
    "                            {\n",
    "                            if loaded_attachments.is_empty()\n"
    "                                && property_context_payloads.is_empty()\n"
    "                                && property_context_attachments.is_empty()\n"
    "                            {\n",
    "empty payload condition",
)
engine = replace_once(
    engine,
    "                                message.extraction_status = if loaded_attachments.is_empty() {\n"
    "                                    \"metadata_body_and_attachment_metadata\".to_string()\n"
    "                                } else {\n"
    "                                    \"metadata_and_payload\".to_string()\n"
    "                                };\n"
    "                                attachments.append(&mut property_context_attachments);\n",
    "                                message.extraction_status = if loaded_attachments.is_empty()\n"
    "                                    && property_context_payloads.is_empty()\n"
    "                                {\n"
    "                                    \"metadata_body_and_attachment_metadata\".to_string()\n"
    "                                } else {\n"
    "                                    \"metadata_and_payload\".to_string()\n"
    "                                };\n"
    "                                for payload in property_context_payloads.drain(..) {\n"
    "                                    attachments.push(payload.record.clone());\n"
    "                                    attachment_payloads.push(payload);\n"
    "                                }\n"
    "                                attachments.append(&mut property_context_attachments);\n",
    "payload emission",
)
engine_path.write_text(engine, encoding="utf-8")
