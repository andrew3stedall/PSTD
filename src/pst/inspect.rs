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
    pub bbt_status: String,
    pub bbt_entries: usize,
    pub nbt_status: String,
    pub nbt_entries: usize,
}

impl InspectSummary {
    pub fn to_human_text(&self) -> String {
        format!(
            "PSTD inspect\ninput: {}\nfile_size: {}\nformat: {}\nvariant: {}\nversion: {:?}\nheader_status: {}\nbbt_status: {}\nbbt_entries: {}\nnbt_status: {}\nnbt_entries: {}",
            self.input_path,
            self.file_size,
            self.header.format,
            self.header.variant,
            self.header.version,
            self.header.parser_status,
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

    let bbt = BbtIndex::load_root(&reader, header.roots.bbt_root)?;
    let nbt = NbtIndex::load_root(&reader, header.roots.nbt_root)?;

    Ok(InspectSummary {
        input_path: reader.input_path().display().to_string(),
        file_size: reader.file_size(),
        header: header.summary,
        bbt_status: bbt.status,
        bbt_entries: bbt.entries.len(),
        nbt_status: nbt.status,
        nbt_entries: nbt.entries.len(),
    })
}
