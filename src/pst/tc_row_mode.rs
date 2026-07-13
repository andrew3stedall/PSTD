use crate::pst::tc_row_offsets::{derive_validated_row_offsets, TcRowAddressMode};

pub fn select_validated_row_address_mode(
    row_references: &[u32],
    row_width: usize,
    row_data_byte_len: usize,
    direct_fixed_width_rows: bool,
    ordinal_index_rows: bool,
) -> Result<TcRowAddressMode, String> {
    let mode = match (direct_fixed_width_rows, ordinal_index_rows) {
        (true, false) => TcRowAddressMode::DirectOffsets,
        (false, true) => TcRowAddressMode::OrdinalIndices,
        (false, false) => return Err("validated row address mode is unavailable".to_string()),
        (true, true) => return Err("validated row address mode is ambiguous".to_string()),
    };

    derive_validated_row_offsets(row_references, row_width, row_data_byte_len, mode)?;
    Ok(mode)
}

#[cfg(test)]
mod tests {
    use super::select_validated_row_address_mode;
    use crate::pst::tc_row_offsets::TcRowAddressMode;

    #[test]
    fn selects_validated_direct_offsets() {
        let mode = select_validated_row_address_mode(&[0, 52, 104, 156], 52, 208, true, false)
            .expect("direct layout evidence should select direct addressing");

        assert_eq!(mode, TcRowAddressMode::DirectOffsets);
    }

    #[test]
    fn selects_validated_ordinal_indices() {
        let mode = select_validated_row_address_mode(&[0, 1, 2, 3], 52, 208, false, true)
            .expect("ordinal layout evidence should select ordinal addressing");

        assert_eq!(mode, TcRowAddressMode::OrdinalIndices);
    }

    #[test]
    fn rejects_unavailable_layout_evidence() {
        let error = select_validated_row_address_mode(&[0, 1], 52, 104, false, false)
            .expect_err("missing layout evidence must fail closed");

        assert_eq!(error, "validated row address mode is unavailable");
    }

    #[test]
    fn rejects_ambiguous_layout_evidence() {
        let error = select_validated_row_address_mode(&[0, 52], 52, 104, true, true)
            .expect_err("conflicting layout evidence must fail closed");

        assert_eq!(error, "validated row address mode is ambiguous");
    }

    #[test]
    fn rejects_selected_mode_when_offsets_do_not_validate() {
        let error = select_validated_row_address_mode(&[0, 1, 4], 52, 208, false, true)
            .expect_err("mode selection must retain the PQ65 bounds invariant");

        assert_eq!(error, "derived row is outside the resolved payload");
    }
}
