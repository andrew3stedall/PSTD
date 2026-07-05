use std::collections::{HashSet, VecDeque};

use crate::error::{PstdError, PstdResult};
use crate::pst::binary::{u16_le_at, u32_le_at, u64_le_at, u8_at};
use crate::pst::limits::ParserLimits;
use crate::pst::primitives::{BlockId, BlockRef, ByteOffset, PageRef};
use crate::pst::reader::PstByteReader;
use crate::pst::trailer::PageTrailer;

const BT_PAGE_ENTRY_AREA_BYTES: usize = 488;
const BT_PAGE_ENTRY_COUNT_OFFSET: usize = 488;
const BT_PAGE_ENTRY_CAPACITY_OFFSET: usize = 489;
const BT_PAGE_ENTRY_SIZE_OFFSET: usize = 490;
const BT_PAGE_LEVEL_OFFSET: usize = 491;

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
pub struct BbtChildPageRef {
    pub block_id: BlockId,
    pub offset: ByteOffset,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BbtPageDiagnostic {
    pub source_offset: u64,
    pub entry_count: u8,
    pub entry_capacity: u8,
    pub entry_size: u8,
    pub parsed_entry_count: usize,
    pub truncated_entry_count: usize,
    pub page_type: Option<u8>,
    pub page_level: u8,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BbtPage {
    pub source_offset: u64,
    pub entry_count: u8,
    pub entry_capacity: u8,
    pub entry_size: u8,
    pub parsed_entry_count: usize,
    pub truncated_entry_count: usize,
    pub page_type: Option<u8>,
    pub page_level: Option<u8>,
    pub entries: Vec<BbtEntry>,
    pub child_page_refs: Vec<BbtChildPageRef>,
    pub trailer: Option<PageTrailer>,
    pub status: String,
}

impl BbtPage {
    pub fn parse(page: &[u8], source_offset: u64) -> PstdResult<Self> {
        if page.len() < PageTrailer::LEN + 4 {
            return Err(PstdError::pst_parse(
                Some(source_offset),
                "BBT page too short",
            ));
        }
        if page.len() < 512 {
            return Err(PstdError::pst_parse(
                Some(source_offset),
                "BBT page shorter than 512 bytes",
            ));
        }

        let entry_count = u8_at(page, BT_PAGE_ENTRY_COUNT_OFFSET, source_offset)?;
        let entry_capacity = u8_at(page, BT_PAGE_ENTRY_CAPACITY_OFFSET, source_offset)?;
        let entry_size = u8_at(page, BT_PAGE_ENTRY_SIZE_OFFSET, source_offset)?;
        let page_level = u8_at(page, BT_PAGE_LEVEL_OFFSET, source_offset)?;
        let trailer = PageTrailer::parse_from_page(page, source_offset).ok();
        let page_type = trailer.as_ref().map(|value| value.page_type);

        if entry_size == 0 {
            return Ok(Self::unsupported(
                source_offset,
                entry_count,
                entry_capacity,
                entry_size,
                page_type,
                page_level,
                trailer,
                "unsupported_zero_entry_size",
            ));
        }

        let capacity_by_size = BT_PAGE_ENTRY_AREA_BYTES / entry_size as usize;
        let entries_to_parse = (entry_count as usize)
            .min(entry_capacity as usize)
            .min(capacity_by_size);
        let truncated_entry_count = (entry_count as usize).saturating_sub(entries_to_parse);
        let mut entries = Vec::new();
        let mut child_page_refs = Vec::new();

        for idx in 0..entries_to_parse {
            let start = idx * entry_size as usize;
            if page_level > 0 {
                if entry_size < 24 {
                    return Ok(Self::unsupported(
                        source_offset,
                        entry_count,
                        entry_capacity,
                        entry_size,
                        page_type,
                        page_level,
                        trailer,
                        "unsupported_internal_entry_size",
                    ));
                }
                let block_id = BlockId(u64_le_at(page, start + 8, source_offset)?);
                let offset = ByteOffset(u64_le_at(page, start + 16, source_offset)?);
                child_page_refs.push(BbtChildPageRef { block_id, offset });
            } else {
                if entry_size < 20 {
                    return Ok(Self::unsupported(
                        source_offset,
                        entry_count,
                        entry_capacity,
                        entry_size,
                        page_type,
                        page_level,
                        trailer,
                        "unsupported_leaf_entry_size",
                    ));
                }
                let block_id = BlockId(u64_le_at(page, start, source_offset)?);
                let offset = ByteOffset(u64_le_at(page, start + 8, source_offset)?);
                let size = u16_le_at(page, start + 16, source_offset)? as u64;
                entries.push(BbtEntry {
                    block_id,
                    offset,
                    size,
                });
            }
        }

        let parsed_entry_count = if page_level > 0 {
            child_page_refs.len()
        } else {
            entries.len()
        };
        let status = match (truncated_entry_count == 0, page_level > 0) {
            (true, true) => "complete_internal".to_string(),
            (true, false) => "complete_leaf".to_string(),
            (false, true) => "truncated_internal_entries".to_string(),
            (false, false) => "truncated_leaf_entries".to_string(),
        };

        Ok(Self {
            source_offset,
            entry_count,
            entry_capacity,
            entry_size,
            parsed_entry_count,
            truncated_entry_count,
            page_type,
            page_level: Some(page_level),
            entries,
            child_page_refs,
            trailer,
            status,
        })
    }

    fn unsupported(
        source_offset: u64,
        entry_count: u8,
        entry_capacity: u8,
        entry_size: u8,
        page_type: Option<u8>,
        page_level: u8,
        trailer: Option<PageTrailer>,
        status: &str,
    ) -> Self {
        Self {
            source_offset,
            entry_count,
            entry_capacity,
            entry_size,
            parsed_entry_count: 0,
            truncated_entry_count: entry_count as usize,
            page_type,
            page_level: Some(page_level),
            entries: Vec::new(),
            child_page_refs: Vec::new(),
            trailer,
            status: status.to_string(),
        }
    }

    pub fn is_internal(&self) -> bool {
        self.page_level.unwrap_or(0) > 0
    }

    pub fn diagnostic(&self) -> BbtPageDiagnostic {
        BbtPageDiagnostic {
            source_offset: self.source_offset,
            entry_count: self.entry_count,
            entry_capacity: self.entry_capacity,
            entry_size: self.entry_size,
            parsed_entry_count: self.parsed_entry_count,
            truncated_entry_count: self.truncated_entry_count,
            page_type: self.page_type,
            page_level: self.page_level.unwrap_or(0),
            status: self.status.clone(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BbtIndex {
    pub root: Option<PageRef>,
    pub entries: Vec<BbtEntry>,
    pub parsed_pages: u64,
    pub discovered_child_pages: u64,
    pub traversal_error_count: u64,
    pub duplicate_entry_count: u64,
    pub truncated_entry_count: u64,
    pub page_diagnostics: Vec<BbtPageDiagnostic>,
    pub status: String,
}

impl BbtIndex {
    pub fn load_root(reader: &PstByteReader, root: Option<PageRef>) -> PstdResult<Self> {
        Self::load_root_with_limits(reader, root, ParserLimits::default())
    }

    pub fn load_root_with_limits(
        reader: &PstByteReader,
        root: Option<PageRef>,
        limits: ParserLimits,
    ) -> PstdResult<Self> {
        let Some(root_ref) = root else {
            return Ok(Self {
                root,
                entries: Vec::new(),
                parsed_pages: 0,
                discovered_child_pages: 0,
                traversal_error_count: 0,
                duplicate_entry_count: 0,
                truncated_entry_count: 0,
                page_diagnostics: Vec::new(),
                status: "root_unavailable".to_string(),
            });
        };

        if root_ref.offset.0 >= reader.file_size() {
            return Err(PstdError::pst_parse(
                Some(root_ref.offset.0),
                "BBT root offset is beyond file size",
            ));
        }

        let mut entries = Vec::new();
        let mut parsed_pages = 0u64;
        let mut discovered_child_pages = 0u64;
        let mut traversal_error_count = 0u64;
        let mut truncated_entry_count = 0u64;
        let mut page_diagnostics = Vec::new();
        let mut seen_offsets = HashSet::new();
        let mut queue = VecDeque::from([root_ref]);

        while let Some(page_ref) = queue.pop_front() {
            if parsed_pages as usize >= limits.max_btree_pages {
                traversal_error_count += 1;
                break;
            }
            if !seen_offsets.insert(page_ref.offset.0) {
                continue;
            }
            if page_ref.offset.0 >= reader.file_size() {
                traversal_error_count += 1;
                continue;
            }

            let remaining = reader.file_size() - page_ref.offset.0;
            let page_size = remaining.min(512) as usize;
            let page = reader.read_at(page_ref.offset.0, page_size)?;
            let parsed = match BbtPage::parse(&page, page_ref.offset.0) {
                Ok(parsed) => parsed,
                Err(error) if parsed_pages == 0 => return Err(error),
                Err(_) => {
                    traversal_error_count += 1;
                    continue;
                }
            };

            parsed_pages += 1;
            truncated_entry_count += parsed.truncated_entry_count as u64;
            page_diagnostics.push(parsed.diagnostic());

            if parsed.is_internal() {
                for child in parsed.child_page_refs {
                    discovered_child_pages += 1;
                    if child.offset.0 < reader.file_size()
                        && !seen_offsets.contains(&child.offset.0)
                    {
                        queue.push_back(PageRef {
                            offset: child.offset,
                        });
                    } else {
                        traversal_error_count += 1;
                    }
                }
            } else {
                entries.extend(parsed.entries);
            }
        }

        let duplicate_entry_count = duplicate_block_count(&entries);
        let status = format!(
            "tree_traversed; parsed_pages={}; discovered_child_pages={}; entries={}; truncated_entries={}; duplicate_entries={}; traversal_errors={}; max_pages={}",
            parsed_pages,
            discovered_child_pages,
            entries.len(),
            truncated_entry_count,
            duplicate_entry_count,
            traversal_error_count,
            limits.max_btree_pages
        );

        Ok(Self {
            root,
            entries,
            parsed_pages,
            discovered_child_pages,
            traversal_error_count,
            duplicate_entry_count,
            truncated_entry_count,
            page_diagnostics,
            status,
        })
    }

    pub fn lookup(&self, block_id: BlockId) -> Option<BlockRef> {
        self.entries
            .iter()
            .find(|entry| entry.block_id == block_id)
            .map(BbtEntry::block_ref)
    }
}

fn duplicate_block_count(entries: &[BbtEntry]) -> u64 {
    let mut seen = HashSet::new();
    entries
        .iter()
        .filter(|entry| !seen.insert(entry.block_id))
        .count() as u64
}
