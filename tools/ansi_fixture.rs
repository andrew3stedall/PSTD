use std::{env, fs, io, path::Path};

const PAGE_SIZE: usize = 512;
const BBT_OFFSET: usize = 1024;
const NBT_OFFSET: usize = 1536;
const FILE_SIZE: usize = NBT_OFFSET + PAGE_SIZE;
/// HNHDR: ibHnpm, bSig, bClientSig, hidUserRoot and rgbFillLevel.
const HN_HEADER_SIZE: u16 = 12;
/// Data blocks use leaf BIDs (bit 0x2 clear); only the SLBLOCK is internal.
const FOLDER_BID: u32 = 0x100;
const MESSAGE_BID: u32 = 0x104;
const CONTENTS_TABLE_BID: u32 = 0x108;
const SLBLOCK_BID: u32 = 0x10e;
const RECIPIENT_TABLE_BID: u32 = 0x110;
const ATTACHMENT_BID: u32 = 0x114;
const ATTACHMENT_DATA_BID: u32 = 0x118;
const INDIRECT_ATTACHMENT_DATA_NID: u32 = 0x311;
const EXTERNAL_ATTACHMENT_DATA_ID2: u32 = 0x31f;
/// MS-PST fixed subnode NID of a message's recipient table.
const NID_RECIPIENT_TABLE: u32 = 0x692;
/// Subnode holding the stage-C attachment Property Context. Not an
/// NID_TYPE_ATTACHMENT NID reached through an attachment table (#642).
const ATTACHMENT_SUBNODE_NID: u32 = 0x31;

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn crc32(bytes: &[u8]) -> u32 {
    // MS-PST and libpff use the weak CRC-32 form for header and page fields:
    // reflected polynomial, initial value zero, with no final complement.
    let mut crc = 0u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    crc
}

fn block_signature(file_offset: u32, bid: u32) -> u16 {
    let value = file_offset ^ bid;
    ((value >> 16) as u16) ^ value as u16
}

fn make_page(file_offset: usize, ptype: u8, bid: u32) -> [u8; PAGE_SIZE] {
    let mut page = [0u8; PAGE_SIZE];

    // ANSI BBT/NBT pages keep their compact metadata immediately before
    // the 12-byte ANSI page trailer.
    page[496] = 0; // cEnt
    page[497] = 0; // cEntMax
    page[498] = 12; // cbEnt
    page[499] = 0; // cLevel

    page[500] = ptype;
    page[501] = ptype;
    put_u16(
        &mut page,
        502,
        block_signature(file_offset as u32, bid),
    );
    put_u32(&mut page, 504, bid);
    let page_crc = crc32(&page[..500]);
    put_u32(&mut page, 508, page_crc);
    page
}

fn make_fixture(crypt_method: u8) -> Vec<u8> {
    let mut bytes = vec![0u8; FILE_SIZE];

    bytes[0..4].copy_from_slice(b"!BDN");
    bytes[8..10].copy_from_slice(b"SM");
    put_u16(&mut bytes, 10, 14);
    put_u16(&mut bytes, 12, 19);
    bytes[14] = 1; // bPlatformCreate
    bytes[15] = 1; // bPlatformAccess

    // ANSI ROOT starts at offset 164. The parser consumes the two ANSI
    // BREF IB fields at offsets 188 and 196.
    put_u32(&mut bytes, 168, FILE_SIZE as u32);
    put_u32(&mut bytes, 184, 0x61); // NBT page BID
    put_u32(&mut bytes, 188, NBT_OFFSET as u32);
    put_u32(&mut bytes, 192, 0x22); // BBT page BID
    put_u32(&mut bytes, 196, BBT_OFFSET as u32);
    bytes[200] = 2; // VALID_AMAP2
    bytes[204..460].fill(0xff);
    bytes[460] = 0x80; // bSentinel
    bytes[461] = crypt_method;
    // ANSI reserved bytes remain zero.

    let bbt = make_page(BBT_OFFSET, 0x80, 0x22);
    let nbt = make_page(NBT_OFFSET, 0x81, 0x61);
    bytes[BBT_OFFSET..BBT_OFFSET + PAGE_SIZE].copy_from_slice(&bbt);
    bytes[NBT_OFFSET..NBT_OFFSET + PAGE_SIZE].copy_from_slice(&nbt);

    let header_crc = crc32(&bytes[8..479]);
    put_u32(&mut bytes, 4, header_crc);
    bytes
}

fn make_index_page(
    file_offset: usize,
    page_type: u8,
    page_bid: u32,
    entry_count: usize,
    entries: &[u8],
) -> [u8; PAGE_SIZE] {
    let mut page = [0u8; PAGE_SIZE];
    assert!(entry_count <= u8::MAX as usize);
    assert!(entries.len() <= 496);
    page[..entries.len()].copy_from_slice(entries);
    page[496] = entry_count as u8;
    page[497] = (496 / 12) as u8;
    page[498] = 12;
    page[499] = 0;
    page[500] = page_type;
    page[501] = page_type;
    put_u16(
        &mut page,
        502,
        block_signature(file_offset as u32, page_bid),
    );
    put_u32(&mut page, 504, page_bid);
    let page_crc = crc32(&page[..500]);
    put_u32(&mut page, 508, page_crc);
    page
}

fn make_folder_payload() -> Vec<u8> {
    make_property_context_heap(&[
        (0x3001, 0x001e, b"Synthetic Mail".to_vec()),
        (0x3602, 0x0003, 1i32.to_le_bytes().to_vec()),
    ])
}

fn transport_headers() -> Vec<u8> {
    b"Date: Tue, 01 Jan 2019 00:00:00 +0000\r\nTo: Recipient <recipient@example.test>\r\n".to_vec()
}

fn make_message_payload() -> Vec<u8> {
    make_property_context_heap(&[
        (0x001a, 0x001e, b"IPM.Note".to_vec()),
        (0x0037, 0x001e, b"ANSI Stage B synthetic message".to_vec()),
        (0x0c1a, 0x001e, b"PSTD Fixture Sender".to_vec()),
        (0x0c1f, 0x001e, b"sender@example.test".to_vec()),
        (0x0c1e, 0x001e, b"SMTP".to_vec()),
        (
            0x1000,
            0x001e,
            b"Hello from the deterministic ANSI Stage-B fixture.".to_vec(),
        ),
        (0x1035, 0x001e, b"<ansi-stage-b-001@example.test>".to_vec()),
        (0x007d, 0x001e, transport_headers()),
    ])
}

fn make_attachment_message_payload() -> Vec<u8> {
    make_property_context_heap(&[
        (0x001a, 0x001e, b"IPM.Note".to_vec()),
        (0x0037, 0x001e, b"ANSI Stage C attachment message".to_vec()),
        (0x0c1a, 0x001e, b"PSTD Fixture Sender".to_vec()),
        (0x0c1f, 0x001e, b"sender@example.test".to_vec()),
        (0x0c1e, 0x001e, b"SMTP".to_vec()),
        (0x0e1b, 0x000b, 1u32.to_le_bytes().to_vec()),
        (
            0x1000,
            0x001e,
            b"Hello from the deterministic ANSI Stage-C fixture.".to_vec(),
        ),
        (0x1035, 0x001e, b"<ansi-stage-c-001@example.test>".to_vec()),
        (0x007d, 0x001e, transport_headers()),
    ])
}

fn make_message_contents_table() -> Vec<u8> {
    let mut bth_header = vec![0xb5, 4, 4, 0];
    bth_header.extend_from_slice(&0x60u32.to_le_bytes());

    let mut tcinfo = vec![0u8; 30];
    tcinfo[0] = 0x7c;
    tcinfo[1] = 1;
    for (offset, boundary) in [(2, 4u16), (4, 4), (6, 4), (8, 5)] {
        tcinfo[offset..offset + 2].copy_from_slice(&boundary.to_le_bytes());
    }
    tcinfo[10..14].copy_from_slice(&0x20u32.to_le_bytes());
    tcinfo[14..18].copy_from_slice(&0u32.to_le_bytes());
    tcinfo[18..22].copy_from_slice(&0x20u32.to_le_bytes());
    tcinfo[22..26].copy_from_slice(&0x001a_001eu32.to_le_bytes());
    tcinfo[26..28].copy_from_slice(&0u16.to_le_bytes());
    tcinfo[28] = 4;
    tcinfo[29] = 0;

    let mut row_index = Vec::new();
    row_index.extend_from_slice(&0x24u32.to_le_bytes());
    row_index.extend_from_slice(&0u32.to_le_bytes());

    make_heap(vec![bth_header, tcinfo, row_index], 0x40)
}

fn make_recipient_table() -> Vec<u8> {
    let mut bth_header = vec![0xb5, 4, 4, 0];
    bth_header.extend_from_slice(&0x60u32.to_le_bytes());

    let mut tcinfo = vec![0u8; 46];
    tcinfo[0] = 0x7c;
    tcinfo[1] = 3;
    for (offset, boundary) in [(2, 12u16), (4, 12), (6, 12), (8, 13)] {
        tcinfo[offset..offset + 2].copy_from_slice(&boundary.to_le_bytes());
    }
    tcinfo[10..14].copy_from_slice(&0x20u32.to_le_bytes());
    tcinfo[14..18].copy_from_slice(&0x80u32.to_le_bytes());
    tcinfo[18..22].copy_from_slice(&0x20u32.to_le_bytes());
    for (index, (tag, data_offset, data_size)) in [
        (0x0c15_0003u32, 0u16, 4u8),
        (0x3001_001fu32, 4u16, 4u8),
        (0x39fe_001fu32, 8u16, 4u8),
    ]
    .into_iter()
    .enumerate()
    {
        let start = 22 + index * 8;
        tcinfo[start..start + 4].copy_from_slice(&tag.to_le_bytes());
        tcinfo[start + 4..start + 6].copy_from_slice(&data_offset.to_le_bytes());
        tcinfo[start + 6] = data_size;
        tcinfo[start + 7] = index as u8;
    }

    let mut row_index = Vec::new();
    row_index.extend_from_slice(&1u32.to_le_bytes());
    row_index.extend_from_slice(&0u32.to_le_bytes());

    let mut row = vec![0u8; 13];
    row[0..4].copy_from_slice(&1u32.to_le_bytes());
    row[4..8].copy_from_slice(&0xa0u32.to_le_bytes());
    row[8..12].copy_from_slice(&0xc0u32.to_le_bytes());
    // Cell existence bitmap: column iBit n is bit 0x80 >> n of byte n / 8.
    row[12] = 0b1110_0000;

    make_heap(vec![
        bth_header,
        tcinfo,
        row_index,
        row,
        utf16("Recipient"),
        utf16("recipient@example.test"),
    ], 0x40)
}

fn make_heap(allocations: Vec<Vec<u8>>, user_root: u32) -> Vec<u8> {
    let mut offsets = vec![HN_HEADER_SIZE];
    for allocation in &allocations {
        let next = usize::from(*offsets.last().expect("heap offset")) + allocation.len();
        offsets.push(u16::try_from(next).expect("ANSI heap fits in u16"));
    }
    let page_map_offset = *offsets.last().expect("heap page map offset");
    let page_map_len = 4 + offsets.len() * 2;
    let mut bytes = vec![0u8; usize::from(page_map_offset) + page_map_len];
    bytes[0..2].copy_from_slice(&page_map_offset.to_le_bytes());
    bytes[2] = 0xec;
    bytes[3] = 0x7c;
    bytes[4..8].copy_from_slice(&user_root.to_le_bytes());

    for (index, allocation) in allocations.iter().enumerate() {
        let start = usize::from(offsets[index]);
        bytes[start..start + allocation.len()].copy_from_slice(allocation);
    }

    let page_map = usize::from(page_map_offset);
    bytes[page_map..page_map + 2].copy_from_slice(&(allocations.len() as u16).to_le_bytes());
    for (index, offset) in offsets.iter().enumerate() {
        let start = page_map + 4 + index * 2;
        bytes[start..start + 2].copy_from_slice(&offset.to_le_bytes());
    }
    bytes
}

fn utf16(value: &str) -> Vec<u8> {
    value
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}

/// ANSI SLBLOCK: btype, cLevel and cEnt with no padding, then 12-byte
/// SLENTRY records (nid, bidData, bidSub), each field 32 bits wide.
fn make_slblock(entries: &[(u32, u32, u32)]) -> Vec<u8> {
    // SLENTRY records are sorted by subnode NID.
    assert!(entries.windows(2).all(|pair| pair[0].0 < pair[1].0));
    let mut bytes = vec![0u8; 4 + entries.len() * 12];
    bytes[0] = 0x02;
    put_u16(&mut bytes, 2, entries.len() as u16);
    for (index, (node_id, data_bid, subnode_bid)) in entries.iter().enumerate() {
        let start = 4 + index * 12;
        put_u32(&mut bytes, start, *node_id);
        put_u32(&mut bytes, start + 4, *data_bid);
        put_u32(&mut bytes, start + 8, *subnode_bid);
    }
    bytes
}

/// Unicode-width SLBLOCK (8-byte header, 24-byte SLENTRY records) inside an
/// ANSI file. This is NOT spec-valid: stage C keeps it only because PSTD's
/// attachment discovery reads Unicode-width SLBLOCKs alone (#642). Switch
/// stage C to `make_slblock` when that is fixed.
fn make_legacy_unicode_width_slblock(entries: &[(u32, u32, u32)]) -> Vec<u8> {
    assert!(entries.windows(2).all(|pair| pair[0].0 < pair[1].0));
    let mut bytes = vec![0u8; 8 + entries.len() * 24];
    bytes[0] = 0x02;
    put_u16(&mut bytes, 2, entries.len() as u16);
    for (index, (node_id, data_bid, subnode_bid)) in entries.iter().enumerate() {
        let start = 8 + index * 24;
        bytes[start..start + 8].copy_from_slice(&u64::from(*node_id).to_le_bytes());
        bytes[start + 8..start + 16].copy_from_slice(&u64::from(*data_bid).to_le_bytes());
        bytes[start + 16..start + 24].copy_from_slice(&u64::from(*subnode_bid).to_le_bytes());
    }
    bytes
}

fn make_property_context_heap(properties: &[(u16, u16, Vec<u8>)]) -> Vec<u8> {
    let mut bth_header = vec![0xb5, 2, 6, 0];
    bth_header.extend_from_slice(&0x40u32.to_le_bytes());
    let mut leaf = Vec::with_capacity(properties.len() * 8);
    let mut allocations = vec![bth_header, Vec::new()];

    // BTH leaf records are sorted by key, the property ID.
    let mut properties = properties.iter().collect::<Vec<_>>();
    properties.sort_by_key(|(property_id, _, _)| *property_id);
    for window in properties.windows(2) {
        assert_ne!(window[0].0, window[1].0, "duplicate PC property ID");
    }

    for (property_id, property_type, value) in properties {
        // Fixed-size values up to four bytes, and the PtypObject HNID, are
        // stored inline in a PC.
        let value_hid = if matches!(
            *property_type,
            0x0002 | 0x0003 | 0x0004 | 0x000a | 0x000b | 0x000d
        ) {
            u32::from_le_bytes(
                value
                    .as_slice()
                    .try_into()
                    .expect("ANSI inline value or object HNID is four bytes"),
            )
        } else {
            let hid = u32::try_from((allocations.len() + 1) * 0x20)
                .expect("ANSI heap HID fits");
            allocations.push(value.clone());
            hid
        };
        leaf.extend_from_slice(&property_id.to_le_bytes());
        leaf.extend_from_slice(&property_type.to_le_bytes());
        leaf.extend_from_slice(&value_hid.to_le_bytes());
    }
    allocations[1] = leaf;
    let mut bytes = make_heap(allocations, 0x20);
    bytes[3] = 0xbc;
    bytes
}

fn attachment_payload_bytes() -> Vec<u8> {
    b"ANSI Stage-C arbitrary attachment payload\n".to_vec()
}

fn make_attachment_property_context(
    indirect: bool,
    method: i32,
    external_id2: bool,
) -> Vec<u8> {
    let payload = attachment_payload_bytes();
    let payload_len = payload.len();
    let (data_type, data_value) = if indirect {
        (
            0x000d,
            if external_id2 {
                EXTERNAL_ATTACHMENT_DATA_ID2.to_le_bytes().to_vec()
            } else {
                INDIRECT_ATTACHMENT_DATA_NID.to_le_bytes().to_vec()
            },
        )
    } else {
        (0x0102, payload)
    };
    make_property_context_heap(&[
        (0x3704, 0x001e, b"ansi-attachment.bin".to_vec()),
        (0x370e, 0x001e, b"application/octet-stream".to_vec()),
        (0x3705, 0x0003, method.to_le_bytes().to_vec()),
        (
            0x0e20,
            0x0003,
            (payload_len as i32).to_le_bytes().to_vec(),
        ),
        (0x3701, data_type, data_value),
    ])
}

fn make_stage_b_fixture() -> Vec<u8> {
    let block_specs = vec![
        (FOLDER_BID, make_folder_payload()),
        (MESSAGE_BID, make_message_payload()),
        (CONTENTS_TABLE_BID, make_message_contents_table()),
        (
            SLBLOCK_BID,
            make_slblock(&[(NID_RECIPIENT_TABLE, RECIPIENT_TABLE_BID, 0)]),
        ),
        (RECIPIENT_TABLE_BID, make_recipient_table()),
    ];
    let mut block_entries = Vec::new();
    let mut nbt_entries = Vec::new();
    let mut cursor = FILE_SIZE;
    let mut block_bytes = Vec::new();
    for (block_id, payload) in block_specs {
        let offset = cursor;
        cursor += payload.len();
        block_entries.push((block_id, offset as u32, payload.len() as u16));
        block_bytes.push((offset, payload));
        match block_id {
            FOLDER_BID => nbt_entries.push((0x22u32, block_id, 0u32)),
            MESSAGE_BID => nbt_entries.push((0x24u32, block_id, SLBLOCK_BID)),
            CONTENTS_TABLE_BID => nbt_entries.push((0x2eu32, block_id, 0u32)),
            _ => {}
        }
    }

    let mut bytes = vec![0u8; cursor];
    bytes[0..4].copy_from_slice(b"!BDN");
    bytes[8..10].copy_from_slice(b"SM");
    put_u16(&mut bytes, 10, 14);
    put_u16(&mut bytes, 12, 19);
    bytes[14] = 1;
    bytes[15] = 1;
    put_u32(&mut bytes, 168, cursor as u32);
    put_u32(&mut bytes, 184, 0x61);
    put_u32(&mut bytes, 188, NBT_OFFSET as u32);
    put_u32(&mut bytes, 192, 0x22);
    put_u32(&mut bytes, 196, BBT_OFFSET as u32);
    bytes[200] = 2;
    bytes[204..460].fill(0xff);
    bytes[460] = 0x80;
    bytes[461] = 0;

    let mut bbt_entry_bytes = vec![0u8; block_entries.len() * 12];
    for (index, (block_id, offset, size)) in block_entries.iter().enumerate() {
        let start = index * 12;
        put_u32(&mut bbt_entry_bytes, start, *block_id);
        put_u32(&mut bbt_entry_bytes, start + 4, *offset);
        put_u16(&mut bbt_entry_bytes, start + 8, *size);
    }
    let bbt = make_index_page(BBT_OFFSET, 0x80, 0x22, block_entries.len(), &bbt_entry_bytes);
    let mut nbt_entry_bytes = vec![0u8; nbt_entries.len() * 12];
    for (index, (node_id, data_block_id, subnode_block_id)) in nbt_entries.iter().enumerate() {
        let start = index * 12;
        put_u32(&mut nbt_entry_bytes, start, *node_id);
        put_u32(&mut nbt_entry_bytes, start + 4, *data_block_id);
        put_u32(&mut nbt_entry_bytes, start + 8, *subnode_block_id);
    }
    let nbt = make_index_page(NBT_OFFSET, 0x81, 0x61, nbt_entries.len(), &nbt_entry_bytes);
    bytes[BBT_OFFSET..BBT_OFFSET + PAGE_SIZE].copy_from_slice(&bbt);
    bytes[NBT_OFFSET..NBT_OFFSET + PAGE_SIZE].copy_from_slice(&nbt);
    for (offset, payload) in block_bytes {
        bytes[offset..offset + payload.len()].copy_from_slice(&payload);
    }
    let header_crc = crc32(&bytes[8..479]);
    put_u32(&mut bytes, 4, header_crc);
    bytes
}

fn make_stage_c_fixture(indirect: bool, method: i32, external_id2: bool) -> Vec<u8> {
    let mut slblock_entries = vec![
        (ATTACHMENT_SUBNODE_NID, ATTACHMENT_BID, 0),
        (NID_RECIPIENT_TABLE, RECIPIENT_TABLE_BID, 0),
    ];
    if indirect {
        let reference = if external_id2 {
            EXTERNAL_ATTACHMENT_DATA_ID2
        } else {
            INDIRECT_ATTACHMENT_DATA_NID
        };
        slblock_entries.push((reference, ATTACHMENT_DATA_BID, 0));
        slblock_entries.sort_by_key(|(node_id, _, _)| *node_id);
    }
    let mut block_specs = vec![
        (FOLDER_BID, make_folder_payload()),
        (MESSAGE_BID, make_attachment_message_payload()),
        (CONTENTS_TABLE_BID, make_message_contents_table()),
        (SLBLOCK_BID, make_legacy_unicode_width_slblock(&slblock_entries)),
        (RECIPIENT_TABLE_BID, make_recipient_table()),
        (
            ATTACHMENT_BID,
            make_attachment_property_context(indirect, method, external_id2),
        ),
    ];
    if indirect {
        block_specs.push((ATTACHMENT_DATA_BID, attachment_payload_bytes()));
    }
    let mut block_entries = Vec::new();
    let mut nbt_entries = Vec::new();
    let mut cursor = FILE_SIZE;
    let mut block_bytes = Vec::new();
    for (block_id, payload) in block_specs {
        let offset = cursor;
        cursor += payload.len();
        block_entries.push((block_id, offset as u32, payload.len() as u16));
        block_bytes.push((offset, payload));
        match block_id {
            FOLDER_BID => nbt_entries.push((0x22u32, block_id, 0u32)),
            MESSAGE_BID => nbt_entries.push((0x24u32, block_id, SLBLOCK_BID)),
            CONTENTS_TABLE_BID => nbt_entries.push((0x2eu32, block_id, 0u32)),
            _ => {}
        }
    }

    let mut bytes = vec![0u8; cursor];
    bytes[0..4].copy_from_slice(b"!BDN");
    bytes[8..10].copy_from_slice(b"SM");
    put_u16(&mut bytes, 10, 14);
    put_u16(&mut bytes, 12, 19);
    bytes[14] = 1;
    bytes[15] = 1;
    put_u32(&mut bytes, 168, cursor as u32);
    put_u32(&mut bytes, 184, 0x61);
    put_u32(&mut bytes, 188, NBT_OFFSET as u32);
    put_u32(&mut bytes, 192, 0x22);
    put_u32(&mut bytes, 196, BBT_OFFSET as u32);
    bytes[200] = 2;
    bytes[204..460].fill(0xff);
    bytes[460] = 0x80;
    bytes[461] = 0;

    let mut bbt_entry_bytes = vec![0u8; block_entries.len() * 12];
    for (index, (block_id, offset, size)) in block_entries.iter().enumerate() {
        let start = index * 12;
        put_u32(&mut bbt_entry_bytes, start, *block_id);
        put_u32(&mut bbt_entry_bytes, start + 4, *offset);
        put_u16(&mut bbt_entry_bytes, start + 8, *size);
    }
    let bbt = make_index_page(BBT_OFFSET, 0x80, 0x22, block_entries.len(), &bbt_entry_bytes);
    let mut nbt_entry_bytes = vec![0u8; nbt_entries.len() * 12];
    for (index, (node_id, data_block_id, subnode_block_id)) in nbt_entries.iter().enumerate() {
        let start = index * 12;
        put_u32(&mut nbt_entry_bytes, start, *node_id);
        put_u32(&mut nbt_entry_bytes, start + 4, *data_block_id);
        put_u32(&mut nbt_entry_bytes, start + 8, *subnode_block_id);
    }
    let nbt = make_index_page(NBT_OFFSET, 0x81, 0x61, nbt_entries.len(), &nbt_entry_bytes);
    bytes[BBT_OFFSET..BBT_OFFSET + PAGE_SIZE].copy_from_slice(&bbt);
    bytes[NBT_OFFSET..NBT_OFFSET + PAGE_SIZE].copy_from_slice(&nbt);
    for (offset, payload) in block_bytes {
        bytes[offset..offset + payload.len()].copy_from_slice(&payload);
    }
    let header_crc = crc32(&bytes[8..479]);
    put_u32(&mut bytes, 4, header_crc);
    bytes
}

fn main() -> io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        eprintln!("usage: ansi_fixture <output-path> [crypt-method] [--force]");
        eprintln!("       ansi_fixture --stage-b <output-path> [--force]");
        eprintln!("       ansi_fixture --stage-c-attachment <output-path> [--force]");
        eprintln!("       ansi_fixture --stage-c-indirect-attachment <output-path> [--force]");
        eprintln!("       ansi_fixture --stage-c-method-2-attachment <output-path> [--force]");
        eprintln!("       ansi_fixture --stage-c-method-3-attachment <output-path> [--force]");
        eprintln!("       ansi_fixture --stage-c-method-4-attachment <output-path> [--force]");
        eprintln!("       ansi_fixture --stage-c-external-id2-attachment <output-path> [--force]");
        std::process::exit(2);
    }

    let stage_b = args.get(1).is_some_and(|arg| arg == "--stage-b");
    let stage_c_attachment = args
        .get(1)
        .is_some_and(|arg| arg == "--stage-c-attachment");
    let stage_c_indirect_attachment = args
        .get(1)
        .is_some_and(|arg| arg == "--stage-c-indirect-attachment");
    let stage_c_method_2_attachment = args
        .get(1)
        .is_some_and(|arg| arg == "--stage-c-method-2-attachment");
    let stage_c_method_3_attachment = args
        .get(1)
        .is_some_and(|arg| arg == "--stage-c-method-3-attachment");
    let stage_c_method_4_attachment = args
        .get(1)
        .is_some_and(|arg| arg == "--stage-c-method-4-attachment");
    let stage_c_external_id2_attachment = args
        .get(1)
        .is_some_and(|arg| arg == "--stage-c-external-id2-attachment");
    let output_index = if stage_b
        || stage_c_attachment
        || stage_c_indirect_attachment
        || stage_c_method_2_attachment
        || stage_c_method_3_attachment
        || stage_c_method_4_attachment
        || stage_c_external_id2_attachment
    {
        2
    } else {
        1
    };
    if args.len() <= output_index || (stage_b && args.len() > 4) || (!stage_b && args.len() > 4) {
        eprintln!("usage: ansi_fixture <output-path> [crypt-method] [--force]");
        eprintln!("       ansi_fixture --stage-b <output-path> [--force]");
        eprintln!("       ansi_fixture --stage-c-attachment <output-path> [--force]");
        eprintln!("       ansi_fixture --stage-c-indirect-attachment <output-path> [--force]");
        eprintln!("       ansi_fixture --stage-c-method-2-attachment <output-path> [--force]");
        eprintln!("       ansi_fixture --stage-c-method-3-attachment <output-path> [--force]");
        eprintln!("       ansi_fixture --stage-c-method-4-attachment <output-path> [--force]");
        eprintln!("       ansi_fixture --stage-c-external-id2-attachment <output-path> [--force]");
        std::process::exit(2);
    }

    let mut crypt_method = 0u8;
    let mut crypt_method_set = false;
    let mut force = false;
    for argument in args.iter().skip(output_index + 1) {
        if argument == "--force" {
            force = true;
        } else if stage_b
            || stage_c_attachment
            || stage_c_indirect_attachment
            || stage_c_method_2_attachment
            || stage_c_method_3_attachment
            || stage_c_method_4_attachment
            || stage_c_external_id2_attachment {
            eprintln!("unexpected argument: {argument}");
            std::process::exit(2);
        } else if crypt_method_set {
            eprintln!("unexpected argument: {argument}");
            std::process::exit(2);
        } else {
            crypt_method = argument
                .parse::<u8>()
                .expect("crypt method must be an integer");
            crypt_method_set = true;
        }
    }

    if Path::new(&args[output_index]).exists() && !force {
        eprintln!(
            "refusing to overwrite existing fixture {}; pass --force to replace it",
            args[output_index]
        );
        std::process::exit(2);
    }

    let bytes = if stage_b {
        make_stage_b_fixture()
    } else if stage_c_attachment {
        make_stage_c_fixture(false, 1, false)
    } else if stage_c_indirect_attachment {
        make_stage_c_fixture(true, 1, false)
    } else if stage_c_method_2_attachment {
        make_stage_c_fixture(true, 2, false)
    } else if stage_c_method_3_attachment {
        make_stage_c_fixture(true, 3, false)
    } else if stage_c_method_4_attachment {
        make_stage_c_fixture(true, 4, false)
    } else if stage_c_external_id2_attachment {
        make_stage_c_fixture(true, 2, true)
    } else {
        make_fixture(crypt_method)
    };
    fs::write(&args[output_index], bytes)
}
