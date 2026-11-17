#!/usr/bin/env python3
"""Independent byte-level validator for the controlled ANSI Stage-B fixture."""

from __future__ import annotations

import json
import hashlib
import struct
import sys
from pathlib import Path

PAGE_SIZE = 512
BBT_OFFSET = 1024
NBT_OFFSET = 1536
HN_HEADER_SIZE = 12
SLBLOCK_BID = 0x10E
NID_RECIPIENT_TABLE = 0x692
INLINE_PC_TYPES = {0x0002, 0x0003, 0x0004, 0x000A, 0x000B, 0x000D}


def weak_crc32(data: bytes) -> int:
    crc = 0
    for byte in data:
        crc ^= byte
        for _ in range(8):
            crc = (crc >> 1) ^ 0xEDB88320 if crc & 1 else crc >> 1
    return crc


def u16(data: bytes, offset: int) -> int:
    return struct.unpack_from("<H", data, offset)[0]


def u32(data: bytes, offset: int) -> int:
    return struct.unpack_from("<I", data, offset)[0]


def page_signature(file_offset: int, bid: int) -> int:
    value = file_offset ^ bid
    return ((value >> 16) & 0xFFFF) ^ (value & 0xFFFF)


def parse_heap(data: bytes) -> tuple[int, dict[int, bytes]]:
    assert len(data) >= 8
    page_map_offset = u16(data, 0)
    assert data[2] == 0xEC
    user_root = u32(data, 4)
    count = u16(data, page_map_offset)
    offsets = [u16(data, page_map_offset + 4 + index * 2) for index in range(count + 1)]
    assert offsets == sorted(offsets)
    # The full HNHDR (ibHnpm, bSig, bClientSig, hidUserRoot, rgbFillLevel)
    # precedes the first allocation.
    assert offsets[0] == HN_HEADER_SIZE
    allocations = {
        index + 1: data[offsets[index] : offsets[index + 1]]
        for index in range(count)
    }
    assert all(offset <= len(data) for offset in offsets)
    return user_root, allocations


def hid(allocations: dict[int, bytes], value: int) -> bytes:
    assert value & 0x1F == 0
    return allocations[value >> 5]


def parse_property_context(data: bytes) -> dict[int, bytes]:
    """Read a heap-on-node Property Context: HN -> BTH header -> sorted leaf."""
    assert data[2:4] == b"\xec\xbc"
    user_root, allocations = parse_heap(data)
    assert user_root == 0x20
    bth_header = hid(allocations, user_root)
    assert bth_header[:4] == b"\xb5\x02\x06\x00"
    leaf = hid(allocations, u32(bth_header, 4))
    assert len(leaf) % 8 == 0 and leaf
    property_ids = [u16(leaf, start) for start in range(0, len(leaf), 8)]
    assert property_ids == sorted(set(property_ids))
    properties = {}
    for start in range(0, len(leaf), 8):
        property_id = u16(leaf, start)
        property_type = u16(leaf, start + 2)
        value_hid = u32(leaf, start + 4)
        properties[(property_id << 16) | property_type] = (
            value_hid.to_bytes(4, "little")
            if property_type in INLINE_PC_TYPES
            else hid(allocations, value_hid)
        )
    return properties


def string8(properties: dict[int, bytes], tag: int) -> str:
    value = properties[tag]
    assert b"\0" not in value
    return value.decode("latin-1")


def validate(path: Path) -> dict[str, object]:
    data = path.read_bytes()
    assert len(data) > NBT_OFFSET + PAGE_SIZE
    assert data[:4] == b"!BDN"
    assert data[8:10] == b"SM"
    assert u16(data, 10) == 14
    assert u16(data, 12) == 19
    assert data[14:16] == b"\x01\x01"
    assert u32(data, 168) == len(data)
    assert u32(data, 184) == 0x61
    assert u32(data, 188) == NBT_OFFSET
    assert u32(data, 192) == 0x22
    assert u32(data, 196) == BBT_OFFSET
    assert data[461] == 0
    assert u32(data, 4) == weak_crc32(data[8:479])

    bbt_page = data[BBT_OFFSET : BBT_OFFSET + PAGE_SIZE]
    nbt_page = data[NBT_OFFSET : NBT_OFFSET + PAGE_SIZE]
    for page, offset, page_type, bid in (
        (bbt_page, BBT_OFFSET, 0x80, 0x22),
        (nbt_page, NBT_OFFSET, 0x81, 0x61),
    ):
        assert len(page) == PAGE_SIZE
        assert page[498:500] == b"\x0c\x00"
        assert page[500:502] == bytes((page_type, page_type))
        assert u16(page, 502) == page_signature(offset, bid)
        assert u32(page, 504) == bid
        assert u32(page, 508) == weak_crc32(page[:500])

    bbt_count = bbt_page[496]
    assert bbt_count == 5
    blocks = {}
    for index in range(bbt_count):
        start = index * 12
        block_id, offset, size = u32(bbt_page, start), u32(bbt_page, start + 4), u16(
            bbt_page, start + 8
        )
        assert block_id not in blocks
        # Only the SLBLOCK is internal; every data block has a leaf BID.
        assert bool(block_id & 0x2) == (block_id == SLBLOCK_BID), hex(block_id)
        assert offset >= NBT_OFFSET + PAGE_SIZE
        assert size > 0
        assert offset + size <= len(data)
        blocks[block_id] = data[offset : offset + size]

    nbt_count = nbt_page[496]
    assert nbt_count == 3
    nodes = {}
    for index in range(nbt_count):
        start = index * 12
        node_id, data_bid, subnode_bid = (
            u32(nbt_page, start),
            u32(nbt_page, start + 4),
            u32(nbt_page, start + 8),
        )
        assert node_id not in nodes
        assert data_bid in blocks
        nodes[node_id] = (data_bid, subnode_bid)

    assert set(nodes) == {0x22, 0x24, 0x2E}
    assert nodes[0x22] == (0x100, 0)
    assert nodes[0x24] == (0x104, SLBLOCK_BID)
    assert nodes[0x2E] == (0x108, 0)

    folder = parse_property_context(blocks[0x100])
    assert string8(folder, 0x3001_001E) == "Synthetic Mail"
    assert u32(folder[0x3602_0003], 0) == 1

    message = parse_property_context(blocks[0x104])
    expected_message = {
        0x001A_001E: "IPM.Note",
        0x0037_001E: "ANSI Stage B synthetic message",
        0x0C1A_001E: "PSTD Fixture Sender",
        0x0C1F_001E: "sender@example.test",
        0x0C1E_001E: "SMTP",
        0x1000_001E: "Hello from the deterministic ANSI Stage-B fixture.",
        0x1035_001E: "<ansi-stage-b-001@example.test>",
    }
    for tag, value in expected_message.items():
        assert string8(message, tag) == value
    assert "Date: Tue, 01 Jan 2019 00:00:00 +0000" in string8(message, 0x007D_001E)
    assert "To: Recipient <recipient@example.test>" in string8(message, 0x007D_001E)

    assert blocks[0x108][3] == 0x7C
    contents_root, contents_allocations = parse_heap(blocks[0x108])
    assert contents_root == 0x40
    contents_tcinfo = hid(contents_allocations, contents_root)
    assert contents_tcinfo[:2] == b"\x7c\x01"
    assert u32(contents_tcinfo, 10) == 0x20
    contents_index_header = hid(contents_allocations, u32(contents_tcinfo, 10))
    contents_index = hid(contents_allocations, u32(contents_index_header, 4))
    assert u32(contents_index, 0) == 0x24
    assert u32(contents_index, 4) == 0

    slblock = blocks[SLBLOCK_BID]
    assert slblock[:4] == b"\x02\x00\x01\x00"
    # ANSI SLBLOCK: no padding after cEnt; SLENTRY is nid, bidData, bidSub.
    assert len(slblock) == 4 + 12
    assert u32(slblock, 4) == NID_RECIPIENT_TABLE
    assert u32(slblock, 8) == 0x110
    assert u32(slblock, 12) == 0

    assert blocks[0x110][3] == 0x7C
    recipient_root, recipient_allocations = parse_heap(blocks[0x110])
    assert recipient_root == 0x40
    recipient_tcinfo = hid(recipient_allocations, recipient_root)
    assert recipient_tcinfo[:2] == b"\x7c\x03"
    assert u32(recipient_tcinfo, 10) == 0x20
    assert u32(recipient_tcinfo, 14) == 0x80
    assert recipient_tcinfo[22:26] == (0x0C15_0003).to_bytes(4, "little")
    assert recipient_tcinfo[30:34] == (0x3001_001F).to_bytes(4, "little")
    assert recipient_tcinfo[38:42] == (0x39FE_001F).to_bytes(4, "little")
    recipient_row = hid(recipient_allocations, 0x80)
    assert recipient_row[:4] == (1).to_bytes(4, "little")
    assert u32(recipient_row, 4) == 0xA0
    assert u32(recipient_row, 8) == 0xC0
    # CEB bits are most-significant first: columns 0, 1 and 2 are present.
    assert recipient_row[12] == 0xE0
    assert hid(recipient_allocations, 0xA0).decode("utf-16-le").rstrip("\0") == "Recipient"
    assert (
        hid(recipient_allocations, 0xC0).decode("utf-16-le").rstrip("\0")
        == "recipient@example.test"
    )

    return {
        "fixture": path.name,
        "status": "valid_ansi_stage_b_one_folder_one_message",
        "file_size": len(data),
        "sha256": hashlib.sha256(data).hexdigest(),
        "crypt_method": data[461],
        "bbt_entries": bbt_count,
        "nbt_entries": nbt_count,
        "folder_node": "0x22",
        "message_node": "0x24",
        "contents_table_node": "0x2e",
        "recipient_table_node": "0x692",
        "attachment_count": 0,
    }


def u64(data: bytes, offset: int) -> int:
    return struct.unpack_from("<Q", data, offset)[0]


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: validate_ansi_stage_b.py <fixture>")
    print(json.dumps(validate(Path(sys.argv[1])), sort_keys=True))
