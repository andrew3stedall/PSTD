use crate::pst::tcinfo::TcColumnDescriptor;
use std::collections::BTreeSet;

const PT_I2: u16 = 0x0002;
const PT_LONG: u16 = 0x0003;
const PT_BOOLEAN: u16 = 0x000b;
const PT_I8: u16 = 0x0014;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FixedWidthRowEvidence {
    pub bitmap_index: u8,
    pub descriptor_order: usize,
    pub property_tag: u32,
    pub data_offset: u16,
    pub data_size: u8,
    pub row_values_hex: Vec<String>,
    pub decoded_values: Vec<String>,
    pub distinct_value_count: usize,
}

pub fn select_fixed_width_row_evidence(
    columns: &[TcColumnDescriptor],
    bitmap_masks: &[String],
    row_data: &[u8],
    row_offsets: &[usize],
    row_width: usize,
    fixed_data_end: usize,
) -> Result<FixedWidthRowEvidence, String> {
    if columns.is_empty() || bitmap_masks.is_empty() || row_offsets.is_empty() {
        return Err("fixed-width evidence inputs unavailable".to_string());
    }
    if bitmap_masks.len() != row_offsets.len() {
        return Err("bitmap mask and row offset counts differ".to_string());
    }
    if row_width == 0 || fixed_data_end > row_width {
        return Err("fixed-width row boundary is invalid".to_string());
    }
    if bitmap_masks.iter().any(|mask| {
        mask.len() != columns.len() || mask.bytes().any(|value| value != b'0' && value != b'1')
    }) {
        return Err("bitmap masks are incomplete or non-binary".to_string());
    }

    let mut candidates = Vec::new();
    for (descriptor_order, column) in columns.iter().enumerate() {
        let bitmap_index = column.bitmap_bit as usize;
        let end = column.data_offset as usize + column.data_size as usize;
        if !is_supported_fixed_width_type(column.property_tag, column.data_size)
            || bitmap_index >= columns.len()
            || end > fixed_data_end
            || bitmap_masks
                .iter()
                .any(|mask| mask.as_bytes()[bitmap_index] != b'1')
        {
            continue;
        }

        let mut row_values_hex = Vec::with_capacity(row_offsets.len());
        let mut decoded_values = Vec::with_capacity(row_offsets.len());
        for row_offset in row_offsets {
            let row_end = row_offset
                .checked_add(row_width)
                .ok_or_else(|| "row end overflow".to_string())?;
            let start = row_offset
                .checked_add(column.data_offset as usize)
                .ok_or_else(|| "column start overflow".to_string())?;
            let value_end = start
                .checked_add(column.data_size as usize)
                .ok_or_else(|| "column end overflow".to_string())?;
            if row_end > row_data.len() || value_end > row_end {
                return Err("fixed-width value is outside the validated row".to_string());
            }
            let value = &row_data[start..value_end];
            row_values_hex.push(
                value
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>(),
            );
            decoded_values.push(decode_fixed_width_value(column.property_tag, value)?);
        }

        let distinct_value_count = decoded_values.iter().collect::<BTreeSet<_>>().len();
        candidates.push(FixedWidthRowEvidence {
            bitmap_index: column.bitmap_bit,
            descriptor_order,
            property_tag: column.property_tag,
            data_offset: column.data_offset,
            data_size: column.data_size,
            row_values_hex,
            decoded_values,
            distinct_value_count,
        });
    }

    candidates
        .into_iter()
        .max_by(|left, right| {
            left.distinct_value_count
                .cmp(&right.distinct_value_count)
                .then_with(|| right.bitmap_index.cmp(&left.bitmap_index))
        })
        .ok_or_else(|| "no bounded supported fixed-width descriptor is set in every row".to_string())
}

fn is_supported_fixed_width_type(property_tag: u32, data_size: u8) -> bool {
    matches!(
        (property_tag as u16, data_size),
        (PT_I2, 2) | (PT_LONG, 4) | (PT_BOOLEAN, 2) | (PT_I8, 8)
    )
}

fn decode_fixed_width_value(property_tag: u32, value: &[u8]) -> Result<String, String> {
    match property_tag as u16 {
        PT_I2 if value.len() == 2 => Ok(i16::from_le_bytes([value[0], value[1]]).to_string()),
        PT_LONG if value.len() == 4 => Ok(i32::from_le_bytes([
            value[0], value[1], value[2], value[3],
        ])
        .to_string()),
        PT_BOOLEAN if value.len() == 2 => match u16::from_le_bytes([value[0], value[1]]) {
            0 => Ok("false".to_string()),
            1 => Ok("true".to_string()),
            _ => Err("MAPI boolean value is neither zero nor one".to_string()),
        },
        PT_I8 if value.len() == 8 => Ok(i64::from_le_bytes([
            value[0], value[1], value[2], value[3], value[4], value[5], value[6], value[7],
        ])
        .to_string()),
        _ => Err("unsupported or size-mismatched fixed-width MAPI type".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::select_fixed_width_row_evidence;
    use crate::pst::tcinfo::TcColumnDescriptor;

    #[test]
    fn selects_and_decodes_the_candidate_with_the_clearest_distinct_values() {
        let columns = vec![
            descriptor(0, 0, 4, 0x0003),
            descriptor(1, 4, 4, 0x0003),
            descriptor(2, 8, 2, 0x0002),
        ];
        let masks = vec!["110".to_string(); 4];
        let mut rows = vec![0u8; 48];
        for row in 0..4 {
            let base = row * 12;
            rows[base..base + 4].copy_from_slice(&1i32.to_le_bytes());
            rows[base + 4..base + 8].copy_from_slice(&((row + 1) as i32).to_le_bytes());
        }

        let evidence =
            select_fixed_width_row_evidence(&columns, &masks, &rows, &[0, 12, 24, 36], 12, 10)
                .expect("bounded evidence should validate");

        assert_eq!(evidence.bitmap_index, 1);
        assert_eq!(evidence.descriptor_order, 1);
        assert_eq!(
            evidence.row_values_hex,
            vec!["01000000", "02000000", "03000000", "04000000"]
        );
        assert_eq!(evidence.decoded_values, vec!["1", "2", "3", "4"]);
        assert_eq!(evidence.distinct_value_count, 4);
    }

    #[test]
    fn decodes_signed_short_long_and_i8_values() {
        let cases = [
            (descriptor(0, 0, 2, 0x0002), (-2i16).to_le_bytes().to_vec(), "-2"),
            (descriptor(0, 0, 4, 0x0003), (-3i32).to_le_bytes().to_vec(), "-3"),
            (descriptor(0, 0, 8, 0x0014), (-4i64).to_le_bytes().to_vec(), "-4"),
        ];

        for (column, bytes, expected) in cases {
            let width = bytes.len();
            let evidence = select_fixed_width_row_evidence(
                &[column],
                &["1".to_string()],
                &bytes,
                &[0],
                width,
                width,
            )
            .expect("supported scalar should decode");
            assert_eq!(evidence.decoded_values, vec![expected]);
        }
    }

    #[test]
    fn decodes_canonical_mapi_booleans() {
        for (bytes, expected) in [([0u8, 0u8], "false"), ([1u8, 0u8], "true")] {
            let evidence = select_fixed_width_row_evidence(
                &[descriptor(0, 0, 2, 0x000b)],
                &["1".to_string()],
                &bytes,
                &[0],
                2,
                2,
            )
            .expect("canonical boolean should decode");
            assert_eq!(evidence.decoded_values, vec![expected]);
        }
    }

    #[test]
    fn rejects_noncanonical_mapi_boolean_without_partial_evidence() {
        let error = select_fixed_width_row_evidence(
            &[descriptor(0, 0, 2, 0x000b)],
            &["1".to_string()],
            &[2, 0],
            &[0],
            2,
            2,
        )
        .expect_err("noncanonical boolean must fail closed");

        assert_eq!(error, "MAPI boolean value is neither zero nor one");
    }

    #[test]
    fn uses_the_lowest_bitmap_index_as_a_deterministic_tie_breaker() {
        let columns = vec![
            descriptor(1, 4, 4, 0x0003),
            descriptor(0, 0, 4, 0x0003),
        ];
        let masks = vec!["11".to_string(); 2];
        let rows = vec![0u8; 16];

        let evidence = select_fixed_width_row_evidence(&columns, &masks, &rows, &[0, 8], 8, 8)
            .expect("one candidate should be selected");

        assert_eq!(evidence.bitmap_index, 0);
        assert_eq!(evidence.descriptor_order, 1);
    }

    #[test]
    fn rejects_out_of_bounds_rows_without_partial_evidence() {
        let error = select_fixed_width_row_evidence(
            &[descriptor(0, 4, 4, 0x0003)],
            &["1".to_string()],
            &[0; 7],
            &[0],
            8,
            8,
        )
        .expect_err("truncated rows must fail closed");

        assert!(error.contains("outside the validated row"));
    }

    #[test]
    fn rejects_masks_that_do_not_validate_the_candidate_for_every_row() {
        let error = select_fixed_width_row_evidence(
            &[descriptor(0, 0, 4, 0x0003)],
            &["1".to_string(), "0".to_string()],
            &[0; 16],
            &[0, 8],
            8,
            8,
        )
        .expect_err("partially set candidates must be excluded");

        assert_eq!(
            error,
            "no bounded supported fixed-width descriptor is set in every row"
        );
    }

    #[test]
    fn ignores_unsupported_fixed_width_types() {
        let error = select_fixed_width_row_evidence(
            &[descriptor(0, 0, 4, 0x0004)],
            &["1".to_string()],
            &[0; 4],
            &[0],
            4,
            4,
        )
        .expect_err("unsupported type must not be guessed");

        assert_eq!(
            error,
            "no bounded supported fixed-width descriptor is set in every row"
        );
    }

    fn descriptor(
        bitmap_bit: u8,
        data_offset: u16,
        data_size: u8,
        property_type: u16,
    ) -> TcColumnDescriptor {
        TcColumnDescriptor {
            property_tag: (0x3001u32 << 16) | u32::from(property_type),
            data_offset,
            data_size,
            bitmap_bit,
        }
    }
}
