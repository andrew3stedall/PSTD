use crate::pst::heap::HeapOnNode;
use crate::pst::payload::PayloadBlock;
use crate::pst::tc_recipient_identity_reference::{
    extract_recipient_identity_references, RecipientIdentityReferenceEvidence,
};
use crate::pst::tc_recipient_identity_string::{
    resolve_recipient_identity_heap_strings, RecipientIdentityStringEvidence,
};
use crate::pst::tc_row_payload_candidates::resolve_row_payload_candidates;
use crate::pst::tc_row_transport_metadata::resolve_row_transport_metadata;
use crate::pst::tc_subnode_rows::TcSubnodeRowResolutionReport;
use crate::pst::tcinfo::TcColumnDescriptor;

pub const RECIPIENT_IDENTITY_VALIDATED: &str = "tc_recipient_identity_validated";
pub const RECIPIENT_IDENTITY_UNAVAILABLE: &str = "tc_recipient_identity_unavailable";
pub const RECIPIENT_IDENTITY_FAILED: &str = "tc_recipient_identity_failed";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecipientIdentityProjectionReport {
    pub candidate_status: String,
    pub transport_status: String,
    pub identity_status: String,
    pub references: Option<RecipientIdentityReferenceEvidence>,
    pub strings: Option<RecipientIdentityStringEvidence>,
    pub failure_reason: Option<String>,
}

pub fn project_recipient_identity_strings(
    payloads: &[PayloadBlock],
    rows_nid: u32,
    row_resolution: &TcSubnodeRowResolutionReport,
    columns: &[TcColumnDescriptor],
    bitmap_masks: &[String],
    table_heap_bytes: &[u8],
    table_heap_base_offset: u64,
    fixed_data_end: usize,
) -> RecipientIdentityProjectionReport {
    let candidates = resolve_row_payload_candidates(payloads, rows_nid);
    let transport = resolve_row_transport_metadata(payloads, rows_nid, row_resolution);

    let unavailable = |reason: Option<String>| RecipientIdentityProjectionReport {
        candidate_status: candidates.status.clone(),
        transport_status: transport.transport_status.clone(),
        identity_status: if reason.is_some() {
            RECIPIENT_IDENTITY_FAILED.to_string()
        } else {
            RECIPIENT_IDENTITY_UNAVAILABLE.to_string()
        },
        references: None,
        strings: None,
        failure_reason: reason,
    };

    if candidates.payloads.len() != 1 {
        return unavailable(None);
    }
    let Some(row_width) = transport.row_width else {
        return unavailable(transport.failure_reason);
    };
    if transport.absolute_row_offsets.is_empty() {
        return unavailable(None);
    }

    let references = match extract_recipient_identity_references(
        columns,
        bitmap_masks,
        &candidates.payloads[0].bytes,
        &transport.absolute_row_offsets,
        row_width,
        fixed_data_end,
    ) {
        Ok(evidence) => evidence,
        Err(reason) => return unavailable(Some(reason)),
    };

    let heap = match HeapOnNode::parse(table_heap_bytes, table_heap_base_offset) {
        Ok(heap) => heap,
        Err(error) => return unavailable(Some(error.to_string())),
    };
    let strings = match resolve_recipient_identity_heap_strings(
        &references,
        &heap,
        table_heap_bytes,
        table_heap_base_offset,
    ) {
        Ok(evidence) => evidence,
        Err(reason) => return unavailable(Some(reason)),
    };

    RecipientIdentityProjectionReport {
        candidate_status: candidates.status,
        transport_status: transport.transport_status,
        identity_status: RECIPIENT_IDENTITY_VALIDATED.to_string(),
        references: Some(references),
        strings: Some(strings),
        failure_reason: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        project_recipient_identity_strings, RECIPIENT_IDENTITY_FAILED,
        RECIPIENT_IDENTITY_VALIDATED,
    };
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset};
    use crate::pst::tc_subnode_rows::resolve_subnode_row_storage;
    use crate::pst::tcinfo::TcColumnDescriptor;

    #[test]
    fn resolves_recipient_display_names_from_rows_to_heap_strings() {
        let table_heap = sample_heap(&[
            "Alice\0"
                .encode_utf16()
                .flat_map(u16::to_le_bytes)
                .collect(),
            "Bob\0".encode_utf16().flat_map(u16::to_le_bytes).collect(),
        ]);
        let mut rows = Vec::new();
        for reference in [0x20u32, 0x40u32] {
            rows.extend_from_slice(&reference.to_le_bytes());
            rows.push(0x80);
        }
        let payloads = vec![slblock(0x82, 0x74, 0x7a), payload(0x7a, rows)];
        let resolution = resolve_subnode_row_storage(&payloads, 0x74, &[0, 1], 1, 4, 5);
        let columns = vec![TcColumnDescriptor {
            property_tag: 0x3001_001f,
            data_offset: 0,
            data_size: 4,
            bitmap_bit: 0,
        }];

        let report = project_recipient_identity_strings(
            &payloads,
            0x74,
            &resolution,
            &columns,
            &resolution.bitmap_masks,
            &table_heap,
            0,
            4,
        );

        assert_eq!(report.identity_status, RECIPIENT_IDENTITY_VALIDATED);
        assert_eq!(
            report.strings.expect("strings should resolve").row_values,
            vec!["Alice", "Bob"]
        );
    }

    #[test]
    fn fails_closed_when_a_recipient_reference_is_node_resident() {
        let table_heap = sample_heap(&[b"Alice\0".to_vec()]);
        let mut rows = Vec::new();
        rows.extend_from_slice(&0x24u32.to_le_bytes());
        rows.push(0x80);
        let payloads = vec![slblock(0x82, 0x74, 0x7a), payload(0x7a, rows)];
        let resolution = resolve_subnode_row_storage(&payloads, 0x74, &[0], 1, 4, 5);
        let columns = vec![TcColumnDescriptor {
            property_tag: 0x3001_001e,
            data_offset: 0,
            data_size: 4,
            bitmap_bit: 0,
        }];

        let report = project_recipient_identity_strings(
            &payloads,
            0x74,
            &resolution,
            &columns,
            &resolution.bitmap_masks,
            &table_heap,
            0,
            4,
        );

        assert_eq!(report.identity_status, RECIPIENT_IDENTITY_FAILED);
        assert!(report.references.is_none());
        assert!(report.strings.is_none());
        assert!(report
            .failure_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("not heap-resident")));
    }

    fn sample_heap(values: &[Vec<u8>]) -> Vec<u8> {
        let data_start = 16usize;
        let data_len: usize = values.iter().map(Vec::len).sum();
        let page_map_offset = data_start + data_len;
        let mut buf = vec![0u8; page_map_offset + 4 + (values.len() + 1) * 2];
        buf[0..2].copy_from_slice(&(page_map_offset as u16).to_le_bytes());
        buf[2] = 0xec;
        buf[3] = 0xbc;
        buf[4..8].copy_from_slice(&0x20u32.to_le_bytes());

        let mut cursor = data_start;
        for value in values {
            buf[cursor..cursor + value.len()].copy_from_slice(value);
            cursor += value.len();
        }

        buf[page_map_offset..page_map_offset + 2]
            .copy_from_slice(&(values.len() as u16).to_le_bytes());
        let offsets_start = page_map_offset + 4;
        let mut offset = data_start as u16;
        buf[offsets_start..offsets_start + 2].copy_from_slice(&offset.to_le_bytes());
        for (index, value) in values.iter().enumerate() {
            offset += value.len() as u16;
            let start = offsets_start + (index + 1) * 2;
            buf[start..start + 2].copy_from_slice(&offset.to_le_bytes());
        }
        buf
    }

    fn slblock(block_id: u64, nid: u64, data_bid: u64) -> PayloadBlock {
        let mut bytes = vec![0u8; 32];
        bytes[0] = 0x02;
        bytes[2..4].copy_from_slice(&1u16.to_le_bytes());
        bytes[8..16].copy_from_slice(&nid.to_le_bytes());
        bytes[16..24].copy_from_slice(&data_bid.to_le_bytes());
        payload(block_id, bytes)
    }

    fn payload(block_id: u64, bytes: Vec<u8>) -> PayloadBlock {
        PayloadBlock {
            block_id: BlockId(block_id),
            block_ref: BlockRef {
                block_id: BlockId(block_id),
                offset: ByteOffset(0),
                size: bytes.len() as u64,
            },
            bytes,
            status: "test".to_string(),
        }
    }
}
