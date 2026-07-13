use crate::pst::tc_row_offsets::{derive_validated_row_offsets, TcRowAddressMode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcRowTransportEvidence {
    pub payload: Vec<u8>,
    pub absolute_row_offsets: Vec<usize>,
    pub row_width: usize,
    pub address_mode: TcRowAddressMode,
}

pub fn build_validated_row_transport(
    row_payload: &[u8],
    row_references: &[u32],
    row_width: usize,
    address_mode: TcRowAddressMode,
) -> Result<TcRowTransportEvidence, String> {
    let absolute_row_offsets =
        derive_validated_row_offsets(row_references, row_width, row_payload.len(), address_mode)?;

    if absolute_row_offsets.len() != row_references.len() {
        return Err("validated row offset count does not match references".to_string());
    }

    Ok(TcRowTransportEvidence {
        payload: row_payload.to_vec(),
        absolute_row_offsets,
        row_width,
        address_mode,
    })
}

#[cfg(test)]
mod tests {
    use super::build_validated_row_transport;
    use crate::pst::tc_row_offsets::TcRowAddressMode;

    #[test]
    fn transports_ordinal_rows_with_absolute_offsets() {
        let payload = (0..208).map(|value| value as u8).collect::<Vec<_>>();

        let evidence = build_validated_row_transport(
            &payload,
            &[0, 1, 2, 3],
            52,
            TcRowAddressMode::OrdinalIndices,
        )
        .expect("ordinal rows should build complete transport evidence");

        assert_eq!(evidence.payload, payload);
        assert_eq!(evidence.absolute_row_offsets, vec![0, 52, 104, 156]);
        assert_eq!(evidence.row_width, 52);
        assert_eq!(evidence.address_mode, TcRowAddressMode::OrdinalIndices);
    }

    #[test]
    fn transports_direct_rows_without_reinterpreting_offsets() {
        let payload = vec![0x5a; 16];

        let evidence = build_validated_row_transport(
            &payload,
            &[0, 4, 8, 12],
            4,
            TcRowAddressMode::DirectOffsets,
        )
        .expect("direct rows should build complete transport evidence");

        assert_eq!(evidence.absolute_row_offsets, vec![0, 4, 8, 12]);
        assert_eq!(evidence.payload.len(), 16);
    }

    #[test]
    fn rejects_out_of_bounds_rows_without_retaining_payload() {
        let error = build_validated_row_transport(
            &[0; 12],
            &[0, 1, 3],
            4,
            TcRowAddressMode::OrdinalIndices,
        )
        .expect_err("out-of-bounds rows must fail closed");

        assert_eq!(error, "derived row is outside the resolved payload");
    }

    #[test]
    fn rejects_empty_references_without_retaining_payload() {
        let error =
            build_validated_row_transport(&[0; 12], &[], 4, TcRowAddressMode::DirectOffsets)
                .expect_err("missing references must fail closed");

        assert_eq!(error, "row references are unavailable");
    }
}
