#!/usr/bin/env python3
"""Independent byte-level validator for the controlled ANSI Stage-A fixture."""

from __future__ import annotations

import json
import struct
import sys
from pathlib import Path

PAGE_SIZE = 512
BBT_OFFSET = 1024
NBT_OFFSET = 1536
FILE_SIZE = 2048


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


def validate(path: Path, expected_crypt: int = 0) -> dict[str, object]:
    data = path.read_bytes()
    assert len(data) == FILE_SIZE, (path, len(data))
    assert data[:4] == b"!BDN"
    assert data[8:10] == b"SM"
    assert u16(data, 10) == 14
    assert u16(data, 12) == 19
    assert data[14:16] == b"\x01\x01"
    assert u32(data, 168) == FILE_SIZE
    assert u32(data, 188) == NBT_OFFSET
    assert u32(data, 196) == BBT_OFFSET
    assert data[200] == 2
    assert data[204:460] == b"\xff" * 256
    assert data[460] == 0x80
    assert data[461] == expected_crypt
    assert u32(data, 4) == weak_crc32(data[8:479])

    pages = []
    for offset, ptype, bid in ((BBT_OFFSET, 0x80, 0x22), (NBT_OFFSET, 0x81, 0x61)):
        page = data[offset : offset + PAGE_SIZE]
        assert len(page) == PAGE_SIZE
        assert page[496:500] == b"\x00\x00\x0c\x00"
        assert page[500] == ptype
        assert page[501] == ptype
        assert u16(page, 502) == page_signature(offset, bid)
        assert u32(page, 504) == bid
        assert u32(page, 508) == weak_crc32(page[:500])
        pages.append({"offset": offset, "type": hex(ptype), "bid": hex(bid), "entries": 0})

    return {
        "fixture": path.name,
        "status": "valid_empty_ansi_stage_a",
        "file_size": len(data),
        "crypt_method": expected_crypt,
        "bbt_offset": BBT_OFFSET,
        "nbt_offset": NBT_OFFSET,
        "bbt_entries": 0,
        "nbt_entries": 0,
        "pages": pages,
    }


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: validate_ansi_stage_a.py <fixture> <crypt-method>")
    print(json.dumps(validate(Path(sys.argv[1]), int(sys.argv[2])), sort_keys=True))
