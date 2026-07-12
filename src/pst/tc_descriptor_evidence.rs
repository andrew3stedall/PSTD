use crate::error::{PstdError, PstdResult};
use crate::pst::tcinfo::TcInfo;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TcDescriptorBitmapEvidence {
    pub bitmap_bit: usize,
    pub descriptor_order: usize,
    pub property_tag: u32,
    pub property_type: u16,
    pub data_offset: u16,
    pub data_size: u8,
    pub row_states: String,
}

pub fn build_descriptor_bitmap_evidence(
    tcinfo: &TcInfo,
    bitmap_masks: &[String],
) -> PstdResult<Vec<TcDescriptorBitmapEvidence>> {
    let column_count = tcinfo.columns.len();
    for (row_index, mask) in bitmap_masks.iter().enumerate() {
        if mask.len() != column_count {
            return Err(PstdError::pst_parse(
                None,
                format!(
                    "TCINFO row bitmap mask {row_index} has {} bits, expected {column_count}",
                    mask.len()
                ),
            ));
        }
        if !mask.bytes().all(|bit| matches!(bit, b'0' | b'1')) {
            return Err(PstdError::pst_parse(
                None,
                format!("TCINFO row bitmap mask {row_index} contains a non-binary state"),
            ));
        }
    }

    let mut evidence = tcinfo
        .columns
        .iter()
        .enumerate()
        .map(|(descriptor_order, descriptor)| {
            let bitmap_bit = descriptor.bitmap_bit as usize;
            let row_states = bitmap_masks
                .iter()
                .map(|mask| mask.as_bytes()[bitmap_bit] as char)
                .collect::<String>();
            TcDescriptorBitmapEvidence {
                bitmap_bit,
                descriptor_order,
                property_tag: descriptor.property_tag,
                property_type: (descriptor.property_tag >> 16) as u16,
                data_offset: descriptor.data_offset,
                data_size: descriptor.data_size,
                row_states,
            }
        })
        .collect::<Vec<_>>();
    evidence.sort_by_key(|item| item.bitmap_bit);
    Ok(evidence)
}

#[cfg(test)]
mod tests {
    use super::build_descriptor_bitmap_evidence;
    use crate::pst::tcinfo::TcInfo;

    #[test]
    fn maps_descriptor_metadata_to_raw_row_states_in_bitmap_order() {
        let info = TcInfo::parse(&sample_tcinfo([1, 0]), 0).unwrap();
        let masks = vec!["10".to_string(), "01".to_string()];

        let evidence = build_descriptor_bitmap_evidence(&info, &masks).unwrap();

        assert_eq!(evidence.len(), 2);
        assert_eq!(evidence[0].bitmap_bit, 0);
        assert_eq!(evidence[0].descriptor_order, 1);
        assert_eq!(evidence[0].property_tag, 0x001f3001);
        assert_eq!(evidence[0].property_type, 0x001f);
        assert_eq!(evidence[0].data_offset, 4);
        assert_eq!(evidence[0].data_size, 4);
        assert_eq!(evidence[0].row_states, "10");
        assert_eq!(evidence[1].bitmap_bit, 1);
        assert_eq!(evidence[1].descriptor_order, 0);
        assert_eq!(evidence[1].row_states, "01");
    }

    #[test]
    fn rejects_masks_with_the_wrong_width() {
        let info = TcInfo::parse(&sample_tcinfo([0, 1]), 0).unwrap();
        let error = build_descriptor_bitmap_evidence(&info, &["1".to_string()]).unwrap_err();
        assert!(error.to_string().contains("has 1 bits, expected 2"));
    }

    #[test]
    fn rejects_non_binary_mask_states() {
        let info = TcInfo::parse(&sample_tcinfo([0, 1]), 0).unwrap();
        let error = build_descriptor_bitmap_evidence(&info, &["1x".to_string()]).unwrap_err();
        assert!(error.to_string().contains("non-binary state"));
    }

    fn sample_tcinfo(bitmap_bits: [u8; 2]) -> Vec<u8> {
        let mut bytes = vec![0; 38];
        bytes[0] = 0x7c;
        bytes[1] = 2;
        bytes[2..4].copy_from_slice(&4u16.to_le_bytes());
        bytes[4..6].copy_from_slice(&8u16.to_le_bytes());
        bytes[6..8].copy_from_slice(&10u16.to_le_bytes());
        bytes[8..10].copy_from_slice(&12u16.to_le_bytes());
        bytes[10..14].copy_from_slice(&0x60u32.to_le_bytes());
        bytes[14..18].copy_from_slice(&0x74u32.to_le_bytes());
        bytes[18..22].copy_from_slice(&0x80u32.to_le_bytes());
        bytes[22..26].copy_from_slice(&0x001a0037u32.to_le_bytes());
        bytes[26..28].copy_from_slice(&0u16.to_le_bytes());
        bytes[28] = 4;
        bytes[29] = bitmap_bits[0];
        bytes[30..34].copy_from_slice(&0x001f3001u32.to_le_bytes());
        bytes[34..36].copy_from_slice(&4u16.to_le_bytes());
        bytes[36] = 4;
        bytes[37] = bitmap_bits[1];
        bytes
    }
}
