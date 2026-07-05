use std::path::Path;

use crate::error::PstdResult;
use crate::pst::bbt::{BbtIndex, BbtPageDiagnostic};
use crate::pst::header::PstHeader;
use crate::pst::nbt::{NbtIndex, NbtPageDiagnostic};
use crate::pst::reader::PstByteReader;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InspectSummary {
    pub input_path: String,
    pub file_size: u64,
    pub header: crate::pst::header::PstHeaderSummary,
    pub root_diagnostic_condition: String,
    pub bbt_status: String,
    pub bbt_entries: usize,
    pub bbt_page_diagnostics: Vec<BbtPageDiagnostic>,
    pub nbt_status: String,
    pub nbt_entries: usize,
    pub nbt_page_diagnostics: Vec<NbtPageDiagnostic>,
}

impl InspectSummary {
    pub fn to_human_text(&self) -> String {
        format!(
            "PSTD inspect\ninput: {}\nfile_size: {}\nformat: {}\nvariant: {}\nversion: {:?}\nheader_status: {}\nroot_diagnostic_condition: {}\nroot_selected_source: {:?}\nroot_candidate_count: {}\nbbt_status: {}\nbbt_entries: {}\nbbt_pages_diagnosed: {}\nnbt_status: {}\nnbt_entries: {}\nnbt_pages_diagnosed: {}",
            self.input_path,
            self.file_size,
            self.header.format,
            self.header.variant,
            self.header.version,
            self.header.parser_status,
            self.root_diagnostic_condition,
            self.header.root_diagnostics.selected_source,
            self.header.root_diagnostics.candidate_count,
            self.bbt_status,
            self.bbt_entries,
            self.bbt_page_diagnostics.len(),
            self.nbt_status,
            self.nbt_entries,
            self.nbt_page_diagnostics.len(),
        )
    }
}

pub fn inspect_pst(input: impl AsRef<Path>) -> PstdResult<InspectSummary> {
    let reader = PstByteReader::open(input.as_ref())?;
    let header = PstHeader::parse(&reader)?;
    let root_diagnostic_condition = header.summary.root_diagnostics.condition.clone();

    let (bbt_status, bbt_entries, bbt_page_diagnostics) =
        match BbtIndex::load_root_with_diagnostics(&reader, header.roots.bbt_root) {
            Ok((bbt, diagnostics)) => (bbt.status, bbt.entries.len(), diagnostics),
            Err(err) => (format!("unavailable: {err}"), 0, Vec::new()),
        };
    let (nbt_status, nbt_entries, nbt_page_diagnostics) =
        match NbtIndex::load_root(&reader, header.roots.nbt_root) {
            Ok(nbt) => (nbt.status, nbt.entries.len(), nbt.page_diagnostics),
            Err(err) => (format!("unavailable: {err}"), 0, Vec::new()),
        };

    Ok(InspectSummary {
        input_path: reader.input_path().display().to_string(),
        file_size: reader.file_size(),
        header: header.summary,
        root_diagnostic_condition,
        bbt_status,
        bbt_entries,
        bbt_page_diagnostics,
        nbt_status,
        nbt_entries,
        nbt_page_diagnostics,
    })
}
