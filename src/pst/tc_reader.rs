//! General Table Context reader (MS-PST 2.3.4).
//!
//! Reads a table by its declared structure instead of selecting evidence from
//! candidate payloads: the heap-on-node may span several data blocks, the row
//! index BTH may have intermediate levels, and the row matrix may live in a
//! subnode whose data blocks each hold whole rows. Cells are addressed by
//! column tag and are present only when the row's existence bitmap says so.
//!
//! The reader fails closed: a table whose heap, row index, row matrix or
//! column layout disagree yields an error and no rows.
use std::collections::HashSet;

use crate::error::{PstdError, PstdResult};
use crate::pst::bbt::BbtIndex;
use crate::pst::data_tree::load_attachment_data_payload;
use crate::pst::limits::ParserLimits;
use crate::pst::payload::load_payload_block;
use crate::pst::primitives::BlockId;
use crate::pst::property_source::find_subnode_entry;
use crate::pst::reader::PstByteReader;
use crate::pst::tcinfo::{TcColumnDescriptor, TcInfo};

const BID_INTERNAL_MASK: u64 = 0x02;
const XBLOCK_TYPE: u8 = 0x01;
const XBLOCK_HEADER_BYTES: usize = 8;
const HEAP_SIGNATURE: u8 = 0xec;
const HEAP_CLIENT_TABLE_CONTEXT: u8 = 0x7c;
const BTH_HEADER_TYPE: u8 = 0xb5;
const MAX_BTH_LEVELS: u8 = 8;
const PTYP_STRING: u32 = 0x001f;
const PTYP_STRING8: u32 = 0x001e;

/// Everything needed to read blocks of one PST file.
#[derive(Clone, Copy)]
pub struct TcSource<'a> {
    pub reader: &'a PstByteReader,
    pub bbt: &'a BbtIndex,
    pub limits: ParserLimits,
}

/// One row: its row id from the row index and its fixed-size row bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcRow {
    pub row_id: u32,
    bytes: Vec<u8>,
}

/// A fully read table. Rows are in row-matrix order.
#[derive(Debug, Clone)]
pub struct TcTable {
    pub columns: Vec<TcColumnDescriptor>,
    pub rows: Vec<TcRow>,
    heap: Vec<Vec<u8>>,
    subnode_root: Option<BlockId>,
    bitmap_offset: usize,
}

/// Read the table stored in a node with the given data and subnode blocks.
pub fn read_table(
    source: TcSource<'_>,
    data_bid: BlockId,
    subnode_root: Option<BlockId>,
) -> PstdResult<TcTable> {
    let heap = load_data_blocks(source, data_bid)?;
    let first = heap
        .first()
        .ok_or_else(|| parse_error("table node has no data blocks"))?;
    if first.len() < 8 || first[2] != HEAP_SIGNATURE || first[3] != HEAP_CLIENT_TABLE_CONTEXT {
        return Err(parse_error("table node is not a table-context heap"));
    }
    let user_root = u32_at(first, 4)?;
    let tcinfo = TcInfo::parse(heap_allocation(&heap, user_root)?, 0)?;

    let row_size = tcinfo.data_region_boundaries[3] as usize;
    let bitmap_offset = tcinfo.data_region_boundaries[2] as usize;
    let bitmap_bytes = tcinfo.columns.len().div_ceil(8);
    if row_size == 0 || bitmap_offset + bitmap_bytes > row_size {
        return Err(parse_error("table row layout cannot hold its bitmap"));
    }
    for column in &tcinfo.columns {
        if column.data_offset as usize + column.data_size as usize > bitmap_offset {
            return Err(parse_error(format!(
                "table column 0x{:08x} extends past the row data area",
                column.property_tag
            )));
        }
    }

    let index = read_row_index(&heap, tcinfo.row_index_hid)?;
    let matrix = if index.is_empty() {
        Vec::new()
    } else {
        load_row_matrix(source, &heap, subnode_root, tcinfo.rows_hnid)?
    };

    // Rows never span blocks: each block contributes the whole rows it holds.
    let mut slots: Vec<&[u8]> = Vec::new();
    for block in &matrix {
        slots.extend(block.chunks_exact(row_size));
    }
    let mut ordered = index;
    ordered.sort_by_key(|(_, row_index)| *row_index);
    let mut rows = Vec::with_capacity(ordered.len());
    let mut previous = None;
    for (row_id, row_index) in ordered {
        if previous == Some(row_index) {
            return Err(parse_error(format!(
                "table row index repeats row {row_index}"
            )));
        }
        previous = Some(row_index);
        let bytes = slots.get(row_index as usize).ok_or_else(|| {
            parse_error(format!(
                "table row {row_index} is outside the {}-row matrix",
                slots.len()
            ))
        })?;
        rows.push(TcRow {
            row_id,
            bytes: bytes.to_vec(),
        });
    }

    Ok(TcTable {
        columns: tcinfo.columns,
        rows,
        heap,
        subnode_root,
        bitmap_offset,
    })
}

/// Read only the row ids of a table, in row-index order.
///
/// Folder membership needs nothing but the ids, so a damaged row matrix does
/// not cost a folder its messages.
pub fn read_row_ids(source: TcSource<'_>, data_bid: BlockId) -> PstdResult<Vec<u32>> {
    let heap = load_data_blocks(source, data_bid)?;
    let first = heap
        .first()
        .ok_or_else(|| parse_error("table node has no data blocks"))?;
    if first.len() < 8 || first[2] != HEAP_SIGNATURE || first[3] != HEAP_CLIENT_TABLE_CONTEXT {
        return Err(parse_error("table node is not a table-context heap"));
    }
    let tcinfo = TcInfo::parse(heap_allocation(&heap, u32_at(first, 4)?)?, 0)?;
    Ok(read_row_index(&heap, tcinfo.row_index_hid)?
        .into_iter()
        .map(|(row_id, _)| row_id)
        .collect())
}

impl TcTable {
    fn column(&self, tag: u32) -> Option<&TcColumnDescriptor> {
        self.columns
            .iter()
            .find(|column| column.property_tag == tag)
    }

    /// The raw cell bytes, or `None` when the column is absent from the table
    /// or the row's existence bitmap marks the cell as not set.
    fn cell<'a>(&self, row: &'a TcRow, tag: u32) -> Option<&'a [u8]> {
        let column = self.column(tag)?;
        let bit = column.bitmap_bit as usize;
        let flags = *row.bytes.get(self.bitmap_offset + bit / 8)?;
        if flags & (0x80 >> (bit % 8)) == 0 {
            return None;
        }
        let start = column.data_offset as usize;
        row.bytes.get(start..start + column.data_size as usize)
    }

    /// A 32-bit integer cell.
    pub fn u32(&self, row: &TcRow, tag: u32) -> Option<u32> {
        let cell = self.cell(row, tag)?;
        Some(u32::from_le_bytes(cell.try_into().ok()?))
    }

    /// A variable-size cell, resolved through the heap or the table's subnodes.
    /// `Ok(None)` means the cell is not set; an unresolvable reference is an error.
    pub fn bytes(
        &self,
        source: TcSource<'_>,
        row: &TcRow,
        tag: u32,
    ) -> PstdResult<Option<Vec<u8>>> {
        let Some(cell) = self.cell(row, tag) else {
            return Ok(None);
        };
        let hnid = u32::from_le_bytes(
            cell.try_into()
                .map_err(|_| parse_error("variable-size table cell is not a 4-byte reference"))?,
        );
        if hnid == 0 {
            return Ok(Some(Vec::new()));
        }
        if hnid & 0x1f == 0 {
            return Ok(Some(heap_allocation(&self.heap, hnid)?.to_vec()));
        }
        let (data_bid, _) = self
            .subnode_root
            .and_then(|root| {
                find_subnode_entry(root, hnid, source.reader, source.bbt, source.limits)
            })
            .ok_or_else(|| parse_error(format!("table cell subnode 0x{hnid:x} not found")))?;
        let payload =
            load_attachment_data_payload(source.reader, source.bbt, data_bid, None, source.limits)?;
        Ok(Some(payload.bytes))
    }

    /// A string cell by property id, whichever string type the column uses.
    /// `decode_string8` decodes non-Unicode columns.
    pub fn string(
        &self,
        source: TcSource<'_>,
        row: &TcRow,
        property_id: u16,
        decode_string8: impl Fn(&[u8]) -> String,
    ) -> PstdResult<Option<String>> {
        let base = u32::from(property_id) << 16;
        if let Some(raw) = self.bytes(source, row, base | PTYP_STRING)? {
            if raw.len() % 2 != 0 {
                return Err(parse_error("UTF-16 table cell has an odd length"));
            }
            let units: Vec<u16> = raw
                .chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .collect();
            return Ok(Some(String::from_utf16_lossy(&units)));
        }
        Ok(self
            .bytes(source, row, base | PTYP_STRING8)?
            .map(|raw| decode_string8(&raw)))
    }
}

fn parse_error(message: impl Into<String>) -> PstdError {
    PstdError::pst_parse(None, message.into())
}

fn u16_at(bytes: &[u8], offset: usize) -> PstdResult<u16> {
    bytes
        .get(offset..offset + 2)
        .map(|field| u16::from_le_bytes([field[0], field[1]]))
        .ok_or_else(|| parse_error("table heap field outside its block"))
}

fn u32_at(bytes: &[u8], offset: usize) -> PstdResult<u32> {
    bytes
        .get(offset..offset + 4)
        .map(|field| u32::from_le_bytes([field[0], field[1], field[2], field[3]]))
        .ok_or_else(|| parse_error("table heap field outside its block"))
}

fn is_ansi(source: TcSource<'_>) -> PstdResult<bool> {
    let version = source.reader.read_at(10, 2)?;
    Ok(matches!(
        u16::from_le_bytes([version[0], version[1]]),
        14 | 15
    ))
}

/// Load the leaf data blocks of a data tree, one buffer per block, in order.
fn load_data_blocks(source: TcSource<'_>, root: BlockId) -> PstdResult<Vec<Vec<u8>>> {
    let bid_width = if is_ansi(source)? { 4 } else { 8 };
    let mut blocks = Vec::new();
    let mut seen = HashSet::new();
    append_data_blocks(source, root, bid_width, 0, &mut seen, &mut blocks)?;
    Ok(blocks)
}

fn append_data_blocks(
    source: TcSource<'_>,
    bid: BlockId,
    bid_width: usize,
    depth: usize,
    seen: &mut HashSet<BlockId>,
    blocks: &mut Vec<Vec<u8>>,
) -> PstdResult<()> {
    if depth > source.limits.max_subnode_depth || !seen.insert(bid) {
        return Err(parse_error(format!(
            "table data tree is too deep or repeats block 0x{:x}",
            bid.0
        )));
    }
    let payload = load_payload_block(source.reader, source.bbt, bid, source.limits)?;
    if bid.0 & BID_INTERNAL_MASK == 0 {
        blocks.push(payload.bytes);
        return Ok(());
    }
    let bytes = &payload.bytes;
    if bytes.len() < XBLOCK_HEADER_BYTES || bytes[0] != XBLOCK_TYPE || !matches!(bytes[1], 1 | 2) {
        return Err(parse_error(format!(
            "table data tree block 0x{:x} is not an XBLOCK",
            bid.0
        )));
    }
    let count = u16_at(bytes, 2)? as usize;
    let children = bytes
        .get(XBLOCK_HEADER_BYTES..XBLOCK_HEADER_BYTES + count * bid_width)
        .ok_or_else(|| parse_error("table data tree child array is truncated"))?;
    for child in children.chunks_exact(bid_width) {
        let child = if bid_width == 4 {
            u64::from(u32::from_le_bytes([child[0], child[1], child[2], child[3]]))
        } else {
            u64::from_le_bytes(child.try_into().expect("8-byte chunk"))
        };
        append_data_blocks(source, BlockId(child), bid_width, depth + 1, seen, blocks)?;
    }
    Ok(())
}

/// Resolve a heap id across the blocks of a heap-on-node. Every block starts
/// with the offset of its own page map, whatever header it carries.
fn heap_allocation(heap: &[Vec<u8>], hid: u32) -> PstdResult<&[u8]> {
    let missing = || parse_error(format!("table heap id 0x{hid:08x} does not resolve"));
    let index = ((hid >> 5) & 0x7ff) as usize;
    if hid & 0x1f != 0 || index == 0 {
        return Err(missing());
    }
    let block = heap.get((hid >> 16) as usize).ok_or_else(missing)?;
    let page_map = u16_at(block, 0)? as usize;
    let count = u16_at(block, page_map)? as usize;
    if index > count {
        return Err(missing());
    }
    let start = u16_at(block, page_map + 4 + (index - 1) * 2)? as usize;
    let end = u16_at(block, page_map + 4 + index * 2)? as usize;
    if end < start || end > page_map {
        return Err(missing());
    }
    Ok(&block[start..end])
}

/// Read the row index BTH as `(row id, row index)` pairs in key order.
fn read_row_index(heap: &[Vec<u8>], header_hid: u32) -> PstdResult<Vec<(u32, u32)>> {
    let header = heap_allocation(heap, header_hid)?;
    if header.len() < 8 || header[0] != BTH_HEADER_TYPE {
        return Err(parse_error("table row index is not a BTH"));
    }
    let (key_size, value_size, levels) = (header[1], header[2], header[3]);
    if key_size != 4 || !matches!(value_size, 2 | 4) || levels > MAX_BTH_LEVELS {
        return Err(parse_error(format!(
            "unsupported table row index: key={key_size}, value={value_size}, levels={levels}"
        )));
    }
    let root = u32_at(header, 4)?;
    let mut entries = Vec::new();
    if root != 0 {
        let mut seen = HashSet::new();
        append_row_index(
            heap,
            root,
            levels,
            value_size as usize,
            &mut seen,
            &mut entries,
        )?;
    }
    let mut ids = HashSet::with_capacity(entries.len());
    if let Some((row_id, _)) = entries.iter().find(|(row_id, _)| !ids.insert(*row_id)) {
        return Err(parse_error(format!(
            "table row index repeats row id 0x{row_id:08x}"
        )));
    }
    Ok(entries)
}

fn append_row_index(
    heap: &[Vec<u8>],
    hid: u32,
    level: u8,
    value_size: usize,
    seen: &mut HashSet<u32>,
    entries: &mut Vec<(u32, u32)>,
) -> PstdResult<()> {
    if !seen.insert(hid) {
        return Err(parse_error(format!(
            "table row index revisits heap id 0x{hid:08x}"
        )));
    }
    let allocation = heap_allocation(heap, hid)?;
    let record = 4 + if level == 0 { value_size } else { 4 };
    if allocation.len() % record != 0 {
        return Err(parse_error("table row index node has a partial record"));
    }
    for chunk in allocation.chunks_exact(record) {
        let key = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        if level > 0 {
            let child = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
            append_row_index(heap, child, level - 1, value_size, seen, entries)?;
        } else if value_size == 2 {
            entries.push((key, u32::from(u16::from_le_bytes([chunk[4], chunk[5]]))));
        } else {
            entries.push((
                key,
                u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]),
            ));
        }
    }
    Ok(())
}

/// Load the row matrix as one buffer per block: a single heap allocation, or
/// the data blocks of the subnode `hnidRows` names.
fn load_row_matrix(
    source: TcSource<'_>,
    heap: &[Vec<u8>],
    subnode_root: Option<BlockId>,
    rows_hnid: u32,
) -> PstdResult<Vec<Vec<u8>>> {
    if rows_hnid == 0 {
        return Ok(Vec::new());
    }
    if rows_hnid & 0x1f == 0 {
        return Ok(vec![heap_allocation(heap, rows_hnid)?.to_vec()]);
    }
    let (data_bid, _) = subnode_root
        .and_then(|root| {
            find_subnode_entry(root, rows_hnid, source.reader, source.bbt, source.limits)
        })
        .ok_or_else(|| parse_error(format!("table row subnode 0x{rows_hnid:x} not found")))?;
    load_data_blocks(source, data_bid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pst::bbt::BbtEntry;
    use crate::pst::primitives::ByteOffset;

    const TAG_TYPE: u32 = 0x0c15_0003;
    const TAG_NAME: u32 = 0x3001_001f;
    const ROW_BYTES: usize = 9;

    struct Fixture {
        file: tempfile::NamedTempFile,
        bbt: BbtIndex,
        reader: Option<PstByteReader>,
    }

    impl Fixture {
        fn new(blocks: &[(u64, Vec<u8>)]) -> Self {
            let file = tempfile::NamedTempFile::new().unwrap();
            let mut bytes = vec![0; 1024];
            bytes[10..12].copy_from_slice(&23u16.to_le_bytes());
            let mut bbt = BbtIndex {
                root: None,
                entries: Vec::new(),
                parsed_pages: 0,
                discovered_child_pages: 0,
                traversal_error_count: 0,
                duplicate_entry_count: 0,
                truncated_entry_count: 0,
                status: "test".into(),
                lookup_index: Default::default(),
                page_limit_reached: false,
            };
            for (bid, payload) in blocks {
                bbt.entries.push(BbtEntry {
                    block_id: BlockId(*bid),
                    offset: ByteOffset(bytes.len() as u64),
                    size: payload.len() as u64,
                });
                bytes.extend(payload);
            }
            std::fs::write(file.path(), bytes).unwrap();
            let reader = PstByteReader::open(file.path()).unwrap();
            Self {
                file,
                bbt,
                reader: Some(reader),
            }
        }

        fn source(&self) -> TcSource<'_> {
            let _ = &self.file;
            TcSource {
                reader: self.reader.as_ref().unwrap(),
                bbt: &self.bbt,
                limits: ParserLimits::default(),
            }
        }
    }

    /// One heap block: `header` bytes, the allocations, then the page map.
    fn heap_block(header: &[u8], allocations: &[Vec<u8>]) -> Vec<u8> {
        let mut bytes = header.to_vec();
        let mut offsets = vec![bytes.len() as u16];
        for allocation in allocations {
            bytes.extend(allocation);
            offsets.push(bytes.len() as u16);
        }
        let page_map = bytes.len() as u16;
        bytes[0..2].copy_from_slice(&page_map.to_le_bytes());
        bytes.extend((allocations.len() as u16).to_le_bytes());
        bytes.extend(0u16.to_le_bytes());
        for offset in offsets {
            bytes.extend(offset.to_le_bytes());
        }
        bytes
    }

    fn first_header() -> Vec<u8> {
        let mut header = vec![0, 0, HEAP_SIGNATURE, HEAP_CLIENT_TABLE_CONTEXT];
        header.extend(0x20u32.to_le_bytes());
        header.extend([0; 4]);
        header
    }

    fn tcinfo(rows_hnid: u32) -> Vec<u8> {
        let mut bytes = vec![0x7c, 2];
        for boundary in [8u16, 8, 8, ROW_BYTES as u16] {
            bytes.extend(boundary.to_le_bytes());
        }
        bytes.extend(0x40u32.to_le_bytes());
        bytes.extend(rows_hnid.to_le_bytes());
        bytes.extend(0u32.to_le_bytes());
        for (tag, offset, bit) in [(TAG_TYPE, 0u16, 0u8), (TAG_NAME, 4, 1)] {
            bytes.extend(tag.to_le_bytes());
            bytes.extend(offset.to_le_bytes());
            bytes.extend([4, bit]);
        }
        bytes
    }

    fn bth_header(levels: u8, root: u32) -> Vec<u8> {
        let mut bytes = vec![BTH_HEADER_TYPE, 4, 4, levels];
        bytes.extend(root.to_le_bytes());
        bytes
    }

    fn pairs(entries: &[(u32, u32)]) -> Vec<u8> {
        entries
            .iter()
            .flat_map(|(key, value)| [key.to_le_bytes(), value.to_le_bytes()].concat())
            .collect()
    }

    fn row(kind: u32, name_hnid: u32, bitmap: u8) -> Vec<u8> {
        let mut bytes = kind.to_le_bytes().to_vec();
        bytes.extend(name_hnid.to_le_bytes());
        bytes.push(bitmap);
        bytes
    }

    fn utf16(value: &str) -> Vec<u8> {
        value.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }

    fn sl_leaf(entries: &[(u32, u64)]) -> Vec<u8> {
        let mut bytes = vec![2, 0, entries.len() as u8, 0, 0, 0, 0, 0];
        for (nid, bid) in entries {
            bytes.extend(u64::from(*nid).to_le_bytes());
            bytes.extend(bid.to_le_bytes());
            bytes.extend(0u64.to_le_bytes());
        }
        bytes
    }

    fn xblock(children: &[u64]) -> Vec<u8> {
        let mut bytes = vec![XBLOCK_TYPE, 1];
        bytes.extend((children.len() as u16).to_le_bytes());
        bytes.extend(0u32.to_le_bytes());
        for child in children {
            bytes.extend(child.to_le_bytes());
        }
        bytes
    }

    fn name(table: &TcTable, source: TcSource<'_>, row: &TcRow) -> Option<String> {
        table
            .string(source, row, 0x3001, |_| unreachable!())
            .unwrap()
    }

    #[test]
    fn reads_heap_rows_by_column_tag_and_honours_the_existence_bitmap() {
        let heap = heap_block(
            &first_header(),
            &[
                tcinfo(0x80),
                bth_header(0, 0x60),
                pairs(&[(7, 1), (9, 0)]),
                [row(1, 0xa0, 0xc0), row(2, 0xa0, 0x80)].concat(),
                utf16("Ada"),
            ],
        );
        let fixture = Fixture::new(&[(0x10, heap)]);
        let table = read_table(fixture.source(), BlockId(0x10), None).unwrap();

        // Row-matrix order, not key order.
        assert_eq!(table.rows[0].row_id, 9);
        assert_eq!(table.u32(&table.rows[0], TAG_TYPE), Some(1));
        assert_eq!(
            name(&table, fixture.source(), &table.rows[0]).as_deref(),
            Some("Ada")
        );
        assert_eq!(table.u32(&table.rows[1], TAG_TYPE), Some(2));
        assert_eq!(name(&table, fixture.source(), &table.rows[1]), None);
        assert_eq!(
            read_row_ids(fixture.source(), BlockId(0x10)).unwrap(),
            vec![7, 9]
        );
    }

    #[test]
    fn follows_a_multi_block_heap_a_two_level_index_and_subnode_rows() {
        // Block 0 holds the table header; block 1 the index leaves and a string.
        let block0 = heap_block(
            &first_header(),
            &[
                tcinfo(0x7f),
                bth_header(1, 0x60),
                pairs(&[(1, 0x0001_0020), (3, 0x0001_0040)]),
            ],
        );
        let block1 = heap_block(
            &[0, 0],
            &[pairs(&[(1, 0), (2, 1)]), pairs(&[(3, 2)]), utf16("Bo")],
        );
        // Two rows fit the first row block with a byte to spare; the third row
        // starts the next block.
        let mut rows0 = [row(1, 0, 0x80), row(2, 0, 0x80)].concat();
        rows0.push(0xff);
        let rows1 = row(3, 0x0001_0060, 0xc0);
        let fixture = Fixture::new(&[
            (0x12, xblock(&[0x10, 0x14])),
            (0x10, block0),
            (0x14, block1),
            (0x22, sl_leaf(&[(0x7f, 0x32)])),
            (0x32, xblock(&[0x30, 0x34])),
            (0x30, rows0),
            (0x34, rows1),
        ]);
        let table = read_table(fixture.source(), BlockId(0x12), Some(BlockId(0x22))).unwrap();

        let kinds: Vec<_> = table
            .rows
            .iter()
            .map(|row| table.u32(row, TAG_TYPE))
            .collect();
        assert_eq!(kinds, vec![Some(1), Some(2), Some(3)]);
        assert_eq!(
            name(&table, fixture.source(), &table.rows[2]).as_deref(),
            Some("Bo")
        );
    }

    #[test]
    fn returns_no_rows_when_the_index_points_outside_the_matrix() {
        let heap = heap_block(
            &first_header(),
            &[
                tcinfo(0x80),
                bth_header(0, 0x60),
                pairs(&[(7, 0), (9, 5)]),
                row(1, 0, 0x80),
            ],
        );
        let fixture = Fixture::new(&[(0x10, heap)]);
        let error = read_table(fixture.source(), BlockId(0x10), None).unwrap_err();
        assert!(error.to_string().contains("outside the 1-row matrix"));
    }

    #[test]
    fn reads_an_empty_table() {
        let heap = heap_block(&first_header(), &[tcinfo(0), bth_header(0, 0)]);
        let fixture = Fixture::new(&[(0x10, heap)]);
        let table = read_table(fixture.source(), BlockId(0x10), None).unwrap();
        assert!(table.rows.is_empty());
    }
}
