#!/usr/bin/env python3
"""PQ32: classify whether the current table descriptor layout assumption is valid."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


def status_value(status: str, key: str) -> int:
    match = re.search(rf"(?:^|; )({re.escape(key)})=(\d+)(?:;|$)", status)
    return int(match.group(2)) if match else 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--progress-dir", required=True)
    args = parser.parse_args()

    progress_dir = Path(args.progress_dir)
    run = json.loads((progress_dir / "run_summary.json").read_text(encoding="utf-8"))
    status = str(run.get("status", ""))

    row_width = status_value(status, "subnode_table_row_width")
    first_offset = status_value(status, "subnode_table_first_unknown_offset")
    first_width = status_value(status, "subnode_table_first_unknown_width")
    second_offset = status_value(status, "subnode_table_second_unknown_offset")
    second_width = status_value(status, "subnode_table_second_unknown_width")
    parsed_rows = status_value(status, "pq17_table_rows")
    values = status_value(status, "pq21_table_values")

    descriptor_fields_present = row_width > 0 and (first_width > 0 or second_width > 0)
    contradictory_parse = parsed_rows > 0 and values > 0 and not descriptor_fields_present

    if contradictory_parse:
        diagnosis = "legacy_table_header_assumption_invalid"
        next_blocker = "raw_table_payload_prefix_capture"
    elif descriptor_fields_present:
        diagnosis = "descriptor_layout_fields_present"
        next_blocker = "descriptor_bounds_validation"
    else:
        diagnosis = "insufficient_table_signal"
        next_blocker = "broader_fixture_or_payload_selection"

    summary = {
        "pq32_status": "descriptor_layout_diagnosis_visible",
        "pq32_row_width": row_width,
        "pq32_first_unknown_offset": first_offset,
        "pq32_first_unknown_width": first_width,
        "pq32_second_unknown_offset": second_offset,
        "pq32_second_unknown_width": second_width,
        "pq32_parsed_rows": parsed_rows,
        "pq32_values": values,
        "pq32_descriptor_fields_present": descriptor_fields_present,
        "pq32_contradictory_parse": contradictory_parse,
        "pq32_diagnosis": diagnosis,
        "pq32_next_blocker": next_blocker,
    }

    (progress_dir / "pq32_summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    lines = ["# PQ32 descriptor layout diagnosis", ""]
    lines.extend(f"- `{key}`: `{value}`" for key, value in summary.items())
    (progress_dir / "pq32_summary.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json.dumps(summary, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
