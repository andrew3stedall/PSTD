#!/usr/bin/env python3
"""PQ35: report whether Unicode SLENTRY targets were resolved safely."""

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
    return next(
        (value for value in (status_int(status, key) for status in statuses) if value),
        0,
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--progress-dir", required=True)
    args = parser.parse_args()

    progress_dir = Path(args.progress_dir)
    run = json.loads((progress_dir / "run_summary.json").read_text(encoding="utf-8"))
    run_status = str(run.get("status", ""))
    statuses = load_statuses(progress_dir, run_status)

    entries = first_int(statuses, "subnode_slblock_entries")
    data_references = first_int(statuses, "subnode_slblock_data_references")
    sub_references = first_int(statuses, "subnode_slblock_sub_references")
    decoded_blocks = status_int(run_status, "subnode_decoded_blocks")
    recursive_decodes = status_int(run_status, "subnode_recursive_child_decodes")
    child_references = status_int(run_status, "subnode_child_references")
    unsupported_layouts = status_int(run_status, "subnode_unsupported_layouts")
    table_rows = status_int(run_status, "pq17_table_rows")
    table_values = status_int(run_status, "pq21_table_values")

    target_resolution_visible = (
        entries > 0 and data_references > 0 and recursive_decodes > 0
    )
    cycle_limit_preserved = (
        target_resolution_visible
        and decoded_blocks <= 1 + data_references + sub_references
    )

    if target_resolution_visible and cycle_limit_preserved:
        diagnosis = "slentry_targets_resolved_with_cycle_guard"
        next_blocker = "resolved_data_payload_structure_validation"
    elif entries > 0 and data_references > 0:
        diagnosis = "slentry_targets_visible_but_unresolved"
        next_blocker = "slentry_bbt_lookup_or_depth_limit"
    else:
        diagnosis = "slblock_resolution_signal_absent"
        next_blocker = "slblock_classification_propagation"

    summary = {
        "pq35_status": "slentry_target_resolution_visible",
        "pq35_slblock_entries": entries,
        "pq35_data_references": data_references,
        "pq35_sub_references": sub_references,
        "pq35_child_references": child_references,
        "pq35_decoded_blocks": decoded_blocks,
        "pq35_recursive_child_decodes": recursive_decodes,
        "pq35_unsupported_layouts": unsupported_layouts,
        "pq35_table_rows_after_resolution": table_rows,
        "pq35_table_values_after_resolution": table_values,
        "pq35_target_resolution_visible": target_resolution_visible,
        "pq35_cycle_limit_preserved": cycle_limit_preserved,
        "pq35_diagnosis": diagnosis,
        "pq35_next_blocker": next_blocker,
    }

    (progress_dir / "pq35_summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    lines = ["# PQ35 SLENTRY target resolution", ""]
    lines.extend(f"- `{key}`: `{value}`" for key, value in summary.items())
    (progress_dir / "pq35_summary.md").write_text(
        "\n".join(lines) + "\n", encoding="utf-8"
    )

    print(json.dumps(summary, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
