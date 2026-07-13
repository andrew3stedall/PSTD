//! Classification for Table Context property identifiers observed during extraction.
//!
//! This module deliberately distinguishes table-internal LTP bookkeeping from
//! message properties that may contain user-readable email metadata. It does
//! not infer semantics for unknown identifiers.

pub const PID_TAG_LTP_ROW_ID: u16 = 0x67f2;
pub const PID_TAG_LTP_ROW_VER: u16 = 0x67f3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcPropertyRole {
    /// Internal Table Context/LTP bookkeeping. These values are structurally
    /// useful but are not user-readable message metadata.
    TableInternal,
    /// The identifier is not classified by the bounded registry.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TcPropertyClassification {
    pub property_id: u16,
    pub canonical_name: Option<&'static str>,
    pub role: TcPropertyRole,
}

impl TcPropertyClassification {
    pub fn is_user_readable_candidate(self) -> bool {
        !matches!(self.role, TcPropertyRole::TableInternal)
    }
}

/// Classifies only identifiers whose role is authoritative and required by the
/// current extraction evidence. Unknown identifiers remain explicitly unknown.
pub fn classify_tc_property(property_tag: u32) -> TcPropertyClassification {
    let property_id = (property_tag >> 16) as u16;
    match property_id {
        PID_TAG_LTP_ROW_ID => TcPropertyClassification {
            property_id,
            canonical_name: Some("PidTagLtpRowId"),
            role: TcPropertyRole::TableInternal,
        },
        PID_TAG_LTP_ROW_VER => TcPropertyClassification {
            property_id,
            canonical_name: Some("PidTagLtpRowVer"),
            role: TcPropertyRole::TableInternal,
        },
        _ => TcPropertyClassification {
            property_id,
            canonical_name: None,
            role: TcPropertyRole::Unknown,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_tc_property, TcPropertyRole};

    #[test]
    fn classifies_the_observed_67f2_property_as_ltp_row_identity() {
        let classification = classify_tc_property(0x67f2_0003);

        assert_eq!(classification.property_id, 0x67f2);
        assert_eq!(classification.canonical_name, Some("PidTagLtpRowId"));
        assert_eq!(classification.role, TcPropertyRole::TableInternal);
        assert!(!classification.is_user_readable_candidate());
    }

    #[test]
    fn classifies_ltp_row_version_as_table_internal() {
        let classification = classify_tc_property(0x67f3_0003);

        assert_eq!(classification.canonical_name, Some("PidTagLtpRowVer"));
        assert_eq!(classification.role, TcPropertyRole::TableInternal);
        assert!(!classification.is_user_readable_candidate());
    }

    #[test]
    fn leaves_unverified_properties_unknown_without_guessing() {
        let classification = classify_tc_property(0x3001_001f);

        assert_eq!(classification.property_id, 0x3001);
        assert_eq!(classification.canonical_name, None);
        assert_eq!(classification.role, TcPropertyRole::Unknown);
        assert!(classification.is_user_readable_candidate());
    }
}
