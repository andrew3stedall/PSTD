use crate::pst::tcinfo::TcColumnDescriptor;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FixedWidthRowEvidence {
    pub bitmap_index: u8,
    pub descriptor_order: usize,
    pub property_tag: u32,
    pub data_offset: u16,
    pub data_size: u8,
    pub row_values_hex: Vec<String>,
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
        if column.data_size != 4
            || bitmap_index >= columns.len()
            || end > fixed_data_end
            || bitmap_masks
                .iter()
                .any(|mask| mask.as_bytes()[bitmap_index] != b'1')
        {
            continue;
        }

        let mut row_values_hex = Vec::with_capacity(row_offsets.len());
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
            row_values_hex.push(
                row_data[start..value_end]
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>(),
            );
        }

        let distinct_value_count = row_values_hex.iter().collect::<BTreeSet<_>>().len();
        candidates.push(FixedWidthRowEvidence {
            bitmap_index: column.bitmap_bit,
            descriptor_order,
            property_tag: column.property_tag,
            data_offset: column.data_offset,
            data_size: column.data_size,
            row_values_hex,
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
        .ok_or_else(|| "no bounded four-byte descriptor is set in every row".to_string())
}

#[cfg(test)]
mod tests {
    use super::select_fixed_width_row_evidence;
    use crate::pst::tcinfo::TcColumnDescriptor;

    #[test]
    fn selects_the_candidate_with_the_clearest_distinct_row_bytes() {
        let columns = vec![
            descriptor(0, 0, 4),
            descriptor(1, 4, 4),
            descriptor(2, 8, 2),
        ];
        let masks = vec!["110".to_string(); 4];
        let mut rows = vec![0u8; 48];
        for row in 0..4 {
            let base = row * 12;
            rows[base..base + 4].copy_from_slice(&[1, 0, 0, 0]);
            rows[base + 4..base + 8].copy_from_slice(&[(row + 1) as u8, 0, 0, 0]);
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
        assert_eq!(evidence.distinct_value_count, 4);
    }

    #[test]
    fn uses_the_lowest_bitmap_index_as_a_deterministic_tie_breaker() {
        let columns = vec![descriptor(1, 4, 4), descriptor(0, 0, 4)];
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
            &[descriptor(0, 4, 4)],
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
            &[descriptor(0, 0, 4)],
            &["1".to_string(), "0".to_string()],
            &[0; 16],
            &[0, 8],
            8,
            8,
        )
        .expect_err("partially set candidates must be excluded");

        assert_eq!(error, "no bounded four-byte descriptor is set in every row");
    }

    fn descriptor(bitmap_bit: u8, data_offset: u16, data_size: u8) -> TcColumnDescriptor {
        TcColumnDescriptor {
            property_tag: 0x001f3001 + u32::from(bitmap_bit),
            data_offset,
            data_size,
            bitmap_bit,
        }
    }
}
