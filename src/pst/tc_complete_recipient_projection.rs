use crate::pst::payload::PayloadBlock;
use crate::pst::tc_fixed_width_diagnostic::TcFixedWidthDiagnostic;
use crate::pst::tc_property_classification::{
    PID_TAG_DISPLAY_NAME, PID_TAG_EMAIL_ADDRESS, PID_TAG_SMTP_ADDRESS,
};
use crate::pst::tc_recipient_identity_diagnostic::{
    build_recipient_identity_diagnostic, TcRecipientIdentityDiagnostic,
};
use crate::pst::tc_recipient_identity_projection::project_recipient_identity_strings_from_rows;
use crate::pst::tc_recipient_records::{
    assemble_complete_recipient_records, TcCompleteRecipientRecordReport,
};
use crate::pst::tc_row_payload_candidates::resolve_row_payload_candidates;
use crate::pst::tc_subnode_rows::TcSubnodeRowResolutionReport;
use crate::pst::tcinfo::TcColumnDescriptor;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TcCompleteRecipientProjectionReport {
    pub display_names: TcRecipientIdentityDiagnostic,
    pub addresses: TcRecipientIdentityDiagnostic,
    pub complete_records: TcCompleteRecipientRecordReport,
}

impl TcCompleteRecipientProjectionReport {
    pub fn status_fragment(&self) -> String {
        format!(
            "{},display_names=[{}],addresses=[{}]",
            self.complete_records.status_fragment(),
            self.display_names.status_fragment(),
            self.addresses.status_fragment(),
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub fn project_complete_recipient_records(
    payloads: &[PayloadBlock],
    rows_nid: u32,
    row_resolution: &TcSubnodeRowResolutionReport,
    columns: &[TcColumnDescriptor],
    bitmap_masks: &[String],
    table_heap_bytes: &[u8],
    table_heap_base_offset: u64,
    fixed_data_end: usize,
    recipient_types: &TcFixedWidthDiagnostic,
) -> TcCompleteRecipientProjectionReport {
    let candidates = resolve_row_payload_candidates(payloads, rows_nid);
    let candidate_bytes = candidates
        .payloads
        .iter()
        .map(|payload| payload.bytes.as_slice())
        .collect::<Vec<_>>();
    project_complete_recipient_records_from_rows(
        &candidate_bytes,
        candidates.status,
        row_resolution,
        columns,
        bitmap_masks,
        table_heap_bytes,
        table_heap_base_offset,
        fixed_data_end,
        recipient_types,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn project_complete_recipient_records_from_rows(
    candidate_bytes: &[&[u8]],
    candidate_status: String,
    row_resolution: &TcSubnodeRowResolutionReport,
    columns: &[TcColumnDescriptor],
    bitmap_masks: &[String],
    table_heap_bytes: &[u8],
    table_heap_base_offset: u64,
    fixed_data_end: usize,
    recipient_types: &TcFixedWidthDiagnostic,
) -> TcCompleteRecipientProjectionReport {
    let display_columns = columns
        .iter()
        .filter(|column| (column.property_tag >> 16) as u16 == PID_TAG_DISPLAY_NAME)
        .cloned()
        .collect::<Vec<_>>();
    let address_columns = columns
        .iter()
        .filter(|column| {
            matches!(
                (column.property_tag >> 16) as u16,
                PID_TAG_SMTP_ADDRESS | PID_TAG_EMAIL_ADDRESS
            )
        })
        .cloned()
        .collect::<Vec<_>>();

    let display_names =
        build_recipient_identity_diagnostic(project_recipient_identity_strings_from_rows(
            candidate_bytes,
            candidate_status.clone(),
            row_resolution,
            &display_columns,
            bitmap_masks,
            table_heap_bytes,
            table_heap_base_offset,
            fixed_data_end,
        ));
    let addresses =
        build_recipient_identity_diagnostic(project_recipient_identity_strings_from_rows(
            candidate_bytes,
            candidate_status,
            row_resolution,
            &address_columns,
            bitmap_masks,
            table_heap_bytes,
            table_heap_base_offset,
            fixed_data_end,
        ));
    let complete_records =
        assemble_complete_recipient_records(recipient_types, &display_names, &addresses);

    TcCompleteRecipientProjectionReport {
        display_names,
        addresses,
        complete_records,
    }
}

#[cfg(test)]
mod tests {
    use super::project_complete_recipient_records;
    use crate::pst::payload::PayloadBlock;
    use crate::pst::primitives::{BlockId, BlockRef, ByteOffset};
    use crate::pst::tc_fixed_width_diagnostic::TcFixedWidthDiagnostic;
    use crate::pst::tc_recipient_records::RECIPIENT_RECORDS_VALIDATED;
    use crate::pst::tc_subnode_rows::resolve_subnode_row_storage;
    use crate::pst::tcinfo::TcColumnDescriptor;

    #[test]
    fn projects_names_and_addresses_from_the_same_rows_and_heap() {
        let values = [
            "Recipient 1\0",
            "Recipient 2\0",
            "to1@domain.com\0",
            "to2@domain.com\0",
        ]
        .map(|value| {
            value
                .encode_utf16()
                .flat_map(u16::to_le_bytes)
                .collect::<Vec<_>>()
        });
        let table_heap = sample_heap(&values);
        let mut rows = Vec::new();
        for (display_hid, address_hid) in [(0x20u32, 0x60u32), (0x40u32, 0x80u32)] {
            rows.extend_from_slice(&display_hid.to_le_bytes());
            rows.extend_from_slice(&address_hid.to_le_bytes());
            rows.push(0x03);
        }
        let payloads = vec![slblock(0x82, 0x74, 0x7a), payload(0x7a, rows)];
        let resolution = resolve_subnode_row_storage(&payloads, 0x74, &[0, 1], 2, 8, 9);
        let columns = vec![descriptor(0, 0, 0x3001_001f), descriptor(1, 4, 0x3003_001f)];

        let report = project_complete_recipient_records(
            &payloads,
            0x74,
            &resolution,
            &columns,
            &resolution.bitmap_masks,
            &table_heap,
            0,
            8,
            &recipient_types(&["to", "cc"]),
        );

        assert_eq!(report.complete_records.status, RECIPIENT_RECORDS_VALIDATED);
        assert_eq!(report.complete_records.records.len(), 2);
        assert_eq!(
            report.complete_records.records[0].display_name,
            "Recipient 1"
        );
        assert_eq!(report.complete_records.records[0].address, "to1@domain.com");
        assert_eq!(report.complete_records.records[1].role, "cc");
        assert!(report
            .status_fragment()
            .contains("0:to:Recipient 1:to1@domain.com:native_email_address"));
    }

    #[test]
    fn projects_the_four_public_fixture_recipient_rows_in_order() {
        let values = [
            "Recipient 1\0",
            "Recipient 2\0",
            "Recipient 3\0",
            "Recipient 4\0",
            "to1@domain.com\0",
            "to2@domain.com\0",
            "cc1@domain.com\0",
            "cc2@domain.com\0",
        ]
        .map(|value| {
            value
                .encode_utf16()
                .flat_map(u16::to_le_bytes)
                .collect::<Vec<_>>()
        });
        let table_heap = sample_heap(&values);
        let mut rows = Vec::new();
        for (display_hid, address_hid) in [
            (0x20u32, 0xa0u32),
            (0x40u32, 0xc0u32),
            (0x60u32, 0xe0u32),
            (0x80u32, 0x100u32),
        ] {
            rows.extend_from_slice(&display_hid.to_le_bytes());
            rows.extend_from_slice(&address_hid.to_le_bytes());
            rows.push(0x03);
        }
        let payloads = vec![slblock(0x82, 0x74, 0x7a), payload(0x7a, rows)];
        let resolution = resolve_subnode_row_storage(&payloads, 0x74, &[0, 1, 2, 3], 2, 8, 9);
        let columns = vec![descriptor(0, 0, 0x3001_001f), descriptor(1, 4, 0x3003_001f)];

        let report = project_complete_recipient_records(
            &payloads,
            0x74,
            &resolution,
            &columns,
            &resolution.bitmap_masks,
            &table_heap,
            0,
            8,
            &recipient_types(&["to", "to", "cc", "cc"]),
        );

        assert_eq!(report.complete_records.status, RECIPIENT_RECORDS_VALIDATED);
        assert_eq!(report.complete_records.records.len(), 4);
        assert_eq!(
            report.complete_records.records[0].display_name,
            "Recipient 1"
        );
        assert_eq!(report.complete_records.records[0].address, "to1@domain.com");
        assert_eq!(report.complete_records.records[1].role, "to");
        assert_eq!(
            report.complete_records.records[2].display_name,
            "Recipient 3"
        );
        assert_eq!(report.complete_records.records[2].address, "cc1@domain.com");
        assert_eq!(report.complete_records.records[3].role, "cc");
        assert_eq!(report.complete_records.records[3].address, "cc2@domain.com");
        assert!(report
            .status_fragment()
            .contains("3:cc:Recipient 4:cc2@domain.com:native_email_address"));
    }

    fn recipient_types(values: &[&str]) -> TcFixedWidthDiagnostic {
        TcFixedWidthDiagnostic {
            candidate_status: "candidate".to_string(),
            transport_status: "transport".to_string(),
            evidence_status: "validated".to_string(),
            property_tag: Some(0x0c15_0003),
            property_name: Some("PidTagRecipientType".to_string()),
            data_offset: Some(0),
            data_size: Some(4),
            row_values_hex: Vec::new(),
            decoded_values: Vec::new(),
            semantic_values: values.iter().map(|value| (*value).to_string()).collect(),
            failure_reason: None,
        }
    }

    fn descriptor(bitmap_bit: u8, data_offset: u16, property_tag: u32) -> TcColumnDescriptor {
        TcColumnDescriptor {
            property_tag,
            data_offset,
            data_size: 4,
            bitmap_bit,
        }
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
