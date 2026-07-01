use std::collections::HashSet;

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
    pub parsed_entry_count: usize,
    pub truncated_entry_count: usize,
    pub page_type: Option<u8>,
    pub page_level: Option<u8>,
    pub entries: Vec<NbtEntry>,
    pub trailer: Option<PageTrailer>,
    pub status: String,
}

impl NbtPage {
    pub fn parse(page: &[u8], source_offset: u64) -> PstdResult<Self> {
        if page.len() < PageTrailer::LEN + 4 {
            return Err(PstdError::pst_parse(
                Some(source_offset),
                "node page too short",
            ));
        }

        let entry_count = u8_at(page, 0, source_offset)?;
        let mut entries = Vec::new();
        let entry_size = 32usize;
        let entries_start = 4usize;
        let data_end = page.len().saturating_sub(PageTrailer::LEN);
        let capacity = data_end.saturating_sub(entries_start) / entry_size;
        let entries_to_parse = (entry_count as usize).min(capacity);
        let truncated_entry_count = (entry_count as usize).saturating_sub(entries_to_parse);

        for idx in 0..entries_to_parse {
            let start = entries_start + idx * entry_size;
            let node_id = NodeId(u64_le_at(page, start, source_offset)?);
            let data_block_id = BlockId(u64_le_at(page, start + 8, source_offset)?);
            let raw_subnode = u64_le_at(page, start + 16, source_offset)?;
            let subnode_block_id = if raw_subnode == 0 {
                None
            } else {
                Some(BlockId(raw_subnode))
            };
            entries.push(NbtEntry {
                node_id,
                data_block_id,
                subnode_block_id,
            });
        }

        let trailer = PageTrailer::parse_from_page(page, source_offset).ok();
        let page_type = trailer.as_ref().map(|value| value.page_type);
        let page_level = trailer.as_ref().map(|value| value.page_level);
        let status = if truncated_entry_count == 0 {
            "complete".to_string()
        } else {
            "truncated_entries".to_string()
        };

        Ok(Self {
            source_offset,
            entry_count,
            parsed_entry_count: entries.len(),
            truncated_entry_count,
            page_type,
            page_level,
            entries,
            trailer,
            status,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NbtIndex {
    pub root: Option<PageRef>,
    pub entries: Vec<NbtEntry>,
    pub parsed_pages: u64,
    pub duplicate_entry_count: u64,
    pub truncated_entry_count: u64,
    pub status: String,
}

impl NbtIndex {
    pub fn load_root(reader: &PstByteReader, root: Option<PageRef>) -> PstdResult<Self> {
        let Some(root_ref) = root else {
            return Ok(Self {
                root,
                entries: Vec::new(),
                parsed_pages: 0,
                duplicate_entry_count: 0,
                truncated_entry_count: 0,
                status: "root_unavailable".to_string(),
            });
        };
        if root_ref.offset.0 >= reader.file_size() {
            return Err(PstdError::pst_parse(
                Some(root_ref.offset.0),
                "node root offset beyond file size",
            ));
        }
        let page_size = (reader.file_size() - root_ref.offset.0).min(512) as usize;
        let page = reader.read_at(root_ref.offset.0, page_size)?;
        let parsed = NbtPage::parse(&page, root_ref.offset.0)?;
        let duplicate_entry_count = duplicate_node_count(&parsed.entries);
        let status = format!(
            "root_page_parsed; page_status={}; page_level={}; page_type={}; entries={}; truncated_entries={}; duplicate_entries={}",
            parsed.status,
            option_u8_status(parsed.page_level),
            option_u8_status(parsed.page_type),
            parsed.parsed_entry_count,
            parsed.truncated_entry_count,
            duplicate_entry_count
        );

        Ok(Self {
            root,
            entries: parsed.entries,
            parsed_pages: 1,
            duplicate_entry_count,
            truncated_entry_count: parsed.truncated_entry_count as u64,
            status,
        })
    }

    pub fn lookup(&self, node_id: NodeId) -> Option<&NbtEntry> {
        self.entries.iter().find(|entry| entry.node_id == node_id)
    }
}

fn duplicate_node_count(entries: &[NbtEntry]) -> u64 {
    let mut seen = HashSet::new();
    entries
        .iter()
        .filter(|entry| !seen.insert(entry.node_id))
        .count() as u64
}

fn option_u8_status(value: Option<u8>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
