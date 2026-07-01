use pstd::pst::bbt::BbtPage;
use pstd::pst::nbt::NbtPage;

#[test]
fn bbt_page_reports_complete_parse_diagnostics() {
    let mut page = empty_page();
    page[0] = 1;
    write_u64(&mut page, 4, 0x100);
    write_u64(&mut page, 12, 1024);
    write_u32(&mut page, 20, 128);
    write_trailer(&mut page, 0x80, 0);

    let parsed = BbtPage::parse(&page, 4096).unwrap();
    assert_eq!(parsed.entry_count, 1);
    assert_eq!(parsed.parsed_entry_count, 1);
    assert_eq!(parsed.truncated_entry_count, 0);
    assert_eq!(parsed.page_type, Some(0x80));
    assert_eq!(parsed.page_level, Some(0));
    assert_eq!(parsed.status, "complete");
    assert_eq!(parsed.entries[0].block_id.0, 0x100);
    assert_eq!(parsed.entries[0].offset.0, 1024);
    assert_eq!(parsed.entries[0].size, 128);
}

#[test]
fn bbt_page_reports_truncated_entries() {
    let mut page = empty_page();
    page[0] = 21;
    write_trailer(&mut page, 0x80, 0);

    let parsed = BbtPage::parse(&page, 0).unwrap();
    assert_eq!(parsed.entry_count, 21);
    assert_eq!(parsed.parsed_entry_count, 20);
    assert_eq!(parsed.truncated_entry_count, 1);
    assert_eq!(parsed.status, "truncated_entries");
}

#[test]
fn nbt_page_reports_complete_parse_diagnostics() {
    let mut page = empty_page();
    page[0] = 1;
    write_u64(&mut page, 4, 0x200);
    write_u64(&mut page, 12, 0x300);
    write_u64(&mut page, 20, 0x400);
    write_trailer(&mut page, 0x81, 0);

    let parsed = NbtPage::parse(&page, 8192).unwrap();
    assert_eq!(parsed.entry_count, 1);
    assert_eq!(parsed.parsed_entry_count, 1);
    assert_eq!(parsed.truncated_entry_count, 0);
    assert_eq!(parsed.page_type, Some(0x81));
    assert_eq!(parsed.page_level, Some(0));
    assert_eq!(parsed.status, "complete");
    assert_eq!(parsed.entries[0].node_id.0, 0x200);
    assert_eq!(parsed.entries[0].data_block_id.0, 0x300);
    assert_eq!(parsed.entries[0].subnode_block_id.unwrap().0, 0x400);
}

#[test]
fn nbt_page_reports_truncated_entries() {
    let mut page = empty_page();
    page[0] = 16;
    write_trailer(&mut page, 0x81, 0);

    let parsed = NbtPage::parse(&page, 0).unwrap();
    assert_eq!(parsed.entry_count, 16);
    assert_eq!(parsed.parsed_entry_count, 15);
    assert_eq!(parsed.truncated_entry_count, 1);
    assert_eq!(parsed.status, "truncated_entries");
}

fn empty_page() -> Vec<u8> {
    vec![0; 512]
}

fn write_trailer(page: &mut [u8], page_type: u8, page_level: u8) {
    let start = page.len() - 16;
    page[start] = page_type;
    page[start + 1] = page_level;
}

fn write_u32(page: &mut [u8], offset: usize, value: u32) {
    page[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64(page: &mut [u8], offset: usize, value: u64) {
    page[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
