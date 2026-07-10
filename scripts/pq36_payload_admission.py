#!/usr/bin/env python3
"""PQ36: report decoded subnode payload admission and provenance."""

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
    path = progress_dir / "pq33_messages.jsonl"
    if not path.exists():
        return statuses
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        record = json.loads(line)
        statuses.extend(str(record.get(field, "")) for field in STATUS_FIELDS)
    return statuses


def first_int(statuses: list[str], key: str) -> int:
    return next(
        (value for value in (status_int(status, key) for status in statuses) if value),
        0,
    )


def first_text(statuses: list[str], key: str) -> str:
    return next(
        (value for value in (status_text(status, key) for status in statuses) if value),
        "",
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--progress-dir", required=True)
    args = parser.parse_args()

    progress_dir = Path(args.progress_dir)
    run = json.loads((progress_dir / "run_summary.json").read_text(encoding="utf-8"))
    inspect = json.loads((progress_dir / "inspect.json").read_text(encoding="utf-8"))
    run_status = str(run.get("status", ""))
    statuses = load_statuses(progress_dir, run_status)

    crypt_method = inspect.get("header", {}).get("crypt_method")
    heap_contexts = first_int(statuses, "subnode_heap_contexts")
    heap_table_contexts = first_int(statuses, "subnode_heap_table_contexts")
    heap_property_contexts = first_int(statuses, "subnode_heap_property_contexts")
    heap_bth_contexts = first_int(statuses, "subnode_heap_bth_contexts")
    unsupported_layouts = status_int(run_status, "subnode_unsupported_layouts")
    unsupported_block_id = first_int(statuses, "subnode_unsupported_payload_block_id")
    unsupported_byte_len = first_int(statuses, "subnode_unsupported_payload_byte_len")
    unsupported_prefix = first_text(statuses, "subnode_unsupported_payload_prefix_hex")
    table_rows = status_int(run_status, "pq17_table_rows")
    table_values = status_int(run_status, "pq21_table_values")

    permute_decode_visible = crypt_method == 1 and heap_contexts > 0
    implausible_table_rejected = table_rows == 0 and table_values == 0

    if permute_decode_visible and heap_table_contexts > 0:
        diagnosis = "permutative_decode_revealed_heap_table_context"
        next_blocker = "heap_table_context_header_traversal"
    elif permute_decode_visible:
        diagnosis = "permutative_decode_revealed_heap_context"
        next_blocker = "heap_client_structure_traversal"
    elif unsupported_layouts > 0:
        diagnosis = "decoded_payload_unsupported"
        next_blocker = "unsupported_payload_or_crypt_method_analysis"
    else:
        diagnosis = "payload_admission_signal_absent"
        next_blocker = "payload_provenance_propagation"

    summary = {
        "pq36_status": "decoded_payload_admission_visible",
        "pq36_crypt_method": crypt_method,
        "pq36_heap_contexts": heap_contexts,
        "pq36_heap_table_contexts": heap_table_contexts,
        "pq36_heap_property_contexts": heap_property_contexts,
        "pq36_heap_bth_contexts": heap_bth_contexts,
        "pq36_unsupported_layouts": unsupported_layouts,
        "pq36_unsupported_block_id": unsupported_block_id,
        "pq36_unsupported_byte_len": unsupported_byte_len,
        "pq36_unsupported_prefix_hex": unsupported_prefix,
        "pq36_table_rows": table_rows,
        "pq36_table_values": table_values,
        "pq36_permute_decode_visible": permute_decode_visible,
        "pq36_implausible_table_rejected": implausible_table_rejected,
        "pq36_diagnosis": diagnosis,
        "pq36_next_blocker": next_blocker,
    }

    (progress_dir / "pq36_summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    lines = ["# PQ36 decoded payload admission", ""]
    lines.extend(f"- `{key}`: `{value}`" for key, value in summary.items())
    (progress_dir / "pq36_summary.md").write_text(
        "\n".join(lines) + "\n", encoding="utf-8"
    )

    print(json.dumps(summary, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
