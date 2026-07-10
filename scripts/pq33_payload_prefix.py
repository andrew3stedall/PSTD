#!/usr/bin/env python3
"""PQ33: capture a bounded raw prefix for the selected table-like payload."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


STATUS_FIELDS = (
    "status",
    "extraction_status",
    "metadata_status",
    "body_status",
    "attachment_status",
)


def status_int(status: str, key: str) -> int:
    match = re.search(rf"(?:^|; )({re.escape(key)})=(\d+)(?:;|$)", status)
    return int(match.group(2)) if match else 0


def status_text(status: str, key: str) -> str:
    match = re.search(rf"(?:^|; )({re.escape(key)})=([^;]*)(?:;|$)", status)
    return match.group(2).strip() if match else ""


def load_statuses(progress_dir: Path, run_status: str) -> list[str]:
    statuses = [run_status]
    message_path = progress_dir / "pq33_messages.jsonl"
    if not message_path.exists():
        return statuses
    for line in message_path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        record = json.loads(line)
        statuses.extend(str(record.get(field, "")) for field in STATUS_FIELDS)
    return statuses


def first_int(statuses: list[str], key: str) -> int:
    return next((value for value in (status_int(status, key) for status in statuses) if value), 0)


def first_text(statuses: list[str], key: str) -> str:
    return next((value for value in (status_text(status, key) for status in statuses) if value), "")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--progress-dir", required=True)
    args = parser.parse_args()

    progress_dir = Path(args.progress_dir)
    run = json.loads((progress_dir / "run_summary.json").read_text(encoding="utf-8"))
    status = str(run.get("status", ""))
    statuses = load_statuses(progress_dir, status)

    payload_len = first_int(statuses, "subnode_table_payload_byte_len")
    prefix_len = first_int(statuses, "subnode_table_payload_prefix_byte_len")
    prefix_hex = first_text(statuses, "subnode_table_payload_prefix_hex")
    truncated = first_int(statuses, "subnode_table_payload_prefix_truncated")
    parsed_rows = status_int(status, "pq17_table_rows")
    values = status_int(status, "pq21_table_values")

    expected_hex_len = prefix_len * 2
    prefix_shape_valid = prefix_len > 0 and len(prefix_hex) == expected_hex_len
    payload_prefix_visible = payload_len > 0 and prefix_shape_valid

    if payload_prefix_visible and parsed_rows > 0 and values > 0:
        next_blocker = "table_payload_header_boundary_interpretation"
    elif payload_prefix_visible:
        next_blocker = "table_payload_parser_selection"
    else:
        next_blocker = "table_payload_prefix_capture_gap"

    summary = {
        "pq33_status": "table_payload_prefix_capture_visible",
        "pq33_payload_byte_len": payload_len,
        "pq33_prefix_byte_len": prefix_len,
        "pq33_prefix_hex": prefix_hex,
        "pq33_prefix_hex_len": len(prefix_hex),
        "pq33_prefix_expected_hex_len": expected_hex_len,
        "pq33_prefix_shape_valid": prefix_shape_valid,
        "pq33_prefix_truncated": truncated == 1,
        "pq33_parsed_rows": parsed_rows,
        "pq33_values": values,
        "pq33_payload_prefix_visible": payload_prefix_visible,
        "pq33_status_sources": len(statuses),
        "pq33_next_blocker": next_blocker,
    }

    (progress_dir / "pq33_summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    lines = ["# PQ33 table payload prefix capture", ""]
    lines.extend(f"- `{key}`: `{value}`" for key, value in summary.items())
    (progress_dir / "pq33_summary.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json.dumps(summary, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
