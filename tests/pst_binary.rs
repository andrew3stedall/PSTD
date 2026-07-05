use std::fs;

use pstd::pst::bbt::BbtPage;
use pstd::pst::binary::{u16_le_at, u32_le_at, u64_le_at};
use pstd::pst::header::{PstHeader, PST_MAGIC};
use pstd::pst::nbt::NbtPage;
use pstd::pst::primitives::{BlockId, ByteOffset, NodeId};
use pstd::pst::reader::PstByteReader;
use pstd::pst::trailer::{BlockTrailer, PageTrailer};

#[test]
fn byte_reader_reads_bounded_ranges() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("sample.bin");
    fs::write(&path, b"abcdef").unwrap();
    let reader = PstByteReader::open(&path).unwrap();
    assert_eq!(reader.file_size(), 6);
    assert_eq!(reader.read_at(2, 3).unwrap(), b"cde");
    assert!(reader.read_at(5, 2).is_err());
}

#[test]
fn binary_helpers_read_little_endian_values() {
    let bytes = [0x34, 0x12, 0x78, 0x56, 0, 0, 0, 0];
    assert_eq!(u16_le_at(&bytes, 0, 0).unwrap(), 0x1234);
    assert_eq!(u32_le_at(&bytes, 0, 0).unwrap(), 0x56781234);
    assert_eq!(u64_le_at(&bytes, 0, 0).unwrap(), 0x0000000056781234);
    assert!(u64_le_at(&bytes, 2, 0).is_err());
}

#[test]
fn pst_header_accepts_minimal_unicode_header() {
    let mut header = vec![0u8; 64];
    header[0..4].copy_from_slice(&PST_MAGIC);
    header[8..10].copy_from_slice(b"SM");
    header[10..12].copy_from_slice(&23u16.to_le_bytes());
    let parsed = PstHeader::parse_bytes(&header, 64).unwrap();
    assert_eq!(parsed.summary.format, "pst");
    assert_eq!(parsed.summary.version, Some(23));
    assert_eq!(parsed.summary.variant, "unicode");
    assert_eq!(
        parsed.summary.root_diagnostics.condition,
        "root_pointers_absent"
    );
    assert_eq!(parsed.summary.root_diagnostics.candidate_count, 1);
    assert!(parsed.summary.root_diagnostics.selected_source.is_none());
    assert!(parsed.roots.bbt_root.is_none());
    assert!(parsed.roots.nbt_root.is_none());
}

#[test]
fn pst_header_classifies_root_offsets_beyond_file_size() {
    let mut header = vec![0u8; 64];
    header[0..4].copy_from_slice(&PST_MAGIC);
    header[8..10].copy_from_slice(b"SM");
    header[10..12].copy_from_slice(&23u16.to_le_bytes());
    header[48..56].copy_from_slice(&4_415_226_381_312u64.to_le_bytes());
    header[56..64].copy_from_slice(&281_500_746_530_816u64.to_le_bytes());

    let parsed = PstHeader::parse_bytes(&header, 271_360).unwrap();

    assert_eq!(
        parsed.summary.root_diagnostics.condition,
        "root_candidates_unusable"
    );
    assert_eq!(
        parsed.summary.root_diagnostics.candidates[0].condition,
        "root_offsets_out_of_bounds"
    );
    assert_eq!(
        parsed.summary.root_diagnostics.bbt_root.condition,
        "root_offset_beyond_file_size"
    );
    assert_eq!(
        parsed.summary.root_diagnostics.nbt_root.condition,
        "root_offset_beyond_file_size"
    );
    assert!(!parsed.summary.root_diagnostics.bbt_root.offset_in_bounds);
    assert!(!parsed.summary.root_diagnostics.nbt_root.root_page_in_bounds);
    assert!(parsed.summary.root_diagnostics.selected_source.is_none());
    assert!(parsed.roots.bbt_root.is_none());
    assert!(parsed.roots.nbt_root.is_none());
}

#[test]
fn pst_header_classifies_root_pages_in_bounds() {
    let mut header = vec![0u8; 64];
    header[0..4].copy_from_slice(&PST_MAGIC);
    header[8..10].copy_from_slice(b"SM");
    header[10..12].copy_from_slice(&23u16.to_le_bytes());
    header[48..56].copy_from_slice(&1024u64.to_le_bytes());
    header[56..64].copy_from_slice(&2048u64.to_le_bytes());

    let parsed = PstHeader::parse_bytes(&header, 4096).unwrap();

    assert_eq!(
        parsed.summary.root_diagnostics.condition,
        "root_pages_in_bounds"
    );
    assert_eq!(
        parsed.summary.root_diagnostics.selected_source.as_deref(),
        Some("legacy_header_fields")
    );
    assert_eq!(parsed.summary.bbt_root_offset, Some(2048));
    assert_eq!(parsed.summary.nbt_root_offset, Some(1024));
    assert_eq!(parsed.roots.bbt_root.unwrap().offset.0, 2048);
    assert_eq!(parsed.roots.nbt_root.unwrap().offset.0, 1024);
    assert!(parsed.summary.root_diagnostics.bbt_root.offset_in_bounds);
    assert!(parsed.summary.root_diagnostics.nbt_root.root_page_in_bounds);
}

#[test]
fn pst_header_selects_later_unicode_root_candidate_when_legacy_is_impossible() {
    let mut header = vec![0u8; 248];
    header[0..4].copy_from_slice(&PST_MAGIC);
    header[8..10].copy_from_slice(b"SM");
    header[10..12].copy_from_slice(&23u16.to_le_bytes());
    header[48..56].copy_from_slice(&4_415_226_381_312u64.to_le_bytes());
    header[56..64].copy_from_slice(&281_500_746_530_816u64.to_le_bytes());
    header[224..232].copy_from_slice(&1024u64.to_le_bytes());
    header[240..248].copy_from_slice(&2048u64.to_le_bytes());

    let parsed = PstHeader::parse_bytes(&header, 4096).unwrap();

    assert_eq!(
        parsed.summary.root_diagnostics.condition,
        "root_pages_in_bounds"
    );
    assert_eq!(
        parsed.summary.root_diagnostics.selected_source.as_deref(),
        Some("unicode_root_bref_offsets")
    );
    assert_eq!(parsed.summary.root_diagnostics.candidate_count, 2);
    assert_eq!(
        parsed.summary.root_diagnostics.candidates[0].source,
        "unicode_root_bref_offsets"
    );
    assert_eq!(
        parsed.summary.root_diagnostics.candidates[1].condition,
        "root_offsets_out_of_bounds"
    );
    assert_eq!(parsed.summary.bbt_root_offset, Some(2048));
    assert_eq!(parsed.summary.nbt_root_offset, Some(1024));
    assert_eq!(parsed.roots.bbt_root.unwrap().offset.0, 2048);
    assert_eq!(parsed.roots.nbt_root.unwrap().offset.0, 1024);
}

#[test]
fn pst_header_keeps_roots_unavailable_when_no_candidate_pair_is_usable() {
    let mut header = vec![0u8; 248];
    header[0..4].copy_from_slice(&PST_MAGIC);
    header[8..10].copy_from_slice(b"SM");
    header[10..12].copy_from_slice(&23u16.to_le_bytes());
    header[48..56].copy_from_slice(&4_415_226_381_312u64.to_le_bytes());
    header[56..64].copy_from_slice(&281_500_746_530_816u64.to_le_bytes());
    header[224..232].copy_from_slice(&512u64.to_le_bytes());
    header[240..248].copy_from_slice(&1024u64.to_le_bytes());

    let parsed = PstHeader::parse_bytes(&header, 1200).unwrap();

    assert_eq!(
        parsed.summary.root_diagnostics.condition,
        "root_pages_truncated"
    );
    assert!(parsed.summary.root_diagnostics.selected_source.is_none());
    assert!(parsed.roots.bbt_root.is_none());
    assert!(parsed.roots.nbt_root.is_none());
}

#[test]
fn bbt_leaf_page_reads_metadata_and_entries_from_page_body() {
    let mut page = vec![0u8; 512];
    page[0..8].copy_from_slice(&0x22u64.to_le_bytes());
    page[8..16].copy_from_slice(&0x1000u64.to_le_bytes());
    page[16..18].copy_from_slice(&123u16.to_le_bytes());
    write_bt_page_metadata(&mut page, 1, 20, 24, 0, 0x80);

    let parsed = BbtPage::parse(&page, 4096).unwrap();

    assert_eq!(parsed.entry_count, 1);
    assert_eq!(parsed.entry_size, 24);
    assert_eq!(parsed.page_level, Some(0));
    assert_eq!(parsed.entries.len(), 1);
    assert_eq!(parsed.entries[0].block_id.0, 0x22);
    assert_eq!(parsed.entries[0].offset.0, 0x1000);
    assert_eq!(parsed.entries[0].size, 123);
    assert!(parsed.child_page_refs.is_empty());
}

#[test]
fn bbt_internal_page_reads_child_offset_from_bref() {
    let mut page = vec![0u8; 512];
    page[0..8].copy_from_slice(&0x22u64.to_le_bytes());
    page[8..16].copy_from_slice(&0x33u64.to_le_bytes());
    page[16..24].copy_from_slice(&0x2000u64.to_le_bytes());
    write_bt_page_metadata(&mut page, 1, 20, 24, 1, 0x80);

    let parsed = BbtPage::parse(&page, 8192).unwrap();

    assert!(parsed.entries.is_empty());
    assert_eq!(parsed.child_page_refs.len(), 1);
    assert_eq!(parsed.child_page_refs[0].block_id.0, 0x33);
    assert_eq!(parsed.child_page_refs[0].offset.0, 0x2000);
}

#[test]
fn nbt_leaf_page_reads_metadata_and_entries_from_page_body() {
    let mut page = vec![0u8; 512];
    page[0..8].copy_from_slice(&0x61u64.to_le_bytes());
    page[8..16].copy_from_slice(&0x71u64.to_le_bytes());
    page[16..24].copy_from_slice(&0x81u64.to_le_bytes());
    write_bt_page_metadata(&mut page, 1, 15, 32, 0, 0x81);

    let parsed = NbtPage::parse(&page, 12288).unwrap();

    assert_eq!(parsed.entry_count, 1);
    assert_eq!(parsed.entry_size, 32);
    assert_eq!(parsed.page_level, Some(0));
    assert_eq!(parsed.entries.len(), 1);
    assert_eq!(parsed.entries[0].node_id.0, 0x61);
    assert_eq!(parsed.entries[0].data_block_id.0, 0x71);
    assert_eq!(parsed.entries[0].subnode_block_id.unwrap().0, 0x81);
    assert!(parsed.child_page_refs.is_empty());
}

#[test]
fn nbt_internal_page_reads_child_offset_from_bref() {
    let mut page = vec![0u8; 512];
    page[0..8].copy_from_slice(&0x61u64.to_le_bytes());
    page[8..16].copy_from_slice(&0x71u64.to_le_bytes());
    page[16..24].copy_from_slice(&0x3000u64.to_le_bytes());
    write_bt_page_metadata(&mut page, 1, 20, 24, 1, 0x81);

    let parsed = NbtPage::parse(&page, 16384).unwrap();

    assert!(parsed.entries.is_empty());
    assert_eq!(parsed.child_page_refs.len(), 1);
    assert_eq!(parsed.child_page_refs[0].node_id.0, 0x61);
    assert_eq!(parsed.child_page_refs[0].offset.0, 0x3000);
}

#[test]
fn typed_primitives_format_for_diagnostics() {
    assert_eq!(format!("{:?}", NodeId(0x22)), "NodeId(0x22)");
    assert_eq!(format!("{:?}", BlockId(0x33)), "BlockId(0x33)");
    assert_eq!(format!("{:?}", ByteOffset(44)), "ByteOffset(44)");
}

#[test]
fn trailers_parse_from_slices() {
    let page = vec![0u8; 512];
    let page_trailer = PageTrailer::parse_from_page(&page, 0).unwrap();
    assert_eq!(page_trailer.offset, 496);

    let block = vec![0u8; 64];
    let block_trailer = BlockTrailer::parse_from_block(&block, 100).unwrap();
    assert_eq!(block_trailer.offset.0, 148);
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
