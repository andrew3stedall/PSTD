#!/usr/bin/env python3
"""PQ34: identify whether the captured table-like payload is an NDB SLBLOCK."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


SLBLOCK_TYPE = 0x02
SLBLOCK_LEAF_LEVEL = 0x00
UNICODE_SLBLOCK_HEADER_BYTES = 8
UNICODE_SLENTRY_BYTES = 24


def u16_le(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 2], "little")


def u32_le(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 4], "little")


def u64_le(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 8], "little")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--progress-dir", required=True)
    args = parser.parse_args()

    progress_dir = Path(args.progress_dir)
    pq33 = json.loads((progress_dir / "pq33_summary.json").read_text(encoding="utf-8"))
    prefix_hex = str(pq33.get("pq33_prefix_hex", ""))
    try:
        prefix = bytes.fromhex(prefix_hex)
        hex_valid = len(prefix) * 2 == len(prefix_hex)
    except ValueError:
        prefix = b""
        hex_valid = False

    payload_len = int(pq33.get("pq33_payload_byte_len", 0))
    prefix_len = len(prefix)
    prefix_complete = (
        hex_valid
        and payload_len > 0
        and prefix_len == payload_len
        and not bool(pq33.get("pq33_prefix_truncated", False))
    )

    btype = prefix[0] if prefix_len >= 1 else -1
    clevel = prefix[1] if prefix_len >= 2 else -1
    entry_count = u16_le(prefix, 2) if prefix_len >= 4 else 0
    padding = u32_le(prefix, 4) if prefix_len >= 8 else -1
    expected_struct_bytes = (
        UNICODE_SLBLOCK_HEADER_BYTES + entry_count * UNICODE_SLENTRY_BYTES
    )
    slblock_signature_valid = (
        btype == SLBLOCK_TYPE
        and clevel == SLBLOCK_LEAF_LEVEL
        and entry_count > 0
        and padding == 0
    )
    slblock_boundary_visible = (
        slblock_signature_valid
        and prefix_complete
        and payload_len == expected_struct_bytes
    )

    first_entry_visible = slblock_boundary_visible and entry_count >= 1
    first_nid = u64_le(prefix, 8) if first_entry_visible else 0
    first_bid_data = u64_le(prefix, 16) if first_entry_visible else 0
    first_bid_sub = u64_le(prefix, 24) if first_entry_visible else 0

    legacy_rows = int(pq33.get("pq33_parsed_rows", 0))
    legacy_values = int(pq33.get("pq33_values", 0))
    legacy_table_parse_spurious = (
        slblock_boundary_visible and legacy_rows > 0 and legacy_values > 0
    )

    if slblock_boundary_visible:
        diagnosis = "unicode_slblock_boundary_confirmed"
        next_blocker = "slentry_bid_target_resolution"
    elif slblock_signature_valid:
        diagnosis = "slblock_signature_visible_boundary_incomplete"
        next_blocker = "longer_slblock_capture"
    else:
        diagnosis = "slblock_signature_absent"
        next_blocker = "alternate_payload_structure_classification"

    summary = {
        "pq34_status": "slblock_boundary_diagnosis_visible",
        "pq34_payload_byte_len": payload_len,
        "pq34_prefix_byte_len": prefix_len,
        "pq34_prefix_complete": prefix_complete,
        "pq34_btype": btype,
        "pq34_clevel": clevel,
        "pq34_entry_count": entry_count,
        "pq34_unicode_padding": padding,
        "pq34_expected_struct_bytes": expected_struct_bytes,
        "pq34_slblock_signature_valid": slblock_signature_valid,
        "pq34_slblock_boundary_visible": slblock_boundary_visible,
        "pq34_first_entry_nid": first_nid,
        "pq34_first_entry_bid_data": first_bid_data,
        "pq34_first_entry_bid_sub": first_bid_sub,
        "pq34_legacy_table_parse_spurious": legacy_table_parse_spurious,
        "pq34_diagnosis": diagnosis,
        "pq34_next_blocker": next_blocker,
    }

    (progress_dir / "pq34_summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    lines = ["# PQ34 SLBLOCK boundary diagnosis", ""]
    lines.extend(f"- `{key}`: `{value}`" for key, value in summary.items())
    (progress_dir / "pq34_summary.md").write_text(
        "\n".join(lines) + "\n", encoding="utf-8"
    )

    print(json.dumps(summary, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
