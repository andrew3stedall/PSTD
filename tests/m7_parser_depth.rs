use std::fs;

use pstd::pst::bbt::{BbtIndex, BbtPage};
use pstd::pst::nbt::{NbtIndex, NbtPage};
use pstd::pst::primitives::{ByteOffset, PageRef};
use pstd::pst::reader::PstByteReader;
use tempfile::NamedTempFile;

#[test]
fn bbt_page_reports_complete_parse_diagnostics() {
    let mut page = empty_page();
    write_u64(&mut page, 0, 0x100);
    write_u64(&mut page, 8, 1024);
    write_u16(&mut page, 16, 128);
    write_bt_page_metadata(&mut page, 1, 20, 24, 0, 0x80);

    let parsed = BbtPage::parse(&page, 4096).unwrap();
    assert_eq!(parsed.entry_count, 1);
    assert_eq!(parsed.entry_capacity, 20);
    assert_eq!(parsed.entry_size, 24);
    assert_eq!(parsed.parsed_entry_count, 1);
    assert_eq!(parsed.truncated_entry_count, 0);
    assert_eq!(parsed.page_type, Some(0x80));
    assert_eq!(parsed.page_level, Some(0));
    assert_eq!(parsed.status, "complete_leaf");
    assert!(parsed.child_page_refs.is_empty());
    assert_eq!(parsed.entries[0].block_id.0, 0x100);
    assert_eq!(parsed.entries[0].offset.0, 1024);
    assert_eq!(parsed.entries[0].size, 128);
}

#[test]
fn bbt_page_reports_truncated_entries() {
    let mut page = empty_page();
    write_bt_page_metadata(&mut page, 21, 20, 24, 0, 0x80);

    let parsed = BbtPage::parse(&page, 0).unwrap();
    assert_eq!(parsed.entry_count, 21);
    assert_eq!(parsed.parsed_entry_count, 20);
    assert_eq!(parsed.truncated_entry_count, 1);
    assert_eq!(parsed.status, "truncated_leaf_entries");
}

#[test]
fn bbt_index_traverses_internal_page_to_leaf_page() {
    let mut root = empty_page();
    write_u64(&mut root, 0, 0x10);
    write_u64(&mut root, 8, 0x99);
    write_u64(&mut root, 16, 512);
    write_bt_page_metadata(&mut root, 1, 20, 24, 1, 0x80);

    let mut leaf = empty_page();
    write_u64(&mut leaf, 0, 0x20);
    write_u64(&mut leaf, 8, 2048);
    write_u16(&mut leaf, 16, 256);
    write_bt_page_metadata(&mut leaf, 1, 20, 24, 0, 0x80);

    let reader = reader_with_pages([root, leaf].concat());
    let index = BbtIndex::load_root(
        &reader,
        Some(PageRef {
            offset: ByteOffset(0),
        }),
    )
    .unwrap();

    assert_eq!(index.parsed_pages, 2);
    assert_eq!(index.discovered_child_pages, 1);
    assert_eq!(index.traversal_error_count, 0);
    assert_eq!(index.entries.len(), 1);
    assert_eq!(index.entries[0].block_id.0, 0x20);
    assert!(index.status.contains("tree_traversed"));
}

#[test]
fn nbt_page_reports_complete_parse_diagnostics() {
    let mut page = empty_page();
    write_u64(&mut page, 0, 0x200);
    write_u64(&mut page, 8, 0x300);
    write_u64(&mut page, 16, 0x400);
    write_bt_page_metadata(&mut page, 1, 15, 32, 0, 0x81);

    let parsed = NbtPage::parse(&page, 8192).unwrap();
    assert_eq!(parsed.entry_count, 1);
    assert_eq!(parsed.entry_capacity, 15);
    assert_eq!(parsed.entry_size, 32);
    assert_eq!(parsed.parsed_entry_count, 1);
    assert_eq!(parsed.truncated_entry_count, 0);
    assert_eq!(parsed.page_type, Some(0x81));
    assert_eq!(parsed.page_level, Some(0));
    assert_eq!(parsed.status, "complete_leaf");
    assert!(parsed.child_page_refs.is_empty());
    assert_eq!(parsed.entries[0].node_id.0, 0x200);
    assert_eq!(parsed.entries[0].data_block_id.0, 0x300);
    assert_eq!(parsed.entries[0].subnode_block_id.unwrap().0, 0x400);
}

#[test]
fn nbt_page_reports_truncated_entries() {
    let mut page = empty_page();
    write_bt_page_metadata(&mut page, 16, 15, 32, 0, 0x81);

    let parsed = NbtPage::parse(&page, 0).unwrap();
    assert_eq!(parsed.entry_count, 16);
    assert_eq!(parsed.parsed_entry_count, 15);
    assert_eq!(parsed.truncated_entry_count, 1);
    assert_eq!(parsed.status, "truncated_leaf_entries");
}

#[test]
fn nbt_index_traverses_internal_page_to_leaf_page() {
    let mut root = empty_page();
    write_u64(&mut root, 0, 0x10);
    write_u64(&mut root, 8, 0x99);
    write_u64(&mut root, 16, 512);
    write_bt_page_metadata(&mut root, 1, 20, 24, 1, 0x81);

    let mut leaf = empty_page();
    write_u64(&mut leaf, 0, 0x20);
    write_u64(&mut leaf, 8, 0x30);
    write_u64(&mut leaf, 16, 0x40);
    write_bt_page_metadata(&mut leaf, 1, 15, 32, 0, 0x81);

    let reader = reader_with_pages([root, leaf].concat());
    let index = NbtIndex::load_root(
        &reader,
        Some(PageRef {
            offset: ByteOffset(0),
        }),
    )
    .unwrap();

    assert_eq!(index.parsed_pages, 2);
    assert_eq!(index.discovered_child_pages, 1);
    assert_eq!(index.traversal_error_count, 0);
    assert_eq!(index.entries.len(), 1);
    assert_eq!(index.entries[0].node_id.0, 0x20);
    assert_eq!(index.page_diagnostics.len(), 2);
    assert!(index.status.contains("tree_traversed"));
}

/// Builds a BBT with `leaf_count` single-entry leaf pages and as many 20-way internal
/// levels as needed. Returns the file bytes and the root page offset.
fn deep_bbt(leaf_count: usize) -> (Vec<u8>, u64) {
    const FANOUT: usize = 20;
    let mut pages: Vec<Vec<u8>> = Vec::new();
    // (first key, page index) for every page of the level being linked.
    let mut level: Vec<(u64, usize)> = Vec::with_capacity(leaf_count);
    for leaf in 0..leaf_count {
        let mut page = empty_page();
        write_u64(&mut page, 0, leaf as u64 * 4);
        write_u64(&mut page, 8, 4096 + leaf as u64);
        write_u16(&mut page, 16, 64);
        write_bt_page_metadata(&mut page, 1, 20, 24, 0, 0x80);
        level.push((leaf as u64 * 4, pages.len()));
        pages.push(page);
    }
    let mut page_level = 1u8;
    while level.len() > 1 {
        let mut parents = Vec::new();
        for children in level.chunks(FANOUT) {
            let mut page = empty_page();
            for (slot, (key, child_index)) in children.iter().enumerate() {
                let at = slot * 24;
                write_u64(&mut page, at, *key);
                write_u64(&mut page, at + 8, 0x99);
                write_u64(&mut page, at + 16, (*child_index * 512) as u64);
            }
            write_bt_page_metadata(&mut page, children.len() as u8, 20, 24, page_level, 0x80);
            parents.push((children[0].0, pages.len()));
            pages.push(page);
        }
        level = parents;
        page_level += 1;
    }
    let root_offset = (level[0].1 * 512) as u64;
    (pages.concat(), root_offset)
}

#[test]
fn bbt_index_traverses_more_than_the_former_1024_page_limit() {
    // 1,200 leaves + 60 + 3 + 1 internal pages = 1,264 pages, which the former
    // 1,024-page default truncated.
    let (bytes, root_offset) = deep_bbt(1200);
    let reader = reader_with_pages(bytes);
    let index = BbtIndex::load_root(
        &reader,
        Some(PageRef {
            offset: ByteOffset(root_offset),
        }),
    )
    .unwrap();

    assert_eq!(index.parsed_pages, 1264);
    assert_eq!(index.traversal_error_count, 0);
    assert_eq!(index.entries.len(), 1200);
    assert_eq!(index.duplicate_entry_count, 0);
    for leaf in [0u64, 599, 1199] {
        let block = index
            .lookup(pstd::pst::primitives::BlockId(leaf * 4))
            .unwrap();
        assert_eq!(block.offset.0, 4096 + leaf);
    }
}

#[test]
fn bbt_index_reports_a_reached_page_limit_as_a_traversal_error() {
    let (bytes, root_offset) = deep_bbt(1200);
    let reader = reader_with_pages(bytes);
    let limits = pstd::pst::limits::ParserLimits {
        max_btree_pages: 100,
        ..Default::default()
    };
    let index = BbtIndex::load_root_with_limits(
        &reader,
        Some(PageRef {
            offset: ByteOffset(root_offset),
        }),
        limits,
    )
    .unwrap();

    assert_eq!(index.parsed_pages, 100);
    assert!(index.traversal_error_count > 0);
    // Breadth-first: 64 internal pages are parsed before the first leaf.
    assert_eq!(index.entries.len(), 100 - 64);
    assert!(index.status.contains("max_pages=100"));
}

fn empty_page() -> Vec<u8> {
    vec![0; 512]
}

fn reader_with_pages(bytes: Vec<u8>) -> PstByteReader {
    let file = NamedTempFile::new().unwrap();
    fs::write(file.path(), bytes).unwrap();
    PstByteReader::open(file.path()).unwrap()
}

fn write_bt_page_metadata(
    page: &mut [u8],
    entry_count: u8,
    entry_capacity: u8,
    entry_size: u8,
    page_level: u8,
    page_type: u8,
) {
    page[488] = entry_count;
    page[489] = entry_capacity;
    page[490] = entry_size;
    page[491] = page_level;
    page[496] = page_type;
}

fn write_u16(page: &mut [u8], offset: usize, value: u16) {
    page[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u64(page: &mut [u8], offset: usize, value: u64) {
    page[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
