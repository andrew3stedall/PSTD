use crate::error::{PstdError, PstdResult};
use crate::pst::binary::{u16_le_at, u64_le_at};
use crate::pst::primitives::{ByteOffset, PageRef, PstVariant, RootPointers};
use crate::pst::reader::PstByteReader;

pub const PST_HEADER_MIN_BYTES: usize = 64;
pub const PST_HEADER_ROOT_CANDIDATE_BYTES: usize = 248;
pub const PST_MAGIC: [u8; 4] = [0x21, 0x42, 0x44, 0x4e];
pub const PST_ROOT_PAGE_SIZE_BYTES: u64 = 512;

const LEGACY_NBT_ROOT_OFFSET_FIELD: usize = 48;
const LEGACY_BBT_ROOT_OFFSET_FIELD: usize = 56;
const UNICODE_ROOT_BASE_OFFSET: usize = 180;
const UNICODE_ROOT_NBT_BREF_OFFSET: usize = UNICODE_ROOT_BASE_OFFSET + 36;
const UNICODE_ROOT_BBT_BREF_OFFSET: usize = UNICODE_ROOT_BASE_OFFSET + 52;
const BREF_IB_OFFSET: usize = 8;
const UNICODE_NBT_ROOT_OFFSET_FIELD: usize = UNICODE_ROOT_NBT_BREF_OFFSET + BREF_IB_OFFSET;
const UNICODE_BBT_ROOT_OFFSET_FIELD: usize = UNICODE_ROOT_BBT_BREF_OFFSET + BREF_IB_OFFSET;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PstRootPointerDiagnostic {
    pub name: String,
    pub offset: Option<u64>,
    pub file_size: u64,
    pub offset_in_bounds: bool,
    pub root_page_in_bounds: bool,
    pub bytes_beyond_file_size: Option<u64>,
    pub condition: String,
}

impl PstRootPointerDiagnostic {
    pub fn classify(name: impl Into<String>, offset: Option<u64>, file_size: u64) -> Self {
        let name = name.into();
        let (offset_in_bounds, root_page_in_bounds, bytes_beyond_file_size, condition) =
            match offset {
                None => (false, false, None, "root_pointer_absent".to_string()),
                Some(value) if value >= file_size => (
                    false,
                    false,
                    Some(value.saturating_sub(file_size)),
                    "root_offset_beyond_file_size".to_string(),
                ),
                Some(value) => {
                    let page_end = value.saturating_add(PST_ROOT_PAGE_SIZE_BYTES);
                    if page_end > file_size {
                        (
                            true,
                            false,
                            Some(page_end.saturating_sub(file_size)),
                            "root_page_truncated".to_string(),
                        )
                    } else {
                        (true, true, None, "root_page_in_bounds".to_string())
                    }
                }
            };

        Self {
            name,
            offset,
            file_size,
            offset_in_bounds,
            root_page_in_bounds,
            bytes_beyond_file_size,
            condition,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PstRootCandidateDiagnostic {
    pub source: String,
    pub bbt_root: PstRootPointerDiagnostic,
    pub nbt_root: PstRootPointerDiagnostic,
    pub selectable_for_traversal: bool,
    pub condition: String,
}

impl PstRootCandidateDiagnostic {
    pub fn from_offsets(
        source: impl Into<String>,
        file_size: u64,
        bbt_root_offset: Option<u64>,
        nbt_root_offset: Option<u64>,
    ) -> Self {
        let source = source.into();
        let bbt_root = PstRootPointerDiagnostic::classify("bbt_root", bbt_root_offset, file_size);
        let nbt_root = PstRootPointerDiagnostic::classify("nbt_root", nbt_root_offset, file_size);
        let condition = classify_root_pair(&bbt_root, &nbt_root);
        let selectable_for_traversal = condition == "root_pages_in_bounds";

        Self {
            source,
            bbt_root,
            nbt_root,
            selectable_for_traversal,
            condition,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PstRootDiagnostics {
    pub file_size: u64,
    pub root_page_size_bytes: u64,
    pub bbt_root: PstRootPointerDiagnostic,
    pub nbt_root: PstRootPointerDiagnostic,
    pub condition: String,
    pub selected_source: Option<String>,
    pub candidate_count: usize,
    pub candidates: Vec<PstRootCandidateDiagnostic>,
    pub recommendation: String,
}

impl PstRootDiagnostics {
    pub fn from_candidates(file_size: u64, candidates: Vec<PstRootCandidateDiagnostic>) -> Self {
        let selected = candidates
            .iter()
            .find(|candidate| candidate.selectable_for_traversal);
        let display_candidate = selected
            .or_else(|| candidates.first())
            .cloned()
            .unwrap_or_else(|| {
                PstRootCandidateDiagnostic::from_offsets("none", file_size, None, None)
            });
        let condition = selected
            .map(|candidate| candidate.condition.clone())
            .unwrap_or_else(|| classify_unusable_candidates(&candidates));
        let selected_source = selected.map(|candidate| candidate.source.clone());
        let recommendation = match condition.as_str() {
            "root_pages_in_bounds" => "Selected root pages are safe to attempt traversal.",
            "root_candidates_unusable" => {
                "No decoded root candidate pair is safe for traversal; classify the fixture or add a decoder candidate."
            }
            "root_offsets_out_of_bounds" => {
                "Verify root offsets, endian handling, and fixture completeness before traversal."
            }
            "root_pages_truncated" => {
                "Treat this fixture as possibly truncated unless header decoding proves otherwise."
            }
            "root_pointers_absent" => "Root pointers are absent; classify as header-only until another root source is supported.",
            _ => "Review individual root pointer diagnostics before parser-quality work continues.",
        }
        .to_string();

        Self {
            file_size,
            root_page_size_bytes: PST_ROOT_PAGE_SIZE_BYTES,
            bbt_root: display_candidate.bbt_root,
            nbt_root: display_candidate.nbt_root,
            condition,
            selected_source,
            candidate_count: candidates.len(),
            candidates,
            recommendation,
        }
    }

    pub fn from_offsets(
        file_size: u64,
        bbt_root_offset: Option<u64>,
        nbt_root_offset: Option<u64>,
    ) -> Self {
        Self::from_candidates(
            file_size,
            vec![PstRootCandidateDiagnostic::from_offsets(
                "legacy_header_fields",
                file_size,
                bbt_root_offset,
                nbt_root_offset,
            )],
        )
    }
}

fn classify_unusable_candidates(candidates: &[PstRootCandidateDiagnostic]) -> String {
    if candidates.is_empty()
        || candidates
            .iter()
            .all(|candidate| candidate.condition == "root_pointers_absent")
    {
        "root_pointers_absent".to_string()
    } else if candidates
        .iter()
        .any(|candidate| candidate.condition == "root_pages_truncated")
    {
        "root_pages_truncated".to_string()
    } else {
        "root_candidates_unusable".to_string()
    }
}

fn classify_root_pair(
    bbt_root: &PstRootPointerDiagnostic,
    nbt_root: &PstRootPointerDiagnostic,
) -> String {
    let bbt_condition = bbt_root.condition.as_str();
    let nbt_condition = nbt_root.condition.as_str();
    if bbt_condition == "root_pointer_absent" && nbt_condition == "root_pointer_absent" {
        "root_pointers_absent".to_string()
    } else if bbt_condition == "root_offset_beyond_file_size"
        || nbt_condition == "root_offset_beyond_file_size"
    {
        "root_offsets_out_of_bounds".to_string()
    } else if bbt_condition == "root_page_truncated" || nbt_condition == "root_page_truncated" {
        "root_pages_truncated".to_string()
    } else if bbt_condition == "root_page_in_bounds" && nbt_condition == "root_page_in_bounds" {
        "root_pages_in_bounds".to_string()
    } else {
        "root_diagnostics_partial".to_string()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PstHeaderSummary {
    pub format: String,
    pub parser_status: String,
    pub file_size: u64,
    pub magic: String,
    pub magic_client: Option<String>,
    pub version: Option<u16>,
    pub variant: String,
    pub bbt_root_offset: Option<u64>,
    pub nbt_root_offset: Option<u64>,
    pub root_diagnostics: PstRootDiagnostics,
}

#[derive(Debug, Clone)]
pub struct PstHeader {
    pub summary: PstHeaderSummary,
    pub variant: PstVariant,
    pub roots: RootPointers,
}

impl PstHeader {
    pub fn parse(reader: &PstByteReader) -> PstdResult<Self> {
        let header_bytes = reader.read_prefix(PST_HEADER_ROOT_CANDIDATE_BYTES)?;
        Self::parse_bytes(&header_bytes, reader.file_size())
    }

    pub fn parse_bytes(buf: &[u8], file_size: u64) -> PstdResult<Self> {
        if buf.len() < PST_HEADER_MIN_BYTES {
            return Err(PstdError::pst_parse(
                Some(0),
                format!("PST header too short: {} bytes", buf.len()),
            ));
        }

        if buf[0..4] != PST_MAGIC {
            return Err(PstdError::pst_parse(Some(0), "missing PST magic !BDN"));
        }

        let magic_client = String::from_utf8_lossy(&buf[8..10]).to_string();
        let version = u16_le_at(buf, 10, 0)?;
        let variant = match version {
            23 | 36 => PstVariant::Unicode,
            14 | 15 => PstVariant::Ansi,
            _ => PstVariant::Unknown,
        };

        let legacy_bbt_root_offset = read_optional_offset(buf, LEGACY_BBT_ROOT_OFFSET_FIELD)?;
        let legacy_nbt_root_offset = read_optional_offset(buf, LEGACY_NBT_ROOT_OFFSET_FIELD)?;
        let candidates = build_root_candidates(
            buf,
            file_size,
            variant,
            legacy_bbt_root_offset,
            legacy_nbt_root_offset,
        )?;
        let root_diagnostics = PstRootDiagnostics::from_candidates(file_size, candidates);
        let selected_bbt_root_offset = root_diagnostics
            .selected_source
            .as_ref()
            .and(root_diagnostics.bbt_root.offset);
        let selected_nbt_root_offset = root_diagnostics
            .selected_source
            .as_ref()
            .and(root_diagnostics.nbt_root.offset);
        let roots = RootPointers {
            bbt_root: selected_bbt_root_offset.map(|offset| PageRef {
                offset: ByteOffset(offset),
            }),
            nbt_root: selected_nbt_root_offset.map(|offset| PageRef {
                offset: ByteOffset(offset),
            }),
        };

        let summary = PstHeaderSummary {
            format: "pst".to_string(),
            parser_status: match variant {
                PstVariant::Unicode => "supported_unicode_header".to_string(),
                PstVariant::Ansi => "detected_ansi_header_unsupported_for_extraction".to_string(),
                PstVariant::Unknown => "detected_unknown_version".to_string(),
            },
            file_size,
            magic: "!BDN".to_string(),
            magic_client: Some(magic_client),
            version: Some(version),
            variant: format!("{:?}", variant).to_lowercase(),
            bbt_root_offset: selected_bbt_root_offset,
            nbt_root_offset: selected_nbt_root_offset,
            root_diagnostics,
        };

        Ok(Self {
            summary,
            variant,
            roots,
        })
    }
}

fn build_root_candidates(
    buf: &[u8],
    file_size: u64,
    variant: PstVariant,
    legacy_bbt_root_offset: Option<u64>,
    legacy_nbt_root_offset: Option<u64>,
) -> PstdResult<Vec<PstRootCandidateDiagnostic>> {
    let mut candidates = Vec::new();
    if variant == PstVariant::Unicode {
        let unicode_bbt_root_offset = read_optional_offset(buf, UNICODE_BBT_ROOT_OFFSET_FIELD)?;
        let unicode_nbt_root_offset = read_optional_offset(buf, UNICODE_NBT_ROOT_OFFSET_FIELD)?;
        if unicode_bbt_root_offset.is_some() || unicode_nbt_root_offset.is_some() {
            candidates.push(PstRootCandidateDiagnostic::from_offsets(
                "unicode_root_bref_offsets",
                file_size,
                unicode_bbt_root_offset,
                unicode_nbt_root_offset,
            ));
        }
    }

    candidates.push(PstRootCandidateDiagnostic::from_offsets(
        "legacy_header_fields",
        file_size,
        legacy_bbt_root_offset,
        legacy_nbt_root_offset,
    ));
    Ok(candidates)
}

fn read_optional_offset(buf: &[u8], start: usize) -> PstdResult<Option<u64>> {
    if buf.len() < start + 8 {
        return Ok(None);
    }
    let value = u64_le_at(buf, start, 0)?;
    if value == 0 {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

pub fn validate_pst_magic(buf: &[u8]) -> PstdResult<()> {
    if buf.len() < 4 {
        return Err(PstdError::pst_parse(
            Some(0),
            "file too short to contain PST magic",
        ));
    }
    if buf[0..4] != PST_MAGIC {
        return Err(PstdError::pst_parse(Some(0), "missing PST magic !BDN"));
    }
    Ok(())
}

pub fn summarize_version(version: Option<u16>) -> String {
    match version {
        Some(23) | Some(36) => "unicode".to_string(),
        Some(14) | Some(15) => "ansi".to_string(),
        Some(value) => format!("unknown_version_{value}"),
        None => "unknown".to_string(),
    }
}
