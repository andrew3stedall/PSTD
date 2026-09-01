#!/usr/bin/env python3
"""Independently validate the controlled ANSI by-value attachment fixture."""

from __future__ import annotations

import hashlib
import json
import struct
import sys
from pathlib import Path

PAGE_SIZE = 512
BBT_OFFSET = 1024
NBT_OFFSET = 1536
ATTACHMENT_PAYLOAD = b"ANSI Stage-C arbitrary attachment payload\n"
INDIRECT_ATTACHMENT_DATA_NID = 0x311


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


def u64(data: bytes, offset: int) -> int:
    return struct.unpack_from("<Q", data, offset)[0]


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
    assert all(offset <= len(data) for offset in offsets)
    return user_root, {
        index + 1: data[offsets[index] : offsets[index + 1]]
        for index in range(count)
    }


def hid(allocations: dict[int, bytes], value: int) -> bytes:
    assert value & 0x1F == 0
    return allocations[value >> 5]


def parse_flat_properties(data: bytes) -> dict[int, bytes]:
    assert data[0] == 4
    width = data[1]
    count = u16(data, 2)
    entry_size = 4 + width
    assert len(data) == 8 + count * entry_size
    return {
        u32(data, 8 + index * entry_size): data[12 + index * entry_size : 8 + (index + 1) * entry_size]
        for index in range(count)
    }


def string8(properties: dict[int, bytes], tag: int) -> str:
    return properties[tag].split(b"\0", 1)[0].decode("latin-1")


def parse_property_heap(data: bytes) -> dict[int, bytes]:
    assert data[2:4] == b"\xec\xbc"
    user_root, allocations = parse_heap(data)
    bth_header = hid(allocations, user_root)
    assert bth_header[:4] == b"\xb5\x02\x06\x00"
    leaf = hid(allocations, u32(bth_header, 4))
    assert len(leaf) == 5 * 8
    properties = {}
    for index in range(5):
        start = index * 8
        property_id = u16(leaf, start)
        property_type = u16(leaf, start + 2)
        value_hid = u32(leaf, start + 4)
        properties[(property_id << 16) | property_type] = (
            value_hid.to_bytes(4, "little")
            if property_type == 0x000D
            else hid(allocations, value_hid)
        )
    return properties


def validate(path: Path, indirect: bool, method: int) -> dict[str, object]:
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
        assert page[498:500] == b"\x0c\x00"
        assert page[500:502] == bytes((page_type, page_type))
        assert u16(page, 502) == page_signature(offset, bid)
        assert u32(page, 504) == bid
        assert u32(page, 508) == weak_crc32(page[:500])

    expected_block_count = 7 if indirect else 6
    assert bbt_page[496] == expected_block_count
    blocks = {}
    for index in range(bbt_page[496]):
        start = index * 12
        block_id, offset, size = u32(bbt_page, start), u32(bbt_page, start + 4), u16(
            bbt_page, start + 8
        )
        assert block_id not in blocks
        assert offset >= NBT_OFFSET + PAGE_SIZE
        assert size > 0
        assert offset + size <= len(data)
        blocks[block_id] = data[offset : offset + size]
    expected_blocks = {0x100, 0x102, 0x104, 0x106, 0x108, 0x10A}
    if indirect:
        expected_blocks.add(0x10C)
    assert set(blocks) == expected_blocks

    assert nbt_page[496] == 3
    nodes = {}
    for index in range(nbt_page[496]):
        start = index * 12
        node_id, data_bid, subnode_bid = (
            u32(nbt_page, start),
            u32(nbt_page, start + 4),
            u32(nbt_page, start + 8),
        )
        assert node_id not in nodes
        assert data_bid in blocks
        nodes[node_id] = (data_bid, subnode_bid)
    assert nodes == {0x22: (0x100, 0), 0x24: (0x102, 0x106), 0x2E: (0x104, 0)}

    folder = parse_flat_properties(blocks[0x100])
    assert string8(folder, 0x3001_001E) == "Synthetic Mail"

    message = parse_flat_properties(blocks[0x102])
    assert string8(message, 0x0037_001E) == "ANSI Stage C attachment message"
    assert message[0x0E1B_000B][0] == 1
    assert string8(message, 0x1000_001E) == "Hello from the deterministic ANSI Stage-C fixture."

    slblock = blocks[0x106]
    assert slblock[:4] == bytes((0x02, 0x00, 0x03 if indirect else 0x02, 0x00))
    assert u64(slblock, 8) == 0x32
    assert u64(slblock, 16) == 0x108
    assert u64(slblock, 32) == 0x31
    assert u64(slblock, 40) == 0x10A
    if indirect:
        assert u64(slblock, 56) == INDIRECT_ATTACHMENT_DATA_NID
        assert u64(slblock, 64) == 0x10C

    attachment = parse_property_heap(blocks[0x10A])
    assert string8(attachment, 0x3704_001E) == "ansi-attachment.bin"
    assert string8(attachment, 0x370E_001E) == "application/octet-stream"
    assert u32(attachment[0x3705_0003], 0) == method
    assert u32(attachment[0x0E20_0003], 0) == len(ATTACHMENT_PAYLOAD)
    if indirect:
        assert attachment[0x3701_000D] == INDIRECT_ATTACHMENT_DATA_NID.to_bytes(4, "little")
        assert blocks[0x10C] == ATTACHMENT_PAYLOAD
    else:
        assert attachment[0x3701_0102] == ATTACHMENT_PAYLOAD

    return {
        "fixture": path.name,
        "status": (
            "valid_ansi_stage_c_method_2_attachment"
            if method == 2
            else (
                "valid_ansi_stage_c_indirect_one_by_value_attachment"
                if indirect
                else "valid_ansi_stage_c_one_by_value_attachment"
            )
        ),
        "file_size": len(data),
        "sha256": hashlib.sha256(data).hexdigest(),
        "crypt_method": data[461],
        "bbt_entries": bbt_page[496],
        "nbt_entries": nbt_page[496],
        "message_node": "0x24",
        "attachment_method": 1,
        "attachment_filename": "ansi-attachment.bin",
        "attachment_size": len(ATTACHMENT_PAYLOAD),
        "attachment_sha256": hashlib.sha256(ATTACHMENT_PAYLOAD).hexdigest(),
    }


if __name__ == "__main__":
    flags = set(sys.argv[2:])
    valid_flags = {"--indirect", "--method-2"}
    if len(sys.argv) > 4 or not flags.issubset(valid_flags):
        raise SystemExit(
            "usage: validate_ansi_attachment.py <fixture> [--indirect] [--method-2]"
        )
    indirect = "--indirect" in flags
    method = 2 if "--method-2" in flags else 1
    print(json.dumps(validate(Path(sys.argv[1]), indirect, method), sort_keys=True))
