use crate::error::{PstdError, PstdResult};
use crate::pst::binary::{u32_le_at, u64_le_at, u8_at};
use crate::pst::primitives::{BlockId, BlockRef, ByteOffset, PageRef};
use crate::pst::reader::PstByteReader;
use crate::pst::trailer::PageTrailer;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BbtEntry {
    pub block_id: BlockId,
    pub offset: ByteOffset,
    pub size: u64,
}

impl BbtEntry {
    pub fn block_ref(&self) -> BlockRef {
        BlockRef {
            block_id: self.block_id,
            offset: self.offset,
            size: self.size,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BbtPage {
    pub source_offset: u64,
    pub entry_count: u8,
    pub entries: Vec<BbtEntry>,
    pub trailer: Option<PageTrailer>,
}

impl BbtPage {
    pub fn parse(page: &[u8], source_offset: u64) -> PstdResult<Self> {
        if page.len() < PageTrailer::LEN + 4 {
            return Err(PstdError::pst_parse(Some(source_offset), "BBT page too short"));
        }

        let entry_count = u8_at(page, 0, source_offset)?;
        let entry_size = 24usize;
        let entries_start = 4usize;
        let data_end = page.len().saturating_sub(PageTrailer::LEN);
        let mut entries = Vec::new();

        for idx in 0..entry_count as usize {
            let start = entries_start + idx * entry_size;
            if start + entry_size > data_end {
                break;
            }
            let block_id = BlockId(u64_le_at(page, start, source_offset)?);
            let offset = ByteOffset(u64_le_at(page, start + 8, source_offset)?);
            let size = u32_le_at(page, start + 16, source_offset)? as u64;
            entries.push(BbtEntry { block_id, offset, size });
        }

        let trailer = PageTrailer::parse_from_page(page, source_offset).ok();
        Ok(Self { source_offset, entry_count, entries, trailer })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BbtIndex {
    pub root: Option<PageRef>,
    pub entries: Vec<BbtEntry>,
    pub status: String,
}

impl BbtIndex {
    pub fn load_root(reader: &PstByteReader, root: Option<PageRef>) -> PstdResult<Self> {
        let Some(root_ref) = root else {
            return Ok(Self {
                root,
                entries: Vec::new(),
                status: "root_unavailable".to_string(),
            });
        };

        if root_ref.offset.0 >= reader.file_size() {
            return Err(PstdError::pst_parse(
                Some(root_ref.offset.0),
                "BBT root offset is beyond file size",
            ));
        }

        let remaining = reader.file_size() - root_ref.offset.0;
        let page_size = remaining.min(512) as usize;
        let page = reader.read_at(root_ref.offset.0, page_size)?;
        let parsed = BbtPage::parse(&page, root_ref.offset.0)?;
        Ok(Self {
            root,
            entries: parsed.entries,
            status: "root_page_parsed".to_string(),
        })
    }

    pub fn lookup(&self, block_id: BlockId) -> Option<BlockRef> {
        self.entries
            .iter()
            .find(|entry| entry.block_id == block_id)
            .map(BbtEntry::block_ref)
    }
}
