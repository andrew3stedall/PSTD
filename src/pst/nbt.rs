use crate::error::{PstdError, PstdResult};
use crate::pst::binary::{u64_le_at, u8_at};
use crate::pst::primitives::{BlockId, NodeId, PageRef};
use crate::pst::reader::PstByteReader;
use crate::pst::trailer::PageTrailer;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NbtEntry {
    pub node_id: NodeId,
    pub data_block_id: BlockId,
    pub subnode_block_id: Option<BlockId>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NbtPage {
    pub source_offset: u64,
    pub entry_count: u8,
    pub entries: Vec<NbtEntry>,
    pub trailer: Option<PageTrailer>,
}

impl NbtPage {
    pub fn parse(page: &[u8], source_offset: u64) -> PstdResult<Self> {
        if page.len() < PageTrailer::LEN + 4 {
            return Err(PstdError::pst_parse(Some(source_offset), "node page too short"));
        }

        let entry_count = u8_at(page, 0, source_offset)?;
        let mut entries = Vec::new();
        let entry_size = 32usize;
        let data_end = page.len().saturating_sub(PageTrailer::LEN);

        for idx in 0..entry_count as usize {
            let start = 4 + idx * entry_size;
            if start + entry_size > data_end {
                break;
            }
            let node_id = NodeId(u64_le_at(page, start, source_offset)?);
            let data_block_id = BlockId(u64_le_at(page, start + 8, source_offset)?);
            let raw_subnode = u64_le_at(page, start + 16, source_offset)?;
            let subnode_block_id = if raw_subnode == 0 { None } else { Some(BlockId(raw_subnode)) };
            entries.push(NbtEntry { node_id, data_block_id, subnode_block_id });
        }

        Ok(Self {
            source_offset,
            entry_count,
            entries,
            trailer: PageTrailer::parse_from_page(page, source_offset).ok(),
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NbtIndex {
    pub root: Option<PageRef>,
    pub entries: Vec<NbtEntry>,
    pub status: String,
}

impl NbtIndex {
    pub fn load_root(reader: &PstByteReader, root: Option<PageRef>) -> PstdResult<Self> {
        let Some(root_ref) = root else {
            return Ok(Self { root, entries: Vec::new(), status: "root_unavailable".to_string() });
        };
        if root_ref.offset.0 >= reader.file_size() {
            return Err(PstdError::pst_parse(Some(root_ref.offset.0), "node root offset beyond file size"));
        }
        let page_size = (reader.file_size() - root_ref.offset.0).min(512) as usize;
        let page = reader.read_at(root_ref.offset.0, page_size)?;
        let parsed = NbtPage::parse(&page, root_ref.offset.0)?;
        Ok(Self { root, entries: parsed.entries, status: "root_page_parsed".to_string() })
    }

    pub fn lookup(&self, node_id: NodeId) -> Option<&NbtEntry> {
        self.entries.iter().find(|entry| entry.node_id == node_id)
    }
}
