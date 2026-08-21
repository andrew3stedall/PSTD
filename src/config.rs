use std::path::PathBuf;

use crate::error::{PstdError, PstdResult};
use crate::pst::item_routing::{ItemRoutingPolicy, ItemTypeFilter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputProfile {
    Canonical,
    Mbox,
    RecursiveMbox,
    Mh,
    Eml,
    Separate,
    Kmail,
    Thunderbird,
    Vcard,
    ContactList,
    Icalendar,
    Vjournal,
    Msg,
}

impl OutputProfile {
    pub fn parse(raw: &str) -> PstdResult<Self> {
        let normalized = raw.trim().to_ascii_lowercase().replace('-', "_");
        let profile = match normalized.as_str() {
            "canonical" | "jsonl" | "tar" => Self::Canonical,
            "mbox" => Self::Mbox,
            "recursive_mbox" | "recursive" => Self::RecursiveMbox,
            "mh" => Self::Mh,
            "eml" => Self::Eml,
            "separate" => Self::Separate,
            "kmail" => Self::Kmail,
            "thunderbird" => Self::Thunderbird,
            "vcard" => Self::Vcard,
            "contact_list" | "contactlist" => Self::ContactList,
            "icalendar" | "ical" => Self::Icalendar,
            "vjournal" => Self::Vjournal,
            "msg" => Self::Msg,
            _ => {
                return Err(PstdError::InvalidConfig(format!(
                    "RPCLI_UNKNOWN_OUTPUT_PROFILE: {raw}"
                )))
            }
        };
        Ok(profile)
    }

    pub fn is_canonical(self) -> bool {
        self == Self::Canonical
    }

    pub fn is_supported(self) -> bool {
        matches!(
            self,
            Self::Canonical
                | Self::Mbox
                | Self::RecursiveMbox
                | Self::Mh
                | Self::Eml
                | Self::Separate
                | Self::Kmail
                | Self::Vcard
                | Self::ContactList
                | Self::Icalendar
                | Self::Vjournal
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticsPolicy {
    Errors,
    Info,
    Debug,
}

impl DiagnosticsPolicy {
    fn parse(raw: &str) -> PstdResult<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "errors" | "error" | "quiet" => Ok(Self::Errors),
            "info" | "normal" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            _ => Err(PstdError::InvalidConfig(format!(
                "RPCLI_INVALID_DIAGNOSTICS: {raw}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollisionPolicy {
    Suffix,
    Skip,
    Fail,
    Replace,
}

impl CollisionPolicy {
    fn parse(raw: &str) -> PstdResult<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "suffix" | "unique" => Ok(Self::Suffix),
            "skip" => Ok(Self::Skip),
            "fail" => Ok(Self::Fail),
            "replace" | "overwrite" => Ok(Self::Replace),
            _ => Err(PstdError::InvalidConfig(format!(
                "RPCLI_INVALID_COLLISION_POLICY: {raw}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReadpstPolicy {
    pub output_profile: OutputProfile,
    pub fallback_charset: Option<String>,
    pub prefer_utf8: bool,
    pub include_deleted: bool,
    pub include_associated: bool,
    pub item_type_filter: ItemTypeFilter,
    pub attachment_extensions: Vec<String>,
    pub emit_synthetic_rtf: bool,
    pub jobs: u16,
    pub diagnostics: DiagnosticsPolicy,
    pub collision: CollisionPolicy,
    pub overwrite: bool,
}

impl Default for ReadpstPolicy {
    fn default() -> Self {
        Self {
            output_profile: OutputProfile::Canonical,
            fallback_charset: None,
            prefer_utf8: false,
            include_deleted: false,
            include_associated: false,
            item_type_filter: ItemTypeFilter::All,
            attachment_extensions: Vec::new(),
            emit_synthetic_rtf: true,
            jobs: 1,
            diagnostics: DiagnosticsPolicy::Info,
            collision: CollisionPolicy::Suffix,
            overwrite: false,
        }
    }
}

impl ReadpstPolicy {
    pub fn from_flags(
        output_profile: &str,
        fallback_charset: Option<String>,
        prefer_utf8: bool,
        include_deleted: bool,
        include_associated: bool,
        item_types: &str,
        attachment_extensions: Option<&str>,
        emit_synthetic_rtf: bool,
        jobs: u16,
        diagnostics: &str,
        collision: &str,
        overwrite: bool,
    ) -> PstdResult<Self> {
        let output_profile = OutputProfile::parse(output_profile)?;
        let item_type_filter = parse_item_type_filter(item_types)?;
        let attachment_extensions = parse_attachment_extensions(attachment_extensions)?;
        let fallback_charset = fallback_charset
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let policy = Self {
            output_profile,
            fallback_charset,
            prefer_utf8,
            include_deleted,
            include_associated,
            item_type_filter,
            attachment_extensions,
            emit_synthetic_rtf,
            jobs,
            diagnostics: DiagnosticsPolicy::parse(diagnostics)?,
            collision: CollisionPolicy::parse(collision)?,
            overwrite,
        };
        policy.validate()?;
        Ok(policy)
    }

    pub fn validate(&self) -> PstdResult<()> {
        if !self.output_profile.is_supported() {
            return Err(PstdError::InvalidConfig(format!(
                "RPCLI_UNSUPPORTED_OUTPUT_PROFILE: {:?}; use a dedicated adapter when available",
                self.output_profile
            )));
        }
        if self.jobs == 0 || self.jobs > 64 {
            return Err(PstdError::InvalidConfig(format!(
                "RPCLI_INVALID_JOBS: {}; expected 1..=64",
                self.jobs
            )));
        }
        if let Some(charset) = &self.fallback_charset {
            if charset.len() > 64 || !charset.is_ascii() || charset.chars().any(char::is_whitespace)
            {
                return Err(PstdError::InvalidConfig(
                    "RPCLI_INVALID_FALLBACK_CHARSET: expected a short ASCII charset name"
                        .to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn routing_policy(&self) -> ItemRoutingPolicy {
        ItemRoutingPolicy {
            include_deleted: self.include_deleted,
            include_associated: self.include_associated,
            item_type_filter: self.item_type_filter,
        }
    }
}

fn parse_item_type_filter(raw: &str) -> PstdResult<ItemTypeFilter> {
    let values = raw
        .split(',')
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if values.is_empty() || values.iter().any(|value| value == "all") {
        if values.len() <= 1 || values.iter().all(|value| value == "all") {
            return Ok(ItemTypeFilter::All);
        }
        return Err(PstdError::InvalidConfig(
            "RPCLI_AMBIGUOUS_ITEM_TYPES: all cannot be combined with a typed filter".to_string(),
        ));
    }
    if values.len() != 1 {
        return Err(PstdError::InvalidConfig(
            "RPCLI_AMBIGUOUS_ITEM_TYPES: choose exactly one of e,a,j,c or all".to_string(),
        ));
    }
    match values[0].as_str() {
        "e" | "email" => Ok(ItemTypeFilter::Email),
        "a" | "appointment" => Ok(ItemTypeFilter::Appointment),
        "j" | "journal" => Ok(ItemTypeFilter::Journal),
        "c" | "contact" => Ok(ItemTypeFilter::Contact),
        _ => Err(PstdError::InvalidConfig(format!(
            "RPCLI_UNKNOWN_ITEM_TYPE: {}",
            values[0]
        ))),
    }
}

fn parse_attachment_extensions(raw: Option<&str>) -> PstdResult<Vec<String>> {
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };
    let mut extensions = Vec::new();
    for value in raw
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let normalized = value.trim_start_matches('.').to_ascii_lowercase();
        if normalized.is_empty()
            || !normalized
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-')
        {
            return Err(PstdError::InvalidConfig(format!(
                "RPCLI_INVALID_ATTACHMENT_EXTENSION: {value}"
            )));
        }
        if !extensions.contains(&normalized) {
            extensions.push(normalized);
        }
    }
    extensions.sort();
    Ok(extensions)
}

#[derive(Debug, Clone)]
pub struct ExtractConfig {
    pub input: PathBuf,
    pub output: PathBuf,
    pub continue_on_error: bool,
    pub overwrite: bool,
    pub manifest_only: bool,
    pub archive_format: String,
    pub data_format: String,
    pub tar_shard_size_mb: u64,
    pub progress: String,
    pub log_level: String,
    pub profile: String,
    pub readpst: ReadpstPolicy,
}

impl ExtractConfig {
    pub fn tar_shard_size_bytes(&self) -> u64 {
        self.tar_shard_size_mb
            .saturating_mul(1024)
            .saturating_mul(1024)
    }
}
