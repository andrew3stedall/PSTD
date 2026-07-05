use std::path::Path;

use crate::error::PstdResult;
use crate::pst::bbt::BbtIndex;
use crate::pst::header::PstHeader;
use crate::pst::nbt::NbtIndex;
use crate::pst::reader::PstByteReader;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InspectSummary {
    pub input_path: String,
    pub file_size: u64,
    pub header: crate::pst::header::PstHeaderSummary,
    pub root_diagnostic_condition: String,
    pub bbt_status: String,
    pub bbt_entries: usize,
    pub nbt_status: String,
    pub nbt_entries: usize,
}

impl InspectSummary {
    pub fn to_human_text(&self) -> String {
        format!(
            "PSTD inspect\ninput: {}\nfile_size: {}\nformat: {}\nvariant: {}\nversion: {:?}\nheader_status: {}\nroot_diagnostic_condition: {}\nroot_selected_source: {:?}\nroot_candidate_count: {}\nbbt_status: {}\nbbt_entries: {}\nnbt_status: {}\nnbt_entries: {}",
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
            self.nbt_status,
            self.nbt_entries,
        )
    }
}

pub fn inspect_pst(input: impl AsRef<Path>) -> PstdResult<InspectSummary> {
    let reader = PstByteReader::open(input.as_ref())?;
    let header = PstHeader::parse(&reader)?;
    let root_diagnostic_condition = header.summary.root_diagnostics.condition.clone();

    let (bbt_status, bbt_entries) = match BbtIndex::load_root(&reader, header.roots.bbt_root) {
        Ok(bbt) => (bbt.status, bbt.entries.len()),
        Err(err) => (format!("unavailable: {err}"), 0),
    };
    let (nbt_status, nbt_entries) = match NbtIndex::load_root(&reader, header.roots.nbt_root) {
        Ok(nbt) => (nbt.status, nbt.entries.len()),
        Err(err) => (format!("unavailable: {err}"), 0),
    };

    Ok(InspectSummary {
        input_path: reader.input_path().display().to_string(),
        file_size: reader.file_size(),
        header: header.summary,
        root_diagnostic_condition,
        bbt_status,
        bbt_entries,
        nbt_status,
        nbt_entries,
    })
}
