#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcRowAddressMode {
    DirectOffsets,
    OrdinalIndices,
}

pub fn derive_validated_row_offsets(
    row_references: &[u32],
    row_width: usize,
    row_data_byte_len: usize,
    mode: TcRowAddressMode,
) -> Result<Vec<usize>, String> {
    if row_references.is_empty() {
        return Err("row references are unavailable".to_string());
    }
    if row_width == 0 {
        return Err("row width is unavailable".to_string());
    }

    let mut offsets = Vec::with_capacity(row_references.len());
    for reference in row_references {
        let offset = match mode {
            TcRowAddressMode::DirectOffsets => *reference as usize,
            TcRowAddressMode::OrdinalIndices => (*reference as usize)
                .checked_mul(row_width)
                .ok_or_else(|| "ordinal row offset overflow".to_string())?,
        };
        let end = offset
            .checked_add(row_width)
            .ok_or_else(|| "row end overflow".to_string())?;
        if end > row_data_byte_len {
            return Err("derived row is outside the resolved payload".to_string());
        }
        offsets.push(offset);
    }

    if offsets.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err("derived row offsets are not strictly increasing".to_string());
    }

    Ok(offsets)
}

#[cfg(test)]
mod tests {
    use super::{derive_validated_row_offsets, TcRowAddressMode};

    #[test]
    fn preserves_validated_direct_offsets() {
        let offsets = derive_validated_row_offsets(
            &[0, 52, 104, 156],
            52,
            208,
            TcRowAddressMode::DirectOffsets,
        )
        .expect("direct offsets should validate");

        assert_eq!(offsets, vec![0, 52, 104, 156]);
    }

    #[test]
    fn converts_validated_ordinal_indices() {
        let offsets = derive_validated_row_offsets(
            &[0, 1, 2, 3],
            52,
            208,
            TcRowAddressMode::OrdinalIndices,
        )
        .expect("ordinal indices should validate");

        assert_eq!(offsets, vec![0, 52, 104, 156]);
    }

    #[test]
    fn rejects_rows_outside_the_payload_without_partial_offsets() {
        let error = derive_validated_row_offsets(
            &[0, 1, 4],
            52,
            208,
            TcRowAddressMode::OrdinalIndices,
        )
        .expect_err("out-of-bounds rows must fail closed");

        assert_eq!(error, "derived row is outside the resolved payload");
    }

    #[test]
    fn rejects_non_increasing_direct_offsets() {
        let error = derive_validated_row_offsets(
            &[0, 52, 52],
            52,
            208,
            TcRowAddressMode::DirectOffsets,
        )
        .expect_err("duplicate offsets must fail closed");

        assert_eq!(error, "derived row offsets are not strictly increasing");
    }
}