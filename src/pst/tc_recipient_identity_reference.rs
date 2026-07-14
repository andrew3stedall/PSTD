use crate::pst::tc_property_classification::{
    classify_tc_property, PID_TAG_DISPLAY_NAME, PID_TAG_EMAIL_ADDRESS, PID_TAG_SMTP_ADDRESS,
};
use crate::pst::tcinfo::{classify_hnid, HnidKind, TcColumnDescriptor};

const PT_STRING8: u16 = 0x001e;
const PT_UNICODE: u16 = 0x001f;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecipientIdentityReferenceEvidence {
    pub property_tag: u32,
    pub property_name: String,
    pub data_offset: u16,
    pub row_references: Vec<u32>,
    pub reference_kinds: Vec<HnidKind>,
}

pub fn extract_recipient_identity_references(
    columns: &[TcColumnDescriptor],
    bitmap_masks: &[String],
    row_data: &[u8],
    row_offsets: &[usize],
    row_width: usize,
    fixed_data_end: usize,
) -> Result<RecipientIdentityReferenceEvidence, String> {
    if bitmap_masks.len() != row_offsets.len() || row_offsets.is_empty() {
        return Err("recipient identity row evidence is unavailable".to_string());
    }
    if row_width == 0 || fixed_data_end > row_width {
        return Err("recipient identity row boundary is invalid".to_string());
    }

    let column = columns
        .iter()
        .filter(|column| is_recipient_identity_string(column.property_tag))
        .filter(|column| column.data_size == 4)
        .filter(|column| {
            let index = column.bitmap_bit as usize;
            bitmap_masks
                .iter()
                .all(|mask| mask.as_bytes().get(index).copied() == Some(b'1'))
        })
        .min_by_key(|column| identity_priority(column.property_tag))
        .ok_or_else(|| {
            "no bounded recipient identity string reference is set in every row".to_string()
        })?;

    let end = column.data_offset as usize + column.data_size as usize;
    if end > fixed_data_end {
        return Err("recipient identity reference exceeds fixed row data".to_string());
    }

    let mut row_references = Vec::with_capacity(row_offsets.len());
    let mut reference_kinds = Vec::with_capacity(row_offsets.len());
    for row_offset in row_offsets {
        let row_end = row_offset
            .checked_add(row_width)
            .ok_or_else(|| "recipient row end overflow".to_string())?;
        let start = row_offset
            .checked_add(column.data_offset as usize)
            .ok_or_else(|| "recipient identity reference offset overflow".to_string())?;
        let value_end = start
            .checked_add(4)
            .ok_or_else(|| "recipient identity reference end overflow".to_string())?;
        if row_end > row_data.len() || value_end > row_end {
            return Err("recipient identity reference is outside the validated row".to_string());
        }
        let value = u32::from_le_bytes([
            row_data[start],
            row_data[start + 1],
            row_data[start + 2],
            row_data[start + 3],
        ]);
        row_references.push(value);
        reference_kinds.push(classify_hnid(value));
    }

    let classification = classify_tc_property(column.property_tag);
    let property_name = classification
        .canonical_name
        .ok_or_else(|| "recipient identity property classification is unavailable".to_string())?
        .to_string();

    Ok(RecipientIdentityReferenceEvidence {
        property_tag: column.property_tag,
        property_name,
        data_offset: column.data_offset,
        row_references,
        reference_kinds,
    })
}

fn is_recipient_identity_string(property_tag: u32) -> bool {
    let property_id = (property_tag >> 16) as u16;
    let property_type = property_tag as u16;
    matches!(
        property_id,
        PID_TAG_DISPLAY_NAME | PID_TAG_EMAIL_ADDRESS | PID_TAG_SMTP_ADDRESS
    ) && matches!(property_type, PT_STRING8 | PT_UNICODE)
}

fn identity_priority(property_tag: u32) -> u8 {
    match (property_tag >> 16) as u16 {
        PID_TAG_SMTP_ADDRESS => 0,
        PID_TAG_EMAIL_ADDRESS => 1,
        PID_TAG_DISPLAY_NAME => 2,
        _ => u8::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::extract_recipient_identity_references;
    use crate::pst::tcinfo::{HnidKind, TcColumnDescriptor};

    #[test]
    fn extracts_display_name_hnid_references_from_validated_rows() {
        let columns = vec![descriptor(0, 0, 0x3001_001f)];
        let rows = [0x60u32, 0x74u32]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect::<Vec<_>>();

        let evidence = extract_recipient_identity_references(
            &columns,
            &["1".to_string(), "1".to_string()],
            &rows,
            &[0, 4],
            4,
            4,
        )
        .expect("bounded references should validate");

        assert_eq!(evidence.property_name, "PidTagDisplayName");
        assert_eq!(evidence.row_references, vec![0x60, 0x74]);
        assert_eq!(
            evidence.reference_kinds,
            vec![HnidKind::HeapId, HnidKind::NodeId]
        );
    }

    #[test]
    fn prefers_smtp_then_native_email_then_display_name() {
        let columns = vec![
            descriptor(0, 0, 0x3001_001f),
            descriptor(1, 4, 0x3003_001f),
            descriptor(2, 8, 0x39fe_001f),
        ];
        let mut row = vec![0u8; 12];
        row[8..12].copy_from_slice(&0x60u32.to_le_bytes());

        let evidence = extract_recipient_identity_references(
            &columns,
            &["111".to_string()],
            &row,
            &[0],
            12,
            12,
        )
        .expect("preferred identity should validate");

        assert_eq!(evidence.property_name, "PidTagSmtpAddress");
        assert_eq!(evidence.row_references, vec![0x60]);
    }

    #[test]
    fn prefers_native_email_address_when_smtp_is_absent() {
        let columns = vec![descriptor(0, 0, 0x3001_001f), descriptor(1, 4, 0x3003_001f)];
        let mut row = vec![0u8; 8];
        row[4..8].copy_from_slice(&0x40u32.to_le_bytes());

        let evidence =
            extract_recipient_identity_references(&columns, &["11".to_string()], &row, &[0], 8, 8)
                .expect("native email address should validate");

        assert_eq!(evidence.property_name, "PidTagEmailAddress");
        assert_eq!(evidence.row_references, vec![0x40]);
    }

    #[test]
    fn falls_back_to_display_name_when_no_address_property_is_available() {
        let columns = vec![descriptor(0, 0, 0x3001_001f)];
        let mut row = vec![0u8; 4];
        row.copy_from_slice(&0x20u32.to_le_bytes());

        let evidence =
            extract_recipient_identity_references(&columns, &["1".to_string()], &row, &[0], 4, 4)
                .expect("display name fallback should validate");

        assert_eq!(evidence.property_name, "PidTagDisplayName");
        assert_eq!(evidence.row_references, vec![0x20]);
    }

    #[test]
    fn fails_closed_when_identity_is_not_present_on_every_row() {
        let error = extract_recipient_identity_references(
            &[descriptor(0, 0, 0x3001_001f)],
            &["1".to_string(), "0".to_string()],
            &[0; 8],
            &[0, 4],
            4,
            4,
        )
        .expect_err("partial identity evidence must not be published");

        assert_eq!(
            error,
            "no bounded recipient identity string reference is set in every row"
        );
    }

    #[test]
    fn rejects_non_string_and_non_hnid_sized_descriptors() {
        let error = extract_recipient_identity_references(
            &[TcColumnDescriptor {
                property_tag: 0x3001_0003,
                data_offset: 0,
                data_size: 4,
                bitmap_bit: 0,
            }],
            &["1".to_string()],
            &[0; 4],
            &[0],
            4,
            4,
        )
        .expect_err("non-string descriptors must not be interpreted as HNIDs");

        assert_eq!(
            error,
            "no bounded recipient identity string reference is set in every row"
        );
    }

    fn descriptor(bitmap_bit: u8, data_offset: u16, property_tag: u32) -> TcColumnDescriptor {
        TcColumnDescriptor {
            property_tag,
            data_offset,
            data_size: 4,
            bitmap_bit,
        }
    }
}
