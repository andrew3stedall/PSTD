use std::{env, fs, io};

const PAGE_SIZE: usize = 512;
const BBT_OFFSET: usize = 1024;
const NBT_OFFSET: usize = 1536;
const FILE_SIZE: usize = NBT_OFFSET + PAGE_SIZE;

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
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
    !crc
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
    put_u32(&mut page, 508, crc32(&page[..500]));
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

    put_u32(&mut bytes, 4, crc32(&bytes[8..479]));
    bytes
}

fn main() -> io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if !(2..=3).contains(&args.len()) {
        eprintln!("usage: ansi_fixture <output-path> [crypt-method]");
        std::process::exit(2);
    }
    let crypt_method = args
        .get(2)
        .map(|value| value.parse::<u8>().expect("crypt method must be an integer"))
        .unwrap_or(0);
    fs::write(&args[1], make_fixture(crypt_method))
}
