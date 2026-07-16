use crate::output::metadata::AttachmentRecord;
use crate::pst::attachments::unavailable_attachment_record_from_properties;
use crate::pst::bth::BthMap;
use crate::pst::heap::HeapOnNode;
use crate::pst::mapi::{
    MapiValue, PR_ATTACH_FILENAME, PR_ATTACH_FILENAME_A, PR_ATTACH_LONG_FILENAME,
    PR_ATTACH_LONG_FILENAME_A, PR_ATTACH_METHOD, PR_ATTACH_SIZE,
};
use crate::pst::payload::PayloadBlock;
use crate::pst::property_context::PropertyContext;

const HEAP_CLIENT_PROPERTY_CONTEXT: u8 = 0xbc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentPropertyContextReport {
    pub property_context_count: usize,
    pub filename_record_count: usize,
    pub rejected_context_count: usize,
    pub status: String,
}

pub fn attachment_records_from_property_context_subnodes(
    message_key: &str,
    blocks: &[PayloadBlock],
) -> (Vec<AttachmentRecord>, AttachmentPropertyContextReport) {
    let mut records = Vec::new();
    let mut property_context_count = 0usize;
    let mut rejected_context_count = 0usize;

    for block in blocks {
        let Ok(heap) = HeapOnNode::parse(&block.bytes, block.block_ref.offset.0) else {
            continue;
        };
        if heap.header.client_signature != HEAP_CLIENT_PROPERTY_CONTEXT {
            continue;
        }
        property_context_count += 1;

        let Ok(bth) = BthMap::parse_property_context_from_heap(
            &heap,
            &block.bytes,
            block.block_ref.offset.0,
        ) else {
            rejected_context_count += 1;
            continue;
        };
        let Ok(report) = PropertyContext::from_bth_with_report(&bth) else {
            rejected_context_count += 1;
            continue;
        };
        let Some(record) = filename_attachment_record(
            message_key,
            records.len(),
            &report.context,
        ) else {
            rejected_context_count += 1;
            continue;
        };
        records.push(record);
    }

    let status = if records.is_empty() {
        "attachment_property_context_filename_absent"
    } else if rejected_context_count == 0 {
        "attachment_property_context_filenames_extracted"
    } else {
        "attachment_property_context_filenames_partially_extracted"
    };
    (
        records,
        AttachmentPropertyContextReport {
            property_context_count,
            filename_record_count: records.len(),
            rejected_context_count,
            status: status.to_string(),
        },
    )
}

fn filename_attachment_record(
    message_key: &str,
    ordinal: usize,
    properties: &PropertyContext,
) -> Option<AttachmentRecord> {
    let filename = first_non_empty_string(
        properties,
        &[
            PR_ATTACH_LONG_FILENAME,
            PR_ATTACH_LONG_FILENAME_A,
            PR_ATTACH_FILENAME,
            PR_ATTACH_FILENAME_A,
        ],
    )?;
    if properties.value(PR_ATTACH_METHOD).is_none() || properties.value(PR_ATTACH_SIZE).is_none() {
        return None;
    }

    let mut record = unavailable_attachment_record_from_properties(
        message_key,
        ordinal,
        properties,
        "attachment_metadata_extracted_payload_reference_unresolved",
    );
    record.filename_original = Some(filename.clone());
    record.filename_safe = crate::pst::attachments::safe_filename(Some(&filename), ordinal);
    record.extension = crate::pst::attachments::file_extension(&record.filename_safe);
    record.archive_path = format!(
        "attachments/{message_key}/{}_{}",
        record.attachment_key, record.filename_safe
    );
    Some(record)
}

fn first_non_empty_string(properties: &PropertyContext, tags: &[u32]) -> Option<String> {
    tags.iter().find_map(|tag| {
        let value = properties.value(*tag)?;
        match value.decoded.as_ref() {
            Some(MapiValue::String(value)) if !value.trim().is_empty() => {
                Some(value.trim().to_string())
            }
            _ => None,
        }
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::filename_attachment_record;
    use crate::pst::mapi::{
        MapiValue, PR_ATTACH_LONG_FILENAME, PR_ATTACH_METHOD, PR_ATTACH_SIZE,
    };
    use crate::pst::property_context::{PropertyContext, PropertyValue};

    fn property(tag: u32, name: &str, decoded: MapiValue) -> PropertyValue {
        PropertyValue {
            tag,
            name: name.to_string(),
            raw: Vec::new(),
            decoded: Some(decoded),
            status: "selected".to_string(),
        }
    }

    #[test]
    fn exposes_validated_attachment_filename_metadata() {
        let mut values = HashMap::new();
        values.insert(
            PR_ATTACH_LONG_FILENAME,
            property(
                PR_ATTACH_LONG_FILENAME,
                "attachment_long_filename",
                MapiValue::String("attachment.docx".to_string()),
            ),
        );
        values.insert(
            PR_ATTACH_METHOD,
            property(
                PR_ATTACH_METHOD,
                "attachment_method",
                MapiValue::Integer32(1),
            ),
        );
        values.insert(
            PR_ATTACH_SIZE,
            property(
                PR_ATTACH_SIZE,
                "attachment_size",
                MapiValue::Integer32(15_503),
            ),
        );
        let record = filename_attachment_record(
            "msg_c6163b9157944cc9",
            0,
            &PropertyContext { values },
        )
        .expect("validated filename-bearing attachment context");

        assert_eq!(record.filename_original.as_deref(), Some("attachment.docx"));
        assert_eq!(record.filename_safe, "attachment.docx");
        assert_eq!(record.extension.as_deref(), Some("docx"));
        assert_eq!(record.declared_size_bytes, Some(15_503));
        assert_eq!(record.attachment_method, Some(1));
        assert_eq!(record.size_bytes, 0);
        assert_eq!(
            record.extraction_status,
            "attachment_metadata_extracted_payload_reference_unresolved"
        );
    }

    #[test]
    fn rejects_blank_or_incomplete_attachment_contexts() {
        let mut blank = HashMap::new();
        blank.insert(
            PR_ATTACH_LONG_FILENAME,
            property(
                PR_ATTACH_LONG_FILENAME,
                "attachment_long_filename",
                MapiValue::String("  ".to_string()),
            ),
        );
        blank.insert(
            PR_ATTACH_METHOD,
            property(
                PR_ATTACH_METHOD,
                "attachment_method",
                MapiValue::Integer32(1),
            ),
        );
        blank.insert(
            PR_ATTACH_SIZE,
            property(
                PR_ATTACH_SIZE,
                "attachment_size",
                MapiValue::Integer32(1),
            ),
        );
        assert!(filename_attachment_record("msg", 0, &PropertyContext { values: blank }).is_none());

        let mut incomplete = HashMap::new();
        incomplete.insert(
            PR_ATTACH_LONG_FILENAME,
            property(
                PR_ATTACH_LONG_FILENAME,
                "attachment_long_filename",
                MapiValue::String("attachment.docx".to_string()),
            ),
        );
        assert!(
            filename_attachment_record("msg", 0, &PropertyContext { values: incomplete }).is_none()
        );
    }
}
