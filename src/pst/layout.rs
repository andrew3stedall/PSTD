use crate::pst::binary::{u16_le_at, u8_at};
use crate::pst::primitives::PstVariant;
use crate::error::PstdResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PstLayout {
    pub variant: PstVariant,
    pub page_size: usize,
    pub entry_area_bytes: usize,
    pub metadata_offset: usize,
    pub count_width: usize,
    pub backlink_offset: usize,
    pub id_width: usize,
}

impl PstLayout {
    pub const fn for_variant(variant: PstVariant) -> Self {
        match variant {
            PstVariant::Ansi => Self {
                variant,
                page_size: 512,
                entry_area_bytes: 496,
                metadata_offset: 496,
                count_width: 1,
                backlink_offset: 504,
                id_width: 4,
            },
            PstVariant::Ost2013 => Self {
                variant,
                page_size: 4096,
                entry_area_bytes: 4056,
                metadata_offset: 4056,
                count_width: 2,
                backlink_offset: 4080,
                id_width: 8,
            },
            PstVariant::Unicode | PstVariant::Unknown => Self {
                variant,
                page_size: 512,
                entry_area_bytes: 488,
                metadata_offset: 488,
                count_width: 1,
                backlink_offset: 504,
                id_width: 8,
            },
        }
    }

    pub fn read_count(&self, page: &[u8], offset: usize, source_offset: u64) -> PstdResult<u16> {
        if self.count_width == 2 {
            u16_le_at(page, offset, source_offset)
        } else {
            Ok(u8_at(page, offset, source_offset)? as u16)
        }
    }

    pub fn read_level(&self, page: &[u8], offset: usize, source_offset: u64) -> PstdResult<u8> {
        u8_at(page, offset, source_offset)
    }

    pub fn has_unicode_page_trailer(&self) -> bool {
        matches!(self.variant, PstVariant::Unicode | PstVariant::Unknown)
    }
}
