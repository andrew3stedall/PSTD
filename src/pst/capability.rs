use std::fs;
use std::path::Path;

use crate::pst::header::PstHeader;
use crate::pst::limits::InputLimits;
use crate::pst::primitives::PstVariant;
use crate::pst::reader::PstByteReader;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputFamily {
    UnicodePst,
    AnsiPst,
    Ost2013,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputCapabilityStatus {
    Ready,
    Partial,
    Unsupported,
    Unavailable,
    Malformed,
    BudgetExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InputCapability {
    pub input_path: String,
    pub file_size: u64,
    pub family: InputFamily,
    pub version: Option<u16>,
    pub index_type: Option<u8>,
    pub crypt_method: Option<u8>,
    pub header_status: String,
    pub index_status: String,
    pub extended_attributes_status: String,
    pub default_charset: String,
    pub bbt_root_offset: Option<u64>,
    pub nbt_root_offset: Option<u64>,
    pub root_condition: String,
    pub budget_status: String,
    pub status: InputCapabilityStatus,
    pub allows_extraction: bool,
    pub limits: InputLimits,
    pub diagnostics: Vec<String>,
}

impl InputCapability {
    pub fn from_header(
        input_path: impl Into<String>,
        header: &PstHeader,
        limits: InputLimits,
    ) -> Self {
        let family = match header.variant {
            PstVariant::Unicode => InputFamily::UnicodePst,
            PstVariant::Ansi => InputFamily::AnsiPst,
            PstVariant::Ost2013 => InputFamily::Ost2013,
            PstVariant::Unknown => InputFamily::Unknown,
        };
        let crypt_method = header.summary.crypt_method;
        let root_condition = header.summary.root_diagnostics.condition.clone();
        let roots_ready = root_condition == "root_pages_in_bounds";

        let (status, index_status, extended_attributes_status, allows_extraction) = match family {
            InputFamily::UnicodePst => match crypt_method {
                Some(0 | 1) if roots_ready => (
                    InputCapabilityStatus::Ready,
                    "ready_to_attempt".to_string(),
                    "not_loaded".to_string(),
                    true,
                ),
                Some(0 | 1) => (
                    InputCapabilityStatus::Partial,
                    format!("not_ready:{root_condition}"),
                    "unavailable_until_index_ready".to_string(),
                    false,
                ),
                Some(2) if roots_ready => (
                    InputCapabilityStatus::Ready,
                    "ready_to_attempt".to_string(),
                    "not_loaded".to_string(),
                    true,
                ),
                Some(2) => (
                    InputCapabilityStatus::Partial,
                    format!("not_ready:{root_condition}"),
                    "unavailable_until_index_ready".to_string(),
                    false,
                ),
                Some(method) => (
                    InputCapabilityStatus::Unsupported,
                    format!("unsupported_crypt_method:{method}"),
                    "unavailable_until_decryption".to_string(),
                    false,
                ),
                None => (
                    InputCapabilityStatus::Partial,
                    "crypt_method_unavailable".to_string(),
                    "unavailable_until_header_complete".to_string(),
                    false,
                ),
            },
            InputFamily::AnsiPst | InputFamily::Ost2013 => match crypt_method {
                Some(0 | 1) if roots_ready => (
                    InputCapabilityStatus::Ready,
                    "ready_to_attempt".to_string(),
                    "not_loaded".to_string(),
                    true,
                ),
                Some(0 | 1) => (
                    InputCapabilityStatus::Partial,
                    format!("not_ready:{root_condition}"),
                    "unavailable_until_index_ready".to_string(),
                    false,
                ),
                Some(2) if roots_ready => (
                    InputCapabilityStatus::Ready,
                    "ready_to_attempt".to_string(),
                    "not_loaded".to_string(),
                    true,
                ),
                Some(2) => (
                    InputCapabilityStatus::Partial,
                    format!("not_ready:{root_condition}"),
                    "unavailable_until_index_ready".to_string(),
                    false,
                ),
                Some(method) => (
                    InputCapabilityStatus::Unsupported,
                    format!("unsupported_crypt_method:{method}"),
                    "unavailable_until_decryption".to_string(),
                    false,
                ),
                None => (
                    InputCapabilityStatus::Partial,
                    "crypt_method_unavailable".to_string(),
                    "unavailable_until_header_complete".to_string(),
                    false,
                ),
            },
            InputFamily::Unknown => (
                InputCapabilityStatus::Unsupported,
                "unsupported_unknown_family".to_string(),
                "unavailable_for_unknown_family".to_string(),
                false,
            ),
        };

        let mut diagnostics = vec![
            format!("family={}", family.as_str()),
            format!("root_condition={root_condition}"),
            format!(
                "crypt_method={}",
                crypt_method
                    .map(|method| method.to_string())
                    .unwrap_or_else(|| "unavailable".to_string())
            ),
        ];
        if !roots_ready {
            diagnostics.push("root_pages_not_ready_for_traversal".to_string());
        }

        Self {
            input_path: input_path.into(),
            file_size: header.summary.file_size,
            family,
            version: header.summary.version,
            index_type: header.summary.index_type,
            crypt_method,
            header_status: header.summary.parser_status.clone(),
            index_status,
            extended_attributes_status,
            default_charset: "iso-8859-1".to_string(),
            bbt_root_offset: header.summary.bbt_root_offset,
            nbt_root_offset: header.summary.nbt_root_offset,
            root_condition,
            budget_status: "within_limits".to_string(),
            status,
            allows_extraction,
            limits,
            diagnostics,
        }
    }

    pub fn probe(input_path: impl AsRef<Path>, limits: InputLimits) -> Self {
        let path = input_path.as_ref();
        let display = path.display().to_string();
        let file_size = fs::metadata(path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        match PstByteReader::open_with_limits(path, &limits) {
            Ok(reader) => match PstHeader::parse(&reader) {
                Ok(header) => Self::from_header(display, &header, limits),
                Err(error) => Self::error(
                    display,
                    file_size,
                    limits,
                    InputCapabilityStatus::Malformed,
                    "header_parse_failed",
                    error.to_string(),
                ),
            },
            Err(error) => {
                let status = if error.to_string().contains("max_file_bytes") {
                    InputCapabilityStatus::BudgetExceeded
                } else {
                    InputCapabilityStatus::Unavailable
                };
                Self::error(
                    display,
                    file_size,
                    limits,
                    status,
                    "input_open_failed",
                    error.to_string(),
                )
            }
        }
    }

    pub fn record_index_status(
        &mut self,
        bbt_status: impl Into<String>,
        nbt_status: impl Into<String>,
    ) {
        let bbt_status = bbt_status.into();
        let nbt_status = nbt_status.into();
        let crypt_unsupported =
            self.status == InputCapabilityStatus::Unsupported && self.crypt_method.is_some();
        if !crypt_unsupported {
            self.index_status = format!("bbt={bbt_status}; nbt={nbt_status}");
        }
        if self.status == InputCapabilityStatus::Ready
            && (index_status_is_partial(&bbt_status) || index_status_is_partial(&nbt_status))
        {
            self.status = InputCapabilityStatus::Partial;
            self.allows_extraction = false;
            self.diagnostics
                .push("index_probe_reported_partial_readiness".to_string());
        }
    }

    pub fn record_extended_attributes_status(&mut self, status: impl Into<String>) {
        self.extended_attributes_status = status.into();
    }

    pub fn family_name(&self) -> &'static str {
        self.family.as_str()
    }

    fn error(
        input_path: String,
        file_size: u64,
        limits: InputLimits,
        status: InputCapabilityStatus,
        code: &str,
        detail: String,
    ) -> Self {
        Self {
            input_path,
            file_size,
            family: InputFamily::Unknown,
            version: None,
            index_type: None,
            crypt_method: None,
            header_status: code.to_string(),
            index_status: "unavailable".to_string(),
            extended_attributes_status: "unavailable".to_string(),
            default_charset: "iso-8859-1".to_string(),
            bbt_root_offset: None,
            nbt_root_offset: None,
            root_condition: "not_decoded".to_string(),
            budget_status: if status == InputCapabilityStatus::BudgetExceeded {
                "input_limit_exceeded".to_string()
            } else {
                "not_evaluated".to_string()
            },
            status,
            allows_extraction: false,
            limits,
            diagnostics: vec![detail],
        }
    }
}

fn index_status_is_partial(status: &str) -> bool {
    if status.starts_with("unavailable") || status.starts_with("error") {
        return true;
    }
    for field in ["traversal_errors=", "truncated_entries="] {
        if let Some(value) = status
            .split(field)
            .nth(1)
            .and_then(|value| value.split(';').next())
        {
            if value != "0" {
                return true;
            }
        }
    }
    false
}

impl InputFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnicodePst => "unicode_pst",
            Self::AnsiPst => "ansi_pst",
            Self::Ost2013 => "ost2013",
            Self::Unknown => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pst::header::PstHeader;
    use crate::pst::limits::InputLimits;
    use crate::pst::primitives::PstVariant;

    #[test]
    fn documents_iso88591_default_and_stable_statuses() {
        let limits = InputLimits::default();
        let mut header_bytes = vec![0u8; 514];
        header_bytes[0..4].copy_from_slice(&crate::pst::header::PST_MAGIC);
        header_bytes[10] = 0x17;
        header_bytes[48..56].copy_from_slice(&1024u64.to_le_bytes());
        header_bytes[56..64].copy_from_slice(&2048u64.to_le_bytes());
        header_bytes[513] = 0;
        let header = PstHeader::parse_bytes(&header_bytes, 4096).expect("header");
        assert_eq!(header.variant, PstVariant::Unicode);

        let capability = InputCapability::from_header("fixture.pst", &header, limits);
        assert_eq!(capability.family, InputFamily::UnicodePst);
        assert_eq!(capability.default_charset, "iso-8859-1");
        assert_eq!(capability.status, InputCapabilityStatus::Ready);
        assert!(capability.allows_extraction);
    }
}
