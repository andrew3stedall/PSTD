//! Deterministic synthetic Unicode PST emitter for scale and robustness testing.
//!
//! This is a fixture-only writer, not a general PST writer. It emits the MS-PST
//! structures a reader needs (header, NBT/BBT pages, data blocks with trailers,
//! XBLOCK/XXBLOCK data trees, SLBLOCK subnode trees, heap-on-node, BTH, property
//! contexts and table contexts) directly from the published layouts and shares no
//! code with the PSTD reader. It does not write allocation maps, so the output is
//! not intended to be opened by Outlook.
//!
//! All content is synthetic: reserved example domains, generated words and
//! pseudo-random bytes derived from the seed. The same arguments always produce the
//! same bytes.
//!
//! Standalone build:
//!
//! ```text
//! rustc --edition=2021 -O tools/synthetic_pst.rs -o synthetic-pst
//! synthetic-pst --messages 50000 --seed 7 --manifest mailbox.json mailbox.pst
//! ```
#![allow(dead_code)]

use std::fmt::Write as _;
use std::io::{self, Seek, SeekFrom, Write};

// ---------------------------------------------------------------------------
// NDB layer constants
// ---------------------------------------------------------------------------

const PAGE_SIZE: usize = 512;
const BLOCK_TRAILER_BYTES: usize = 16;
const MAX_BLOCK_DATA: usize = 8176;
const FIRST_BLOCK_OFFSET: u64 = 0x4400;
const HEADER_BYTES: usize = 564;
const PAGE_TYPE_BBT: u8 = 0x80;
const PAGE_TYPE_NBT: u8 = 0x81;
const NBT_LEAF_ENTRY_BYTES: usize = 32;
const BBT_LEAF_ENTRY_BYTES: usize = 24;
const BT_INTERMEDIATE_ENTRY_BYTES: usize = 24;
const BT_ENTRY_AREA_BYTES: usize = 488;
const XBLOCK_MAX_ENTRIES: usize = (MAX_BLOCK_DATA - 8) / 8;
const SLBLOCK_MAX_ENTRIES: usize = (MAX_BLOCK_DATA - 8) / 24;

// ---------------------------------------------------------------------------
// LTP layer constants
// ---------------------------------------------------------------------------

const HEAP_MAX_ALLOCATION: usize = 3580;
const HEAP_MAX_ALLOCATIONS_PER_BLOCK: usize = 2047;
const HEAP_SIG_PROPERTY_CONTEXT: u8 = 0xbc;
const HEAP_SIG_TABLE_CONTEXT: u8 = 0x7c;

const PT_INT16: u16 = 0x0002;
const PT_INT32: u16 = 0x0003;
const PT_BOOL: u16 = 0x000b;
const PT_OBJECT: u16 = 0x000d;
const PT_INT64: u16 = 0x0014;
const PT_UNICODE: u16 = 0x001f;
const PT_TIME: u16 = 0x0040;
const PT_BINARY: u16 = 0x0102;

// Node id types.
const NID_TYPE_NORMAL_FOLDER: u32 = 0x02;
const NID_TYPE_NORMAL_MESSAGE: u32 = 0x04;
const NID_TYPE_ATTACHMENT: u32 = 0x05;
const NID_TYPE_HIERARCHY_TABLE: u32 = 0x0d;
const NID_TYPE_CONTENTS_TABLE: u32 = 0x0e;
const NID_TYPE_ASSOC_CONTENTS_TABLE: u32 = 0x0f;
const NID_TYPE_LTP: u32 = 0x1f;

const NID_MESSAGE_STORE: u32 = 0x21;
const NID_NAME_TO_ID_MAP: u32 = 0x61;
const NID_ROOT_FOLDER: u32 = 0x122;
const NID_ATTACHMENT_TABLE: u32 = 0x671;
const NID_RECIPIENT_TABLE: u32 = 0x692;

// Property tags (id << 16 | type).
const PR_IMPORTANCE: u32 = 0x0017_0003;
const PR_MESSAGE_CLASS: u32 = 0x001a_001f;
const PR_SUBJECT: u32 = 0x0037_001f;
const PR_CLIENT_SUBMIT_TIME: u32 = 0x0039_0040;
const PR_SENT_REPRESENTING_NAME: u32 = 0x0042_001f;
const PR_TRANSPORT_HEADERS: u32 = 0x007d_001f;
const PR_SENDER_NAME: u32 = 0x0c1a_001f;
const PR_RECIPIENT_TYPE: u32 = 0x0c15_0003;
const PR_SENDER_ADDRTYPE: u32 = 0x0c1e_001f;
const PR_SENDER_EMAIL_ADDRESS: u32 = 0x0c1f_001f;
const PR_DISPLAY_CC: u32 = 0x0e03_001f;
const PR_DISPLAY_TO: u32 = 0x0e04_001f;
const PR_MESSAGE_DELIVERY_TIME: u32 = 0x0e06_0040;
const PR_MESSAGE_FLAGS: u32 = 0x0e07_0003;
const PR_MESSAGE_SIZE: u32 = 0x0e08_0003;
const PR_RESPONSIBILITY: u32 = 0x0e0f_000b;
const PR_HASATTACH: u32 = 0x0e1b_000b;
const PR_ATTACH_SIZE: u32 = 0x0e20_0003;
const PR_RECORD_KEY: u32 = 0x0ff9_0102;
const PR_OBJECT_TYPE: u32 = 0x0ffe_0003;
const PR_BODY: u32 = 0x1000_001f;
const PR_HTML: u32 = 0x1013_0102;
const PR_INTERNET_MESSAGE_ID: u32 = 0x1035_001f;
const PR_DISPLAY_NAME: u32 = 0x3001_001f;
const PR_ADDRTYPE: u32 = 0x3002_001f;
const PR_EMAIL_ADDRESS: u32 = 0x3003_001f;
const PR_CREATION_TIME: u32 = 0x3007_0040;
const PR_LAST_MODIFICATION_TIME: u32 = 0x3008_0040;
const PR_VALID_FOLDER_MASK: u32 = 0x35df_0003;
const PR_IPM_SUBTREE_ENTRYID: u32 = 0x35e0_0102;
const PR_IPM_WASTEBASKET_ENTRYID: u32 = 0x35e3_0102;
const PR_FINDER_ENTRYID: u32 = 0x35e7_0102;
const PR_CONTENT_COUNT: u32 = 0x3602_0003;
const PR_CONTENT_UNREAD: u32 = 0x3603_0003;
const PR_SUBFOLDERS: u32 = 0x360a_000b;
const PR_ATTACH_DATA_BIN: u32 = 0x3701_0102;
const PR_ATTACH_DATA_OBJ: u32 = 0x3701_000d;
const PR_ATTACH_FILENAME: u32 = 0x3704_001f;
const PR_ATTACH_METHOD: u32 = 0x3705_0003;
const PR_ATTACH_LONG_FILENAME: u32 = 0x3707_001f;
const PR_RENDERING_POSITION: u32 = 0x370b_0003;
const PR_ATTACH_MIME_TAG: u32 = 0x370e_001f;
const PR_DISPLAY_TYPE: u32 = 0x3900_0003;
const PR_SMTP_ADDRESS: u32 = 0x39fe_001f;
const PR_LTP_ROW_ID: u32 = 0x67f2_0003;
const PR_LTP_ROW_VER: u32 = 0x67f3_0003;
const PR_PST_PASSWORD: u32 = 0x67ff_0003;
const PR_NAMEID_BUCKET_COUNT: u32 = 0x0001_0003;
const PR_NAMEID_STREAM_GUID: u32 = 0x0002_0102;
const PR_NAMEID_STREAM_ENTRY: u32 = 0x0003_0102;
const PR_NAMEID_STREAM_STRING: u32 = 0x0004_0102;

const MSGFLAG_READ: u32 = 0x01;
const MSGFLAG_UNSENT: u32 = 0x08;
const MSGFLAG_HASATTACH: u32 = 0x10;

const EMBEDDED_OBJECT_NOMINAL_BYTES: u32 = 4096;

/// 2020-01-01T00:00:00Z as a FILETIME.
const BASE_FILETIME: u64 = 132_223_104_000_000_000;
const FILETIME_MINUTE: u64 = 600_000_000;

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

fn crc32_table() -> &'static [u32; 256] {
    static TABLE: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [0u32; 256];
        for (index, slot) in table.iter_mut().enumerate() {
            let mut value = index as u32;
            for _ in 0..8 {
                value = if value & 1 == 1 {
                    (value >> 1) ^ 0xedb8_8320
                } else {
                    value >> 1
                };
            }
            *slot = value;
        }
        table
    })
}

/// MS-PST CRC: reflected CRC-32 polynomial, initial value zero, no final complement.
fn crc32(bytes: &[u8]) -> u32 {
    let table = crc32_table();
    let mut crc = 0u32;
    for &byte in bytes {
        crc = table[((crc ^ u32::from(byte)) & 0xff) as usize] ^ (crc >> 8);
    }
    crc
}

fn signature(file_offset: u64, bid: u64) -> u16 {
    let value = (file_offset ^ bid) as u32;
    ((value >> 16) as u16) ^ (value as u16)
}

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn utf16(value: &str) -> Vec<u8> {
    value.encode_utf16().flat_map(u16::to_le_bytes).collect()
}

/// FNV-1a 64-bit, used by the manifest so tests can compare payloads cheaply.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// SplitMix64: small, well-distributed and fully deterministic.
#[derive(Clone)]
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound.max(1)
    }

    fn bytes(&mut self, len: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(len + 8);
        while out.len() < len {
            out.extend_from_slice(&self.next().to_le_bytes());
        }
        out.truncate(len);
        out
    }
}

const WORDS: &[&str] = &[
    "quarterly",
    "report",
    "meeting",
    "agenda",
    "budget",
    "review",
    "schedule",
    "update",
    "draft",
    "proposal",
    "invoice",
    "summary",
    "project",
    "timeline",
    "release",
    "notes",
    "feedback",
    "request",
    "follow",
    "status",
    "planning",
    "design",
    "contract",
    "renewal",
    "travel",
    "expenses",
    "onboarding",
    "training",
    "survey",
    "results",
    "launch",
    "roadmap",
    "minutes",
    "reminder",
    "approval",
    "estimate",
    "delivery",
    "inventory",
    "support",
    "handover",
];

const NAMES: &[&str] = &[
    "Alex Morgan",
    "Sam Rivera",
    "Jordan Lee",
    "Taylor Quinn",
    "Casey Patel",
    "Robin Chen",
    "Jamie Novak",
    "Morgan Blake",
    "Avery Stone",
    "Riley Okafor",
    "Drew Castillo",
    "Quinn Harper",
];

const DOMAINS: &[&str] = &["example.com", "example.org", "example.net"];

fn words(rng: &mut Rng, count: usize) -> String {
    let mut out = String::new();
    for index in 0..count {
        if index > 0 {
            out.push(' ');
        }
        out.push_str(WORDS[rng.below(WORDS.len() as u64) as usize]);
    }
    out
}

fn person(rng: &mut Rng) -> (String, String) {
    let name = NAMES[rng.below(NAMES.len() as u64) as usize];
    let domain = DOMAINS[rng.below(DOMAINS.len() as u64) as usize];
    let local = name.to_lowercase().replace(' ', ".");
    (name.to_string(), format!("{local}@{domain}"))
}

// ---------------------------------------------------------------------------
// Heap-on-node
// ---------------------------------------------------------------------------

/// Multi-block heap-on-node. Each block is written as one data block of the node's
/// data tree, so a heap identifier carries the block index.
struct Heap {
    client_signature: u8,
    blocks: Vec<Vec<Vec<u8>>>,
    user_root: u32,
}

impl Heap {
    fn new(client_signature: u8) -> Self {
        Self {
            client_signature,
            blocks: vec![Vec::new()],
            user_root: 0,
        }
    }

    fn header_bytes(block_index: usize) -> usize {
        if block_index == 0 {
            12
        } else if block_index % 128 == 8 {
            66
        } else {
            2
        }
    }

    fn block_bytes(block_index: usize, allocations: &[Vec<u8>], extra: usize) -> usize {
        let data: usize = allocations.iter().map(Vec::len).sum::<usize>() + extra;
        let page_map_start = (Self::header_bytes(block_index) + data + 1) & !1;
        let count = allocations.len() + usize::from(extra > 0);
        page_map_start + 4 + 2 * (count + 1)
    }

    /// Stores `bytes` and returns its heap identifier.
    fn alloc(&mut self, bytes: Vec<u8>) -> u32 {
        assert!(!bytes.is_empty(), "heap allocations are non-empty");
        assert!(
            bytes.len() <= HEAP_MAX_ALLOCATION,
            "heap allocation too large"
        );
        let last = self.blocks.len() - 1;
        let fits = Self::block_bytes(last, &self.blocks[last], bytes.len()) <= MAX_BLOCK_DATA
            && self.blocks[last].len() < HEAP_MAX_ALLOCATIONS_PER_BLOCK;
        if !fits {
            self.blocks.push(Vec::new());
        }
        let block_index = self.blocks.len() - 1;
        assert!(block_index <= 0xffff, "heap has too many blocks");
        self.blocks[block_index].push(bytes);
        let index = self.blocks[block_index].len() as u32;
        (index << 5) | ((block_index as u32) << 16)
    }

    /// Replaces an allocation with bytes of the same length.
    fn patch(&mut self, hid: u32, bytes: Vec<u8>) {
        let block_index = (hid >> 16) as usize;
        let index = ((hid >> 5) & 0x7ff) as usize - 1;
        assert_eq!(self.blocks[block_index][index].len(), bytes.len());
        self.blocks[block_index][index] = bytes;
    }

    fn finish(self) -> Vec<Vec<u8>> {
        let mut out = Vec::with_capacity(self.blocks.len());
        for (block_index, allocations) in self.blocks.iter().enumerate() {
            let header = Self::header_bytes(block_index);
            let mut block = vec![0u8; header];
            let mut offsets = Vec::with_capacity(allocations.len() + 1);
            for allocation in allocations {
                offsets.push(block.len() as u16);
                block.extend_from_slice(allocation);
            }
            offsets.push(block.len() as u16);
            if block.len() % 2 == 1 {
                block.push(0);
            }
            let page_map_offset = block.len() as u16;
            block.extend_from_slice(&(allocations.len() as u16).to_le_bytes());
            block.extend_from_slice(&0u16.to_le_bytes());
            for offset in offsets {
                block.extend_from_slice(&offset.to_le_bytes());
            }
            put_u16(&mut block, 0, page_map_offset);
            if block_index == 0 {
                block[2] = 0xec;
                block[3] = self.client_signature;
                put_u32(&mut block, 4, self.user_root);
            }
            assert!(block.len() <= MAX_BLOCK_DATA);
            out.push(block);
        }
        out
    }
}

/// Builds a BTree-on-heap and returns the identifier of its header.
///
/// `header_hid` is a previously reserved eight-byte allocation, so the header keeps
/// its conventional position ahead of the records.
fn build_bth(heap: &mut Heap, header_hid: u32, key_bytes: usize, records: &[Vec<u8>]) {
    let entry_bytes = records.first().map_or(0, |record| record.len() - key_bytes);
    let mut level: Vec<(Vec<u8>, u32)> = Vec::new();
    if let Some(first) = records.first() {
        let per_leaf = HEAP_MAX_ALLOCATION / first.len();
        for chunk in records.chunks(per_leaf) {
            let key = chunk[0][..key_bytes].to_vec();
            let hid = heap.alloc(chunk.concat());
            level.push((key, hid));
        }
    }
    let mut levels = 0u8;
    while level.len() > 1 {
        let per_node = HEAP_MAX_ALLOCATION / (key_bytes + 4);
        let mut next = Vec::new();
        for chunk in level.chunks(per_node) {
            let key = chunk[0].0.clone();
            let mut node = Vec::with_capacity(chunk.len() * (key_bytes + 4));
            for (child_key, child_hid) in chunk {
                node.extend_from_slice(child_key);
                node.extend_from_slice(&child_hid.to_le_bytes());
            }
            next.push((key, heap.alloc(node)));
        }
        level = next;
        levels += 1;
    }
    let root = level.first().map_or(0, |(_, hid)| *hid);
    let mut header = vec![0xb5, key_bytes as u8, entry_bytes as u8, levels];
    header.extend_from_slice(&root.to_le_bytes());
    if records.is_empty() {
        // An empty tree still declares its record shape.
        header[2] = if key_bytes == 2 { 6 } else { 4 };
    }
    heap.patch(header_hid, header);
}

// ---------------------------------------------------------------------------
// Subnodes
// ---------------------------------------------------------------------------

/// Deliberate damage applied when a node's data is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    None,
    /// The node references a data block that is absent from the block index.
    MissingBlock,
    /// The data tree lists a final block that is absent from the block index.
    TruncatedTree,
    /// The data blocks hold pseudo-random bytes instead of the real structure.
    Garbage,
    /// One data byte differs from the bytes the block checksum was computed over.
    ChecksumMismatch,
}

impl Fault {
    pub fn label(self) -> &'static str {
        match self {
            Fault::None => "none",
            Fault::MissingBlock => "missing_block",
            Fault::TruncatedTree => "truncated_tree",
            Fault::Garbage => "garbage",
            Fault::ChecksumMismatch => "checksum_mismatch",
        }
    }
}

struct NodeData {
    blocks: Vec<Vec<u8>>,
    children: SubnodeSet,
    fault: Fault,
}

impl NodeData {
    fn new(blocks: Vec<Vec<u8>>, children: SubnodeSet) -> Self {
        Self {
            blocks,
            children,
            fault: Fault::None,
        }
    }
}

#[derive(Default)]
struct SubnodeSet {
    nodes: Vec<(u32, NodeData)>,
    next_index: u32,
}

impl SubnodeSet {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            next_index: 0x400,
        }
    }

    fn next_nid(&mut self, nid_type: u32) -> u32 {
        let nid = (self.next_index << 5) | nid_type;
        self.next_index += 1;
        nid
    }

    fn add(&mut self, nid: u32, node: NodeData) {
        self.nodes.push((nid, node));
    }

    /// Stores a byte stream as a data tree and returns its node id.
    fn add_stream(&mut self, bytes: &[u8]) -> u32 {
        let nid = self.next_nid(NID_TYPE_LTP);
        let blocks = bytes.chunks(MAX_BLOCK_DATA).map(<[u8]>::to_vec).collect();
        self.add(nid, NodeData::new(blocks, SubnodeSet::default()));
        nid
    }

    /// Stores pre-split blocks (for example whole table rows per block).
    fn add_blocks(&mut self, blocks: Vec<Vec<u8>>) -> u32 {
        let nid = self.next_nid(NID_TYPE_LTP);
        self.add(nid, NodeData::new(blocks, SubnodeSet::default()));
        nid
    }

    fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Property context
// ---------------------------------------------------------------------------

struct Prop {
    tag: u32,
    value: Vec<u8>,
}

fn prop_i32(tag: u32, value: i32) -> Prop {
    Prop {
        tag,
        value: value.to_le_bytes().to_vec(),
    }
}

fn prop_bool(tag: u32, value: bool) -> Prop {
    Prop {
        tag,
        value: vec![u8::from(value)],
    }
}

fn prop_str(tag: u32, value: &str) -> Prop {
    Prop {
        tag,
        value: utf16(value),
    }
}

fn prop_bin(tag: u32, value: &[u8]) -> Prop {
    Prop {
        tag,
        value: value.to_vec(),
    }
}

fn prop_time(tag: u32, filetime: u64) -> Prop {
    Prop {
        tag,
        value: filetime.to_le_bytes().to_vec(),
    }
}

/// Builds a property context. Values too large for a heap allocation become
/// subnode streams in `subnodes`.
fn build_property_context(props: Vec<Prop>, subnodes: &mut SubnodeSet) -> Vec<Vec<u8>> {
    build_property_context_with_reference(props, None, subnodes)
}

/// As `build_property_context`; `reference` adds a property (tag, node id) whose value
/// is an existing subnode id rather than bytes.
fn build_property_context_with_reference(
    mut props: Vec<Prop>,
    reference: Option<(u32, u32)>,
    subnodes: &mut SubnodeSet,
) -> Vec<Vec<u8>> {
    if let Some((tag, _)) = reference {
        props.push(Prop {
            tag,
            value: Vec::new(),
        });
    }
    props.sort_by_key(|prop| prop.tag >> 16);
    let mut heap = Heap::new(HEAP_SIG_PROPERTY_CONTEXT);
    let header_hid = heap.alloc(vec![0u8; 8]);
    heap.user_root = header_hid;
    let mut records = Vec::with_capacity(props.len());
    for prop in &props {
        let prop_type = (prop.tag & 0xffff) as u16;
        let value = match (reference, prop_type) {
            (Some((tag, nid)), _) if tag == prop.tag => nid,
            (_, PT_INT16 | PT_INT32 | PT_BOOL) => {
                let mut inline = [0u8; 4];
                inline[..prop.value.len()].copy_from_slice(&prop.value);
                u32::from_le_bytes(inline)
            }
            _ if prop.value.is_empty() => 0,
            _ if prop.value.len() <= HEAP_MAX_ALLOCATION => heap.alloc(prop.value.clone()),
            _ => subnodes.add_stream(&prop.value),
        };
        let mut record = Vec::with_capacity(8);
        record.extend_from_slice(&((prop.tag >> 16) as u16).to_le_bytes());
        record.extend_from_slice(&prop_type.to_le_bytes());
        record.extend_from_slice(&value.to_le_bytes());
        records.push(record);
    }
    build_bth(&mut heap, header_hid, 2, &records);
    heap.finish()
}

// ---------------------------------------------------------------------------
// Table context
// ---------------------------------------------------------------------------

struct TableRow {
    row_id: u32,
    /// One value per declared column; `None` leaves the cell absent.
    values: Vec<Option<Vec<u8>>>,
}

fn column_width(tag: u32) -> usize {
    match (tag & 0xffff) as u16 {
        PT_INT64 | PT_TIME => 8,
        PT_INT16 => 2,
        PT_BOOL => 1,
        _ => 4,
    }
}

/// Builds a table context for `columns` (row id and row version are added). Rows too
/// large for one heap allocation are stored in a subnode, whole rows per block.
fn build_table_context(
    columns: &[u32],
    rows: &[TableRow],
    subnodes: &mut SubnodeSet,
) -> Vec<Vec<u8>> {
    // Layout order: row id, row version, then 8/4-byte, 2-byte and 1-byte columns.
    let mut layout: Vec<(u32, usize, usize, usize)> = Vec::new(); // tag, offset, width, column
    let mut offset = 8usize;
    let mut ordered: Vec<(usize, u32)> = columns.iter().copied().enumerate().collect();
    ordered.sort_by_key(|(_, tag)| match column_width(*tag) {
        8 => 0,
        4 => 1,
        2 => 2,
        _ => 3,
    });
    let mut ends = [0usize; 4];
    for (column, tag) in &ordered {
        let width = column_width(*tag);
        layout.push((*tag, offset, width, *column));
        offset += width;
        let group = match width {
            8 | 4 => 0,
            2 => 1,
            _ => 2,
        };
        ends[group] = offset;
    }
    ends[0] = ends[0].max(8);
    ends[1] = ends[1].max(ends[0]);
    ends[2] = ends[2].max(ends[1]);
    let column_count = columns.len() + 2;
    let bitmap_bytes = column_count.div_ceil(8);
    ends[3] = ends[2] + bitmap_bytes;
    let row_bytes = ends[3];

    let mut heap = Heap::new(HEAP_SIG_TABLE_CONTEXT);
    let info_hid = heap.alloc(vec![0u8; 22 + 8 * column_count]);
    heap.user_root = info_hid;
    let index_hid = heap.alloc(vec![0u8; 8]);

    let mut matrix = Vec::with_capacity(rows.len());
    for row in rows {
        assert_eq!(row.values.len(), columns.len());
        let mut bytes = vec![0u8; row_bytes];
        put_u32(&mut bytes, 0, row.row_id);
        let set_bit = |bytes: &mut [u8], bit: usize| {
            bytes[ends[2] + bit / 8] |= 1 << (7 - bit % 8);
        };
        set_bit(&mut bytes, 0);
        set_bit(&mut bytes, 1);
        for (position, (tag, offset, width, column)) in layout.iter().enumerate() {
            let Some(value) = &row.values[*column] else {
                continue;
            };
            let variable = matches!((tag & 0xffff) as u16, PT_UNICODE | PT_BINARY);
            if variable {
                let hnid = if value.is_empty() {
                    0
                } else {
                    heap.alloc(value.clone())
                };
                put_u32(&mut bytes, *offset, hnid);
            } else {
                bytes[*offset..*offset + value.len().min(*width)]
                    .copy_from_slice(&value[..value.len().min(*width)]);
            }
            set_bit(&mut bytes, position + 2);
        }
        matrix.push(bytes);
    }

    let rows_hnid = if matrix.is_empty() {
        0
    } else if matrix.len() * row_bytes <= HEAP_MAX_ALLOCATION {
        heap.alloc(matrix.concat())
    } else {
        let rows_per_block = MAX_BLOCK_DATA / row_bytes;
        subnodes.add_blocks(
            matrix
                .chunks(rows_per_block)
                .map(<[Vec<u8>]>::concat)
                .collect(),
        )
    };

    let mut index_records: Vec<Vec<u8>> = rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let mut record = row.row_id.to_le_bytes().to_vec();
            record.extend_from_slice(&(index as u32).to_le_bytes());
            record
        })
        .collect();
    index_records
        .sort_by_key(|record| u32::from_le_bytes([record[0], record[1], record[2], record[3]]));
    build_bth(&mut heap, index_hid, 4, &index_records);

    let mut info = vec![0u8; 22 + 8 * column_count];
    info[0] = 0x7c;
    info[1] = column_count as u8;
    for (slot, end) in ends.iter().enumerate() {
        put_u16(&mut info, 2 + slot * 2, *end as u16);
    }
    put_u32(&mut info, 10, index_hid);
    put_u32(&mut info, 14, rows_hnid);
    let mut descriptors: Vec<(u32, u16, u8, u8)> =
        vec![(PR_LTP_ROW_ID, 0, 4, 0), (PR_LTP_ROW_VER, 4, 4, 1)];
    for (position, (tag, offset, width, _)) in layout.iter().enumerate() {
        descriptors.push((*tag, *offset as u16, *width as u8, (position + 2) as u8));
    }
    descriptors.sort_by_key(|descriptor| descriptor.0);
    for (index, (tag, offset, width, bit)) in descriptors.iter().enumerate() {
        let start = 22 + index * 8;
        put_u32(&mut info, start, *tag);
        put_u16(&mut info, start + 4, *offset);
        info[start + 6] = *width;
        info[start + 7] = *bit;
    }
    heap.patch(info_hid, info);
    heap.finish()
}

// ---------------------------------------------------------------------------
// NDB emitter
// ---------------------------------------------------------------------------

struct Emitter<W: Write + Seek> {
    out: W,
    offset: u64,
    next_bid: u64,
    next_page_bid: u64,
    bbt: Vec<(u64, u64, u16)>,
    nbt: Vec<(u32, u64, u64, u32)>,
    garbage: Rng,
}

impl<W: Write + Seek> Emitter<W> {
    fn new(mut out: W, seed: u64) -> io::Result<Self> {
        out.seek(SeekFrom::Start(0))?;
        out.write_all(&vec![0u8; FIRST_BLOCK_OFFSET as usize])?;
        Ok(Self {
            out,
            offset: FIRST_BLOCK_OFFSET,
            next_bid: 4,
            next_page_bid: 4,
            bbt: Vec::new(),
            nbt: Vec::new(),
            garbage: Rng::new(seed ^ 0x6761_7262_6167_6521),
        })
    }

    fn allocate_bid(&mut self, internal: bool) -> u64 {
        let bid = self.next_bid | if internal { 2 } else { 0 };
        self.next_bid += 4;
        bid
    }

    fn write_block(
        &mut self,
        data: &[u8],
        internal: bool,
        flip_after_checksum: bool,
    ) -> io::Result<u64> {
        assert!(!data.is_empty() && data.len() <= MAX_BLOCK_DATA);
        let bid = self.allocate_bid(internal);
        let total = (data.len() + BLOCK_TRAILER_BYTES).div_ceil(64) * 64;
        let mut block = vec![0u8; total];
        block[..data.len()].copy_from_slice(data);
        let trailer = total - BLOCK_TRAILER_BYTES;
        put_u16(&mut block, trailer, data.len() as u16);
        put_u16(&mut block, trailer + 2, signature(self.offset, bid));
        put_u32(&mut block, trailer + 4, crc32(data));
        put_u64(&mut block, trailer + 8, bid);
        if flip_after_checksum {
            block[data.len() / 2] ^= 0xff;
        }
        self.out.write_all(&block)?;
        self.bbt.push((bid, self.offset, data.len() as u16));
        self.offset += total as u64;
        Ok(bid)
    }

    /// Writes an XBLOCK or XXBLOCK over `children` (block id, byte length).
    fn write_tree_level(&mut self, children: &[(u64, u64)], level: u8) -> io::Result<(u64, u64)> {
        let total: u64 = children.iter().map(|(_, bytes)| *bytes).sum();
        let mut data = vec![0u8; 8 + children.len() * 8];
        data[0] = 0x01;
        data[1] = level;
        put_u16(&mut data, 2, children.len() as u16);
        put_u32(&mut data, 4, total as u32);
        for (index, (bid, _)) in children.iter().enumerate() {
            put_u64(&mut data, 8 + index * 8, *bid);
        }
        Ok((self.write_block(&data, true, false)?, total))
    }

    fn write_data_tree(&mut self, leaves: &[(u64, u64)]) -> io::Result<u64> {
        if leaves.len() == 1 {
            return Ok(leaves[0].0);
        }
        let mut level_one = Vec::new();
        for chunk in leaves.chunks(XBLOCK_MAX_ENTRIES) {
            level_one.push(self.write_tree_level(chunk, 1)?);
        }
        if level_one.len() == 1 {
            return Ok(level_one[0].0);
        }
        assert!(level_one.len() <= XBLOCK_MAX_ENTRIES, "data tree too large");
        Ok(self.write_tree_level(&level_one, 2)?.0)
    }

    /// Writes a node's data blocks and subnode tree. Returns (data bid, subnode bid).
    fn write_node(&mut self, node: NodeData) -> io::Result<(u64, u64)> {
        let NodeData {
            mut blocks,
            children,
            fault,
        } = node;
        assert!(!blocks.is_empty(), "a node has at least one data block");
        if fault == Fault::Garbage {
            for block in &mut blocks {
                *block = self.garbage.bytes(block.len());
            }
        }
        let data_bid = if fault == Fault::MissingBlock {
            self.allocate_bid(false)
        } else {
            let mut leaves = Vec::with_capacity(blocks.len() + 1);
            for (index, block) in blocks.iter().enumerate() {
                let flip = fault == Fault::ChecksumMismatch && index == 0;
                leaves.push((self.write_block(block, false, flip)?, block.len() as u64));
            }
            if fault == Fault::TruncatedTree {
                leaves.push((self.allocate_bid(false), MAX_BLOCK_DATA as u64));
            }
            self.write_data_tree(&leaves)?
        };
        let sub_bid = self.write_subnodes(children)?;
        Ok((data_bid, sub_bid))
    }

    fn write_subnodes(&mut self, set: SubnodeSet) -> io::Result<u64> {
        if set.is_empty() {
            return Ok(0);
        }
        assert!(
            set.nodes.len() <= SLBLOCK_MAX_ENTRIES,
            "too many subnodes for one SLBLOCK"
        );
        let mut entries = Vec::with_capacity(set.nodes.len());
        for (nid, node) in set.nodes {
            let (data_bid, sub_bid) = self.write_node(node)?;
            entries.push((nid, data_bid, sub_bid));
        }
        entries.sort_by_key(|entry| entry.0);
        let mut data = vec![0u8; 8 + entries.len() * 24];
        data[0] = 0x02;
        put_u16(&mut data, 2, entries.len() as u16);
        for (index, (nid, data_bid, sub_bid)) in entries.iter().enumerate() {
            let start = 8 + index * 24;
            put_u64(&mut data, start, u64::from(*nid));
            put_u64(&mut data, start + 8, *data_bid);
            put_u64(&mut data, start + 16, *sub_bid);
        }
        self.write_block(&data, true, false)
    }

    /// Writes a top-level node and registers it in the node index.
    fn add_node(&mut self, nid: u32, parent: u32, node: NodeData) -> io::Result<()> {
        let (data_bid, sub_bid) = self.write_node(node)?;
        self.nbt.push((nid, data_bid, sub_bid, parent));
        Ok(())
    }

    fn write_page(
        &mut self,
        entries: &[u8],
        count: usize,
        max: usize,
        entry_bytes: usize,
        level: u8,
        page_type: u8,
    ) -> io::Result<(u64, u64)> {
        let bid = self.next_page_bid;
        self.next_page_bid += 4;
        let mut page = [0u8; PAGE_SIZE];
        page[..entries.len()].copy_from_slice(entries);
        page[488] = count as u8;
        page[489] = max as u8;
        page[490] = entry_bytes as u8;
        page[491] = level;
        page[496] = page_type;
        page[497] = page_type;
        put_u16(&mut page, 498, signature(self.offset, bid));
        let crc = crc32(&page[..496]);
        put_u32(&mut page, 500, crc);
        put_u64(&mut page, 504, bid);
        self.out.write_all(&page)?;
        let offset = self.offset;
        self.offset += PAGE_SIZE as u64;
        Ok((bid, offset))
    }

    /// Writes a B-tree over sorted leaf entries (key, encoded entry). Returns its root.
    fn write_btree(
        &mut self,
        leaves: &[(u64, Vec<u8>)],
        leaf_entry_bytes: usize,
        page_type: u8,
    ) -> io::Result<(u64, u64)> {
        let leaf_max = BT_ENTRY_AREA_BYTES / leaf_entry_bytes;
        let mut level: Vec<(u64, u64, u64)> = Vec::new(); // first key, page bid, page offset
        if leaves.is_empty() {
            return self.write_page(&[], 0, leaf_max, leaf_entry_bytes, 0, page_type);
        }
        for chunk in leaves.chunks(leaf_max) {
            let bytes: Vec<u8> = chunk.iter().flat_map(|(_, entry)| entry.clone()).collect();
            let (bid, offset) = self.write_page(
                &bytes,
                chunk.len(),
                leaf_max,
                leaf_entry_bytes,
                0,
                page_type,
            )?;
            level.push((chunk[0].0, bid, offset));
        }
        let node_max = BT_ENTRY_AREA_BYTES / BT_INTERMEDIATE_ENTRY_BYTES;
        let mut depth = 1u8;
        while level.len() > 1 {
            let mut next = Vec::new();
            for chunk in level.chunks(node_max) {
                let mut bytes = vec![0u8; chunk.len() * BT_INTERMEDIATE_ENTRY_BYTES];
                for (index, (key, bid, offset)) in chunk.iter().enumerate() {
                    let start = index * BT_INTERMEDIATE_ENTRY_BYTES;
                    put_u64(&mut bytes, start, *key);
                    put_u64(&mut bytes, start + 8, *bid);
                    put_u64(&mut bytes, start + 16, *offset);
                }
                let (bid, offset) = self.write_page(
                    &bytes,
                    chunk.len(),
                    node_max,
                    BT_INTERMEDIATE_ENTRY_BYTES,
                    depth,
                    page_type,
                )?;
                next.push((chunk[0].0, bid, offset));
            }
            level = next;
            depth += 1;
        }
        Ok((level[0].1, level[0].2))
    }

    /// Writes both indexes and the header, and returns the file length.
    fn finish(mut self) -> io::Result<u64> {
        let padding = (PAGE_SIZE as u64 - self.offset % PAGE_SIZE as u64) % PAGE_SIZE as u64;
        self.out.write_all(&vec![0u8; padding as usize])?;
        self.offset += padding;

        self.nbt.sort_by_key(|entry| entry.0);
        for pair in self.nbt.windows(2) {
            assert_ne!(pair[0].0, pair[1].0, "duplicate node id {:#x}", pair[0].0);
        }
        let nbt_leaves: Vec<(u64, Vec<u8>)> = self
            .nbt
            .iter()
            .map(|(nid, data_bid, sub_bid, parent)| {
                let mut entry = vec![0u8; NBT_LEAF_ENTRY_BYTES];
                put_u64(&mut entry, 0, u64::from(*nid));
                put_u64(&mut entry, 8, *data_bid);
                put_u64(&mut entry, 16, *sub_bid);
                put_u32(&mut entry, 24, *parent);
                (u64::from(*nid), entry)
            })
            .collect();
        let (nbt_bid, nbt_offset) =
            self.write_btree(&nbt_leaves, NBT_LEAF_ENTRY_BYTES, PAGE_TYPE_NBT)?;

        self.bbt.sort_by_key(|entry| entry.0);
        let bbt_leaves: Vec<(u64, Vec<u8>)> = self
            .bbt
            .iter()
            .map(|(bid, offset, bytes)| {
                let mut entry = vec![0u8; BBT_LEAF_ENTRY_BYTES];
                put_u64(&mut entry, 0, *bid);
                put_u64(&mut entry, 8, *offset);
                put_u16(&mut entry, 16, *bytes);
                put_u16(&mut entry, 18, 2);
                (*bid, entry)
            })
            .collect();
        let (bbt_bid, bbt_offset) =
            self.write_btree(&bbt_leaves, BBT_LEAF_ENTRY_BYTES, PAGE_TYPE_BBT)?;

        let mut header = vec![0u8; HEADER_BYTES];
        header[0..4].copy_from_slice(b"!BDN");
        header[8..10].copy_from_slice(b"SM");
        put_u16(&mut header, 10, 23);
        put_u16(&mut header, 12, 19);
        header[14] = 1;
        header[15] = 1;
        put_u64(&mut header, 32, self.next_page_bid);
        put_u32(&mut header, 40, 1);
        for slot in 0..32 {
            put_u32(&mut header, 44 + slot * 4, 0x0010_0000);
        }
        put_u64(&mut header, 184, self.offset);
        put_u64(&mut header, 216, nbt_bid);
        put_u64(&mut header, 224, nbt_offset);
        put_u64(&mut header, 232, bbt_bid);
        put_u64(&mut header, 240, bbt_offset);
        header[248] = 0; // allocation maps are not written
        header[256..512].fill(0xff);
        header[512] = 0x80;
        header[513] = 0; // no data block encoding
        put_u64(&mut header, 516, self.next_bid);
        let partial = crc32(&header[8..8 + 471]);
        put_u32(&mut header, 4, partial);
        let full = crc32(&header[8..8 + 516]);
        put_u32(&mut header, 524, full);
        self.out.seek(SeekFrom::Start(0))?;
        self.out.write_all(&header)?;
        self.out.flush()?;
        Ok(self.offset)
    }
}

// ---------------------------------------------------------------------------
// Mailbox model
// ---------------------------------------------------------------------------

/// Message shapes that exercise a specific reader boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    Regular,
    NoSubject,
    EmptyBody,
    UnicodeSubject,
    LongSubject,
    HtmlOnly,
    LargeBody,
    ManyRecipients,
    NoRecipients,
    EmbeddedMessage,
    DuplicateMessageId,
    OrphanMessage,
    ZeroLengthAttachment,
    LargeAttachment,
    ManyAttachments,
    AttachmentMissingBlock,
    AttachmentTruncatedTree,
    AttachmentGarbageProperties,
    AttachmentChecksumMismatch,
    AttachmentDanglingRow,
    MessageMissingBlock,
    MessageGarbageProperties,
    DanglingContentsRow,
    Contact,
    Appointment,
}

pub const SPECIAL_SCENARIOS: &[Scenario] = &[
    Scenario::NoSubject,
    Scenario::EmptyBody,
    Scenario::UnicodeSubject,
    Scenario::LongSubject,
    Scenario::HtmlOnly,
    Scenario::LargeBody,
    Scenario::ManyRecipients,
    Scenario::NoRecipients,
    Scenario::EmbeddedMessage,
    Scenario::DuplicateMessageId,
    Scenario::OrphanMessage,
    Scenario::ZeroLengthAttachment,
    Scenario::LargeAttachment,
    Scenario::ManyAttachments,
    Scenario::AttachmentMissingBlock,
    Scenario::AttachmentTruncatedTree,
    Scenario::AttachmentGarbageProperties,
    Scenario::AttachmentChecksumMismatch,
    Scenario::AttachmentDanglingRow,
    Scenario::MessageMissingBlock,
    Scenario::MessageGarbageProperties,
    Scenario::DanglingContentsRow,
    Scenario::Contact,
    Scenario::Appointment,
];

impl Scenario {
    pub fn label(self) -> &'static str {
        match self {
            Scenario::Regular => "regular",
            Scenario::NoSubject => "no_subject",
            Scenario::EmptyBody => "empty_body",
            Scenario::UnicodeSubject => "unicode_subject",
            Scenario::LongSubject => "long_subject",
            Scenario::HtmlOnly => "html_only",
            Scenario::LargeBody => "large_body",
            Scenario::ManyRecipients => "many_recipients",
            Scenario::NoRecipients => "no_recipients",
            Scenario::EmbeddedMessage => "embedded_message",
            Scenario::DuplicateMessageId => "duplicate_message_id",
            Scenario::OrphanMessage => "orphan_message",
            Scenario::ZeroLengthAttachment => "zero_length_attachment",
            Scenario::LargeAttachment => "large_attachment",
            Scenario::ManyAttachments => "many_attachments",
            Scenario::AttachmentMissingBlock => "attachment_missing_block",
            Scenario::AttachmentTruncatedTree => "attachment_truncated_tree",
            Scenario::AttachmentGarbageProperties => "attachment_garbage_properties",
            Scenario::AttachmentChecksumMismatch => "attachment_checksum_mismatch",
            Scenario::AttachmentDanglingRow => "attachment_dangling_row",
            Scenario::MessageMissingBlock => "message_missing_block",
            Scenario::MessageGarbageProperties => "message_garbage_properties",
            Scenario::DanglingContentsRow => "dangling_contents_row",
            Scenario::Contact => "contact",
            Scenario::Appointment => "appointment",
        }
    }

    /// Whether the message's own property context is deliberately unreadable.
    pub fn message_fault(self) -> Fault {
        match self {
            Scenario::MessageMissingBlock => Fault::MissingBlock,
            Scenario::MessageGarbageProperties => Fault::Garbage,
            _ => Fault::None,
        }
    }
}

/// Folder roles. The first few are the standard mailbox folders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderRole {
    Inbox,
    Drafts,
    SentItems,
    DeletedItems,
    Archive,
    ArchiveYear,
    Projects,
    Contacts,
    Calendar,
}

struct FolderSpec {
    role: Option<FolderRole>,
    name: &'static str,
    index: u32,
    parent: Option<usize>,
}

const FOLDER_ROOT: usize = 0;
const FOLDER_IPM_SUBTREE: usize = 1;
const FOLDER_DELETED: usize = 3;

const FOLDERS: &[FolderSpec] = &[
    FolderSpec {
        role: None,
        name: "",
        index: 0x09,
        parent: None,
    },
    FolderSpec {
        role: None,
        name: "Top of Personal Folders",
        index: 0x401,
        parent: Some(0),
    },
    FolderSpec {
        role: None,
        name: "Search Root",
        index: 0x402,
        parent: Some(0),
    },
    FolderSpec {
        role: Some(FolderRole::DeletedItems),
        name: "Deleted Items",
        index: 0x403,
        parent: Some(1),
    },
    FolderSpec {
        role: Some(FolderRole::Inbox),
        name: "Inbox",
        index: 0x404,
        parent: Some(1),
    },
    FolderSpec {
        role: Some(FolderRole::Drafts),
        name: "Drafts",
        index: 0x405,
        parent: Some(1),
    },
    FolderSpec {
        role: Some(FolderRole::SentItems),
        name: "Sent Items",
        index: 0x406,
        parent: Some(1),
    },
    FolderSpec {
        role: Some(FolderRole::Archive),
        name: "Archive",
        index: 0x407,
        parent: Some(1),
    },
    FolderSpec {
        role: Some(FolderRole::ArchiveYear),
        name: "2023",
        index: 0x408,
        parent: Some(7),
    },
    FolderSpec {
        role: Some(FolderRole::Projects),
        name: "Projekte \u{00fc}ber \u{540d}\u{524d} \u{2709}",
        index: 0x409,
        parent: Some(7),
    },
    FolderSpec {
        role: None,
        name: "Empty Folder",
        index: 0x40a,
        parent: Some(1),
    },
    FolderSpec {
        role: Some(FolderRole::Contacts),
        name: "Contacts",
        index: 0x40b,
        parent: Some(1),
    },
    FolderSpec {
        role: Some(FolderRole::Calendar),
        name: "Calendar",
        index: 0x40c,
        parent: Some(1),
    },
];

fn folder_nid(folder: usize) -> u32 {
    (FOLDERS[folder].index << 5) | NID_TYPE_NORMAL_FOLDER
}

fn folder_path(folder: usize) -> String {
    let mut parts = Vec::new();
    let mut current = Some(folder);
    while let Some(index) = current {
        if !FOLDERS[index].name.is_empty() {
            parts.push(FOLDERS[index].name);
        }
        current = FOLDERS[index].parent;
    }
    parts.reverse();
    parts.join("/")
}

/// What a reader should recover for one attachment.
#[derive(Debug, Clone)]
pub struct ManifestAttachment {
    pub filename: String,
    pub size: usize,
    pub fnv1a64: u64,
    pub fault: Fault,
    /// The table row exists but the attachment node does not.
    pub dangling_row: bool,
    pub embedded_message: bool,
}

/// What a reader should recover for one message.
#[derive(Debug, Clone)]
pub struct ManifestMessage {
    pub index: usize,
    pub node_id: u32,
    pub folder: String,
    pub scenario: Scenario,
    pub message_class: String,
    pub subject: String,
    pub body_text: String,
    pub has_html: bool,
    pub draft: bool,
    pub deleted: bool,
    pub recipient_count: usize,
    pub internet_message_id: String,
    pub in_contents_table: bool,
    pub attachments: Vec<ManifestAttachment>,
}

impl ManifestMessage {
    /// False when the message's own node is deliberately unreadable.
    pub fn recoverable(&self) -> bool {
        self.scenario.message_fault() == Fault::None
    }
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub seed: u64,
    pub file_bytes: u64,
    pub folders: Vec<String>,
    pub messages: Vec<ManifestMessage>,
    /// Contents-table row ids that reference no message node.
    pub dangling_contents_rows: Vec<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub seed: u64,
    pub messages: usize,
}

struct BuiltMessage {
    node: NodeData,
    manifest: ManifestMessage,
    flags: u32,
    size: u32,
    delivery_time: u64,
}

fn scenario_for(index: usize) -> Scenario {
    if index < SPECIAL_SCENARIOS.len() {
        SPECIAL_SCENARIOS[index]
    } else if index % 16 == 5 {
        SPECIAL_SCENARIOS[(index / 16) % SPECIAL_SCENARIOS.len()]
    } else {
        Scenario::Regular
    }
}

fn folder_for(index: usize, scenario: Scenario, rng: &mut Rng) -> usize {
    let role = match scenario {
        Scenario::Contact => FolderRole::Contacts,
        Scenario::Appointment => FolderRole::Calendar,
        _ => match (rng.below(100), index % 7) {
            (0..=49, _) => FolderRole::Inbox,
            (50..=69, _) => FolderRole::SentItems,
            (70..=81, _) => FolderRole::DeletedItems,
            (82..=87, _) => FolderRole::Drafts,
            (88..=93, _) => FolderRole::ArchiveYear,
            _ => FolderRole::Projects,
        },
    };
    FOLDERS
        .iter()
        .position(|folder| folder.role == Some(role))
        .expect("role has a folder")
}

fn attachment_bytes(seed: u64, message: usize, attachment: usize, len: usize) -> Vec<u8> {
    Rng::new(seed ^ ((message as u64) << 20) ^ ((attachment as u64) << 4) ^ 0xa77a).bytes(len)
}

struct AttachmentPlan {
    filename: String,
    bytes: Vec<u8>,
    fault: Fault,
    dangling_row: bool,
    embedded: bool,
}

fn build_attachment(plan: &AttachmentPlan, embedded: Option<NodeData>) -> NodeData {
    let mut children = SubnodeSet::new();
    let mut props = vec![
        prop_i32(PR_ATTACH_SIZE, plan.bytes.len() as i32),
        prop_str(PR_ATTACH_FILENAME, &plan.filename),
        prop_str(PR_ATTACH_LONG_FILENAME, &plan.filename),
        prop_i32(PR_RENDERING_POSITION, -1),
    ];
    if let Some(message) = embedded {
        let nid = children.next_nid(NID_TYPE_NORMAL_MESSAGE);
        children.add(nid, message);
        let mut object = nid.to_le_bytes().to_vec();
        // ulSize: the nominal size of the embedded object; readers reject zero.
        object.extend_from_slice(&EMBEDDED_OBJECT_NOMINAL_BYTES.to_le_bytes());
        props.push(Prop {
            tag: PR_ATTACH_DATA_OBJ,
            value: object,
        });
        props.push(prop_i32(PR_ATTACH_METHOD, 5));
        props.push(prop_str(PR_ATTACH_MIME_TAG, "message/rfc822"));
    } else {
        props.push(prop_i32(PR_ATTACH_METHOD, 1));
        props.push(prop_str(PR_ATTACH_MIME_TAG, "application/octet-stream"));
        // The data stream carries its own fault so the attachment metadata stays valid.
        let data_fault = match plan.fault {
            Fault::Garbage => Fault::None,
            other => other,
        };
        if data_fault == Fault::None && plan.bytes.len() <= HEAP_MAX_ALLOCATION {
            props.push(prop_bin(PR_ATTACH_DATA_BIN, &plan.bytes));
        } else {
            // A faulty or large payload lives in its own subnode stream.
            let nid = children.next_nid(NID_TYPE_LTP);
            let blocks: Vec<Vec<u8>> = if plan.bytes.is_empty() {
                vec![vec![0u8]]
            } else {
                plan.bytes
                    .chunks(MAX_BLOCK_DATA)
                    .map(<[u8]>::to_vec)
                    .collect()
            };
            let mut stream = NodeData::new(blocks, SubnodeSet::default());
            stream.fault = data_fault;
            children.add(nid, stream);
            return finish_attachment(props, children, plan.fault, Some(nid));
        }
    }
    finish_attachment(props, children, plan.fault, None)
}

/// Builds the attachment property context. With `stream_nid`, the data property holds
/// that subnode id instead of an inline heap value.
fn finish_attachment(
    props: Vec<Prop>,
    children: SubnodeSet,
    fault: Fault,
    stream_nid: Option<u32>,
) -> NodeData {
    let reference = stream_nid.map(|nid| (PR_ATTACH_DATA_BIN, nid));
    let mut overflow = SubnodeSet::default();
    let blocks = build_property_context_with_reference(props, reference, &mut overflow);
    assert!(
        overflow.is_empty(),
        "attachment metadata values fit in the heap"
    );
    let mut node = NodeData::new(blocks, children);
    if fault == Fault::Garbage {
        node.fault = Fault::Garbage;
    }
    node
}

fn build_message(
    seed: u64,
    index: usize,
    scenario: Scenario,
    folder: usize,
    depth: usize,
) -> BuiltMessage {
    let mut rng =
        Rng::new(seed ^ (index as u64).wrapping_mul(0x9e37_79b9) ^ ((depth as u64) << 56));
    let role = FOLDERS[folder].role;
    let draft = role == Some(FolderRole::Drafts);
    let deleted = role == Some(FolderRole::DeletedItems);
    let sent = role == Some(FolderRole::SentItems);

    let message_class = match scenario {
        Scenario::Contact => "IPM.Contact",
        Scenario::Appointment => "IPM.Appointment",
        _ => "IPM.Note",
    };
    let subject = match scenario {
        Scenario::NoSubject => String::new(),
        Scenario::UnicodeSubject => {
            format!("\u{00dc}bersicht \u{2014} \u{8a66}\u{9a13} \u{1f4e7} #{index}")
        }
        Scenario::LongSubject => format!("{} #{index}", words(&mut rng, 150)),
        _ => {
            let count = 3 + rng.below(4) as usize;
            format!("{} #{index}", words(&mut rng, count))
        }
    };
    let body_text = match scenario {
        Scenario::EmptyBody | Scenario::HtmlOnly => String::new(),
        Scenario::LargeBody => {
            let mut body = String::new();
            while body.len() < 60_000 {
                body.push_str(&words(&mut rng, 12));
                body.push_str("\r\n");
            }
            body
        }
        _ => {
            let lines = 2 + rng.below(8) as usize;
            let mut body = String::new();
            for _ in 0..lines {
                let count = 6 + rng.below(10) as usize;
                body.push_str(&words(&mut rng, count));
                body.push_str("\r\n");
            }
            body
        }
    };
    let has_html = scenario == Scenario::HtmlOnly
        || (scenario == Scenario::Regular && index.is_multiple_of(5));
    let html = format!(
        "<html><body><p>{}</p></body></html>",
        if body_text.is_empty() {
            format!("html body {index}")
        } else {
            body_text.replace("\r\n", "<br>")
        }
    );

    let (sender_name, sender_email) = person(&mut rng);
    let recipient_count = match scenario {
        _ if draft => 0,
        Scenario::NoRecipients | Scenario::Contact | Scenario::Appointment => 0,
        Scenario::ManyRecipients => 300,
        _ => 1 + rng.below(4) as usize,
    };
    let recipients: Vec<(String, String, i32)> = (0..recipient_count)
        .map(|position| {
            let (name, email) = person(&mut rng);
            let kind = if position == 0 {
                1
            } else {
                1 + (rng.below(3) as i32).min(1)
            };
            (
                format!("{name} {position}"),
                format!("r{position}.{email}"),
                kind,
            )
        })
        .collect();
    let internet_message_id = match scenario {
        Scenario::DuplicateMessageId => "<duplicate.synthetic@example.com>".to_string(),
        _ => format!("<synthetic.{seed}.{index}.{depth}@example.com>"),
    };
    let time = BASE_FILETIME + (index as u64) * 7 * FILETIME_MINUTE;

    // Attachments.
    let mut plans: Vec<AttachmentPlan> = Vec::new();
    let plan = |name: String,
                len: usize,
                fault: Fault,
                dangling: bool,
                embedded: bool,
                plans: &mut Vec<AttachmentPlan>| {
        let position = plans.len();
        plans.push(AttachmentPlan {
            filename: name,
            bytes: attachment_bytes(seed, index, position, len),
            fault,
            dangling_row: dangling,
            embedded,
        });
    };
    match scenario {
        Scenario::ZeroLengthAttachment => {
            plan("empty.bin".into(), 0, Fault::None, false, false, &mut plans)
        }
        Scenario::LargeAttachment => plan(
            "large.bin".into(),
            300_000,
            Fault::None,
            false,
            false,
            &mut plans,
        ),
        Scenario::ManyAttachments => {
            for position in 0..25 {
                plan(
                    format!("part-{position:02}.bin"),
                    200 + position * 37,
                    Fault::None,
                    false,
                    false,
                    &mut plans,
                );
            }
        }
        Scenario::AttachmentMissingBlock => {
            plan(
                "intact.bin".into(),
                900,
                Fault::None,
                false,
                false,
                &mut plans,
            );
            plan(
                "missing-block.bin".into(),
                5_000,
                Fault::MissingBlock,
                false,
                false,
                &mut plans,
            );
        }
        Scenario::AttachmentTruncatedTree => {
            plan(
                "intact.bin".into(),
                900,
                Fault::None,
                false,
                false,
                &mut plans,
            );
            plan(
                "truncated-tree.bin".into(),
                20_000,
                Fault::TruncatedTree,
                false,
                false,
                &mut plans,
            );
        }
        Scenario::AttachmentGarbageProperties => {
            plan(
                "intact.bin".into(),
                900,
                Fault::None,
                false,
                false,
                &mut plans,
            );
            plan(
                "garbage-properties.bin".into(),
                700,
                Fault::Garbage,
                false,
                false,
                &mut plans,
            );
        }
        Scenario::AttachmentChecksumMismatch => {
            plan(
                "intact.bin".into(),
                900,
                Fault::None,
                false,
                false,
                &mut plans,
            );
            plan(
                "checksum-mismatch.bin".into(),
                6_000,
                Fault::ChecksumMismatch,
                false,
                false,
                &mut plans,
            );
        }
        Scenario::AttachmentDanglingRow => {
            plan(
                "intact.bin".into(),
                900,
                Fault::None,
                false,
                false,
                &mut plans,
            );
            plan(
                "dangling-row.bin".into(),
                400,
                Fault::None,
                true,
                false,
                &mut plans,
            );
        }
        Scenario::EmbeddedMessage if depth == 0 => {
            plan(
                "embedded.msg".into(),
                0,
                Fault::None,
                false,
                true,
                &mut plans,
            );
        }
        Scenario::Regular if !draft && index % 5 == 2 => {
            let len = 200 + rng.below(6_000) as usize;
            plan(
                format!("document-{index}.bin"),
                len,
                Fault::None,
                false,
                false,
                &mut plans,
            );
            if index % 25 == 2 {
                plan(
                    format!("scan-{index}.bin"),
                    9_000 + rng.below(40_000) as usize,
                    Fault::None,
                    false,
                    false,
                    &mut plans,
                );
            }
        }
        _ => {}
    }

    let mut flags = if draft {
        MSGFLAG_UNSENT
    } else if index.is_multiple_of(3) {
        0
    } else {
        MSGFLAG_READ
    };
    if !plans.is_empty() {
        flags |= MSGFLAG_HASATTACH;
    }

    let mut children = SubnodeSet::new();
    let mut props = vec![
        prop_str(PR_MESSAGE_CLASS, message_class),
        prop_i32(PR_IMPORTANCE, 1),
        prop_i32(PR_MESSAGE_FLAGS, flags as i32),
        prop_bool(PR_HASATTACH, !plans.is_empty()),
        prop_time(PR_CREATION_TIME, time),
        prop_time(PR_LAST_MODIFICATION_TIME, time + FILETIME_MINUTE),
        prop_str(PR_INTERNET_MESSAGE_ID, &internet_message_id),
    ];
    if !subject.is_empty() {
        props.push(prop_str(PR_SUBJECT, &subject));
    }
    if !body_text.is_empty() {
        props.push(prop_str(PR_BODY, &body_text));
    }
    if has_html {
        props.push(prop_bin(PR_HTML, html.as_bytes()));
    }
    if !draft {
        props.push(prop_str(PR_SENDER_NAME, &sender_name));
        props.push(prop_str(PR_SENDER_EMAIL_ADDRESS, &sender_email));
        props.push(prop_str(PR_SENDER_ADDRTYPE, "SMTP"));
        props.push(prop_str(PR_SENT_REPRESENTING_NAME, &sender_name));
        props.push(prop_time(PR_CLIENT_SUBMIT_TIME, time));
        props.push(prop_time(PR_MESSAGE_DELIVERY_TIME, time + FILETIME_MINUTE));
        if !sent {
            props.push(prop_str(
                PR_TRANSPORT_HEADERS,
                &format!("Message-ID: {internet_message_id}\r\nFrom: {sender_name} <{sender_email}>\r\nSubject: {subject}\r\n"),
            ));
        }
    }
    let display_to: Vec<&str> = recipients
        .iter()
        .filter(|recipient| recipient.2 == 1)
        .take(20)
        .map(|recipient| recipient.0.as_str())
        .collect();
    if !display_to.is_empty() {
        props.push(prop_str(PR_DISPLAY_TO, &display_to.join("; ")));
    }
    let display_cc: Vec<&str> = recipients
        .iter()
        .filter(|recipient| recipient.2 == 2)
        .take(20)
        .map(|recipient| recipient.0.as_str())
        .collect();
    if !display_cc.is_empty() {
        props.push(prop_str(PR_DISPLAY_CC, &display_cc.join("; ")));
    }
    let size = (body_text.len() * 2
        + plans.iter().map(|plan| plan.bytes.len()).sum::<usize>()
        + 512) as u32;
    props.push(prop_i32(PR_MESSAGE_SIZE, size as i32));

    // Recipient table (present on every message except the no-recipients shapes).
    if scenario != Scenario::NoRecipients || draft {
        let columns = [
            PR_RECIPIENT_TYPE,
            PR_RESPONSIBILITY,
            PR_OBJECT_TYPE,
            PR_DISPLAY_NAME,
            PR_ADDRTYPE,
            PR_EMAIL_ADDRESS,
            PR_DISPLAY_TYPE,
            PR_SMTP_ADDRESS,
        ];
        let rows: Vec<TableRow> = recipients
            .iter()
            .enumerate()
            .map(|(position, (name, email, kind))| TableRow {
                row_id: position as u32,
                values: vec![
                    Some(kind.to_le_bytes().to_vec()),
                    Some(vec![1]),
                    Some(6i32.to_le_bytes().to_vec()),
                    Some(utf16(name)),
                    Some(utf16("SMTP")),
                    Some(utf16(email)),
                    Some(0i32.to_le_bytes().to_vec()),
                    Some(utf16(email)),
                ],
            })
            .collect();
        let mut table_children = SubnodeSet::new();
        let blocks = build_table_context(&columns, &rows, &mut table_children);
        children.add(NID_RECIPIENT_TABLE, NodeData::new(blocks, table_children));
    }

    // Attachment table and attachment nodes.
    let mut manifest_attachments = Vec::new();
    if !plans.is_empty() {
        let columns = [
            PR_ATTACH_SIZE,
            PR_ATTACH_FILENAME,
            PR_ATTACH_METHOD,
            PR_RENDERING_POSITION,
        ];
        let mut rows = Vec::new();
        for (position, attachment) in plans.iter().enumerate() {
            let nid = children.next_nid(NID_TYPE_ATTACHMENT);
            rows.push(TableRow {
                row_id: nid,
                values: vec![
                    Some((attachment.bytes.len() as i32).to_le_bytes().to_vec()),
                    Some(utf16(&attachment.filename)),
                    Some(
                        (if attachment.embedded { 5i32 } else { 1i32 })
                            .to_le_bytes()
                            .to_vec(),
                    ),
                    Some((-1i32).to_le_bytes().to_vec()),
                ],
            });
            let embedded = attachment.embedded.then(|| {
                build_message(
                    seed,
                    index * 1000 + position + 1,
                    Scenario::Regular,
                    folder,
                    depth + 1,
                )
                .node
            });
            if !attachment.dangling_row {
                children.add(nid, build_attachment(attachment, embedded));
            }
            manifest_attachments.push(ManifestAttachment {
                filename: attachment.filename.clone(),
                size: attachment.bytes.len(),
                fnv1a64: fnv1a64(&attachment.bytes),
                fault: attachment.fault,
                dangling_row: attachment.dangling_row,
                embedded_message: attachment.embedded,
            });
        }
        let mut table_children = SubnodeSet::new();
        let blocks = build_table_context(&columns, &rows, &mut table_children);
        children.add(NID_ATTACHMENT_TABLE, NodeData::new(blocks, table_children));
    }

    let blocks = build_property_context(props, &mut children);
    let mut node = NodeData::new(blocks, children);
    node.fault = scenario.message_fault();

    BuiltMessage {
        node,
        manifest: ManifestMessage {
            index,
            node_id: 0,
            folder: folder_path(folder),
            scenario,
            message_class: message_class.to_string(),
            subject,
            body_text,
            has_html,
            draft,
            deleted,
            recipient_count,
            internet_message_id,
            in_contents_table: scenario != Scenario::OrphanMessage,
            attachments: manifest_attachments,
        },
        flags,
        size,
        delivery_time: time + FILETIME_MINUTE,
    }
}

struct ContentsRow {
    row_id: u32,
    subject: String,
    flags: u32,
    size: u32,
    delivery_time: u64,
}

fn entry_id(nid: u32) -> Vec<u8> {
    let mut bytes = vec![0u8; 4];
    bytes.extend_from_slice(b"SYNTHETIC-PST-01");
    bytes.extend_from_slice(&nid.to_le_bytes());
    bytes
}

/// Writes a synthetic mailbox to `out` and returns what a reader should recover.
pub fn generate<W: Write + Seek>(out: W, config: Config) -> io::Result<Manifest> {
    let mut emitter = Emitter::new(out, config.seed)?;
    let mut contents: Vec<Vec<ContentsRow>> = FOLDERS.iter().map(|_| Vec::new()).collect();
    let mut messages = Vec::with_capacity(config.messages);
    let mut dangling_contents_rows = Vec::new();
    let mut folder_rng = Rng::new(config.seed ^ 0xf01d_e125);

    for index in 0..config.messages {
        let scenario = scenario_for(index);
        let folder = folder_for(index, scenario, &mut folder_rng);
        let built = build_message(config.seed, index, scenario, folder, 0);
        let nid = ((0x10000 + index as u32) << 5) | NID_TYPE_NORMAL_MESSAGE;
        let mut manifest = built.manifest;
        manifest.node_id = nid;
        if manifest.in_contents_table {
            contents[folder].push(ContentsRow {
                row_id: nid,
                subject: manifest.subject.clone(),
                flags: built.flags,
                size: built.size,
                delivery_time: built.delivery_time,
            });
        }
        if scenario == Scenario::DanglingContentsRow {
            // A row whose node id is never written to the node index.
            let ghost = ((0x0100_0000 + index as u32) << 5) | NID_TYPE_NORMAL_MESSAGE;
            contents[folder].push(ContentsRow {
                row_id: ghost,
                subject: format!("ghost row {index}"),
                flags: MSGFLAG_READ,
                size: 0,
                delivery_time: built.delivery_time,
            });
            dangling_contents_rows.push(ghost);
        }
        emitter.add_node(nid, folder_nid(folder), built.node)?;
        messages.push(manifest);
    }

    // Message store.
    let mut store_children = SubnodeSet::new();
    let store = build_property_context(
        vec![
            prop_bin(PR_RECORD_KEY, b"SYNTHETIC-PST-01"),
            prop_str(PR_DISPLAY_NAME, "Synthetic Mailbox"),
            prop_bin(
                PR_IPM_SUBTREE_ENTRYID,
                &entry_id(folder_nid(FOLDER_IPM_SUBTREE)),
            ),
            prop_bin(
                PR_IPM_WASTEBASKET_ENTRYID,
                &entry_id(folder_nid(FOLDER_DELETED)),
            ),
            prop_bin(PR_FINDER_ENTRYID, &entry_id(folder_nid(2))),
            prop_i32(PR_VALID_FOLDER_MASK, 0x89),
            prop_i32(PR_PST_PASSWORD, 0),
        ],
        &mut store_children,
    );
    emitter.add_node(NID_MESSAGE_STORE, 0, NodeData::new(store, store_children))?;

    // Named property map with no named properties.
    let mut map_children = SubnodeSet::new();
    let map = build_property_context(
        vec![
            prop_i32(PR_NAMEID_BUCKET_COUNT, 251),
            prop_bin(PR_NAMEID_STREAM_GUID, &[]),
            prop_bin(PR_NAMEID_STREAM_ENTRY, &[]),
            prop_bin(PR_NAMEID_STREAM_STRING, &[]),
        ],
        &mut map_children,
    );
    emitter.add_node(NID_NAME_TO_ID_MAP, 0, NodeData::new(map, map_children))?;

    // Folders with their hierarchy, contents and associated-contents tables.
    for (folder, spec) in FOLDERS.iter().enumerate() {
        let nid = folder_nid(folder);
        let parent = spec.parent.map_or(nid, folder_nid);
        let child_folders: Vec<usize> = (0..FOLDERS.len())
            .filter(|candidate| FOLDERS[*candidate].parent == Some(folder))
            .collect();
        let rows = &contents[folder];
        let unread = rows
            .iter()
            .filter(|row| row.flags & MSGFLAG_READ == 0)
            .count();

        let mut children = SubnodeSet::new();
        let pc = build_property_context(
            vec![
                prop_str(PR_DISPLAY_NAME, spec.name),
                prop_i32(PR_CONTENT_COUNT, rows.len() as i32),
                prop_i32(PR_CONTENT_UNREAD, unread as i32),
                prop_bool(PR_SUBFOLDERS, !child_folders.is_empty()),
            ],
            &mut children,
        );
        emitter.add_node(nid, parent, NodeData::new(pc, children))?;

        let hierarchy_rows: Vec<TableRow> = child_folders
            .iter()
            .map(|child| TableRow {
                row_id: folder_nid(*child),
                values: vec![
                    Some(utf16(FOLDERS[*child].name)),
                    Some((contents[*child].len() as i32).to_le_bytes().to_vec()),
                    Some(0i32.to_le_bytes().to_vec()),
                    Some(vec![u8::from(
                        FOLDERS.iter().any(|other| other.parent == Some(*child)),
                    )]),
                ],
            })
            .collect();
        let mut table_children = SubnodeSet::new();
        let table = build_table_context(
            &[
                PR_DISPLAY_NAME,
                PR_CONTENT_COUNT,
                PR_CONTENT_UNREAD,
                PR_SUBFOLDERS,
            ],
            &hierarchy_rows,
            &mut table_children,
        );
        emitter.add_node(
            (spec.index << 5) | NID_TYPE_HIERARCHY_TABLE,
            0,
            NodeData::new(table, table_children),
        )?;

        let contents_rows: Vec<TableRow> = rows
            .iter()
            .map(|row| TableRow {
                row_id: row.row_id,
                values: vec![
                    Some(1i32.to_le_bytes().to_vec()),
                    Some(utf16(&row.subject)),
                    Some(row.delivery_time.to_le_bytes().to_vec()),
                    Some((row.flags as i32).to_le_bytes().to_vec()),
                    Some((row.size as i32).to_le_bytes().to_vec()),
                ],
            })
            .collect();
        let mut table_children = SubnodeSet::new();
        let table = build_table_context(
            &[
                PR_IMPORTANCE,
                PR_SUBJECT,
                PR_MESSAGE_DELIVERY_TIME,
                PR_MESSAGE_FLAGS,
                PR_MESSAGE_SIZE,
            ],
            &contents_rows,
            &mut table_children,
        );
        emitter.add_node(
            (spec.index << 5) | NID_TYPE_CONTENTS_TABLE,
            0,
            NodeData::new(table, table_children),
        )?;

        let mut table_children = SubnodeSet::new();
        let table = build_table_context(&[PR_MESSAGE_CLASS], &[], &mut table_children);
        emitter.add_node(
            (spec.index << 5) | NID_TYPE_ASSOC_CONTENTS_TABLE,
            0,
            NodeData::new(table, table_children),
        )?;
    }

    let file_bytes = emitter.finish()?;
    Ok(Manifest {
        seed: config.seed,
        file_bytes,
        folders: (0..FOLDERS.len())
            .filter(|folder| *folder != FOLDER_ROOT)
            .map(folder_path)
            .collect(),
        messages,
        dangling_contents_rows,
    })
}

// ---------------------------------------------------------------------------
// Manifest JSON
// ---------------------------------------------------------------------------

fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if (ch as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", ch as u32);
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

impl Manifest {
    /// One JSON document; message bodies are represented by length and checksum.
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        let _ = write!(
            out,
            "{{\n  \"generator\": \"tools/synthetic_pst.rs\",\n  \"seed\": {},\n  \"file_bytes\": {},\n  \"folders\": [",
            self.seed, self.file_bytes
        );
        for (index, folder) in self.folders.iter().enumerate() {
            let _ = write!(
                out,
                "{}{}",
                if index == 0 { "" } else { ", " },
                json_string(folder)
            );
        }
        let _ = write!(
            out,
            "],\n  \"dangling_contents_rows\": {:?},\n  \"messages\": [\n",
            self.dangling_contents_rows
        );
        for (index, message) in self.messages.iter().enumerate() {
            let _ = write!(
                out,
                "    {{\"index\": {}, \"node_id\": {}, \"folder\": {}, \"scenario\": {}, \"message_class\": {}, \"subject\": {}, \"body_utf8_bytes\": {}, \"body_fnv1a64\": \"{:016x}\", \"has_html\": {}, \"draft\": {}, \"deleted\": {}, \"recipient_count\": {}, \"internet_message_id\": {}, \"in_contents_table\": {}, \"recoverable\": {}, \"attachments\": [",
                message.index,
                message.node_id,
                json_string(&message.folder),
                json_string(message.scenario.label()),
                json_string(&message.message_class),
                json_string(&message.subject),
                message.body_text.len(),
                fnv1a64(message.body_text.as_bytes()),
                message.has_html,
                message.draft,
                message.deleted,
                message.recipient_count,
                json_string(&message.internet_message_id),
                message.in_contents_table,
                message.recoverable(),
            );
            for (position, attachment) in message.attachments.iter().enumerate() {
                let _ = write!(
                    out,
                    "{}{{\"filename\": {}, \"size\": {}, \"fnv1a64\": \"{:016x}\", \"fault\": {}, \"dangling_row\": {}, \"embedded_message\": {}}}",
                    if position == 0 { "" } else { ", " },
                    json_string(&attachment.filename),
                    attachment.size,
                    attachment.fnv1a64,
                    json_string(attachment.fault.label()),
                    attachment.dangling_row,
                    attachment.embedded_message,
                );
            }
            let _ = writeln!(
                out,
                "]}}{}",
                if index + 1 == self.messages.len() {
                    ""
                } else {
                    ","
                }
            );
        }
        out.push_str("  ]\n}\n");
        out
    }
}

// ---------------------------------------------------------------------------
// Command line
// ---------------------------------------------------------------------------

fn usage() -> ! {
    eprintln!("usage: synthetic-pst [--messages N] [--seed N] [--manifest PATH] OUTPUT.pst");
    std::process::exit(2);
}

fn main() -> io::Result<()> {
    let mut config = Config {
        seed: 1,
        messages: 200,
    };
    let mut manifest_path: Option<String> = None;
    let mut output: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--messages" => {
                config.messages = args
                    .next()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or_else(|| usage())
            }
            "--seed" => {
                config.seed = args
                    .next()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or_else(|| usage())
            }
            "--manifest" => manifest_path = Some(args.next().unwrap_or_else(|| usage())),
            other if other.starts_with('-') => usage(),
            other => {
                if output.replace(other.to_string()).is_some() {
                    usage();
                }
            }
        }
    }
    let Some(output) = output else { usage() };
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)?;
    let manifest = generate(io::BufWriter::with_capacity(1 << 20, file), config)?;
    if let Some(path) = manifest_path {
        std::fs::write(path, manifest.to_json())?;
    }
    println!(
        "wrote {output}: {} bytes, {} messages, {} folders, seed {}",
        manifest.file_bytes,
        manifest.messages.len(),
        manifest.folders.len(),
        manifest.seed
    );
    Ok(())
}
