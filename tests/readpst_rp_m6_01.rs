use pstd::pst::bbt::BbtPage;
use pstd::pst::layout::PstLayout;
use pstd::pst::nbt::NbtPage;
use pstd::pst::primitives::PstVariant;

fn write_count(page: &mut [u8], offset: usize, width: usize, value: u16) {
    if width == 2 {
        page[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    } else {
        page[offset] = value as u8;
    }
}

fn write_id(page: &mut [u8], offset: usize, width: usize, value: u64) {
    if width == 8 {
        page[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    } else {
        page[offset..offset + 4].copy_from_slice(&(value as u32).to_le_bytes());
    }
}

fn write_metadata(page: &mut [u8], variant: PstVariant, entry_size: u16) {
    let layout = PstLayout::for_variant(variant);
    write_count(page, layout.metadata_offset, layout.count_width, 1);
    write_count(
        page,
        layout.metadata_offset + layout.count_width,
        layout.count_width,
        1,
    );
    write_count(
        page,
        layout.metadata_offset + layout.count_width * 2,
        layout.count_width,
        entry_size,
    );
    page[layout.metadata_offset + layout.count_width * 3] = 0;
}

#[test]
fn ansi_32_bit_leaf_pages_decode_through_the_shared_layout() {
    let variant = PstVariant::Ansi;
    let layout = PstLayout::for_variant(variant);
    let mut bbt = vec![0u8; layout.page_size];
    write_id(&mut bbt, 0, layout.id_width, 0x22);
    write_id(&mut bbt, layout.id_width, layout.id_width, 0x1000);
    bbt[layout.id_width * 2..layout.id_width * 2 + 2].copy_from_slice(&123u16.to_le_bytes());
    write_metadata(&mut bbt, variant, 12);

    let parsed_bbt = BbtPage::parse_with_layout(&bbt, 1024, layout).expect("ANSI BBT page");
    assert_eq!(parsed_bbt.entries.len(), 1);
    assert_eq!(parsed_bbt.entries[0].block_id.0, 0x22);
    assert_eq!(parsed_bbt.entries[0].offset.0, 0x1000);
    assert_eq!(parsed_bbt.entries[0].size, 123);

    let mut nbt = vec![0u8; layout.page_size];
    write_id(&mut nbt, 0, layout.id_width, 0x61);
    write_id(&mut nbt, layout.id_width, layout.id_width, 0x71);
    write_id(&mut nbt, layout.id_width * 2, layout.id_width, 0x81);
    write_metadata(&mut nbt, variant, 12);

    let parsed_nbt = NbtPage::parse_with_layout(&nbt, 2048, layout).expect("ANSI NBT page");
    assert_eq!(parsed_nbt.entries.len(), 1);
    assert_eq!(parsed_nbt.entries[0].node_id.0, 0x61);
    assert_eq!(parsed_nbt.entries[0].data_block_id.0, 0x71);
    assert_eq!(parsed_nbt.entries[0].subnode_block_id.unwrap().0, 0x81);
}

#[test]
fn ost_2013_4k_leaf_pages_decode_with_wide_metadata_and_ids() {
    let variant = PstVariant::Ost2013;
    let layout = PstLayout::for_variant(variant);
    let mut bbt = vec![0u8; layout.page_size];
    write_id(&mut bbt, 0, layout.id_width, 0x22);
    write_id(&mut bbt, layout.id_width, layout.id_width, 0x10000);
    bbt[layout.id_width * 2..layout.id_width * 2 + 2].copy_from_slice(&321u16.to_le_bytes());
    write_metadata(&mut bbt, variant, 24);

    let parsed_bbt = BbtPage::parse_with_layout(&bbt, 4096, layout).expect("OST BBT page");
    assert_eq!(parsed_bbt.entries.len(), 1);
    assert_eq!(parsed_bbt.entries[0].block_id.0, 0x22);
    assert_eq!(parsed_bbt.entries[0].offset.0, 0x10000);
    assert_eq!(parsed_bbt.entries[0].size, 321);

    let mut nbt = vec![0u8; layout.page_size];
    write_id(&mut nbt, 0, layout.id_width, 0x61);
    write_id(&mut nbt, layout.id_width, layout.id_width, 0x71);
    write_id(&mut nbt, layout.id_width * 2, layout.id_width, 0x81);
    write_metadata(&mut nbt, variant, 32);

    let parsed_nbt = NbtPage::parse_with_layout(&nbt, 8192, layout).expect("OST NBT page");
    assert_eq!(parsed_nbt.entries.len(), 1);
    assert_eq!(parsed_nbt.entries[0].node_id.0, 0x61);
    assert_eq!(parsed_nbt.entries[0].data_block_id.0, 0x71);
    assert_eq!(parsed_nbt.entries[0].subnode_block_id.unwrap().0, 0x81);
}

#[test]
fn unsupported_short_variant_pages_fail_closed() {
    let ansi = PstLayout::for_variant(PstVariant::Ansi);
    assert!(BbtPage::parse_with_layout(&vec![0u8; 511], 1024, ansi).is_err());

    let ost = PstLayout::for_variant(PstVariant::Ost2013);
    assert!(NbtPage::parse_with_layout(&vec![0u8; 4095], 4096, ost).is_err());
}
