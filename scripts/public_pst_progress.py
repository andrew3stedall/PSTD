#!/usr/bin/env python3
"""Create a compact public PST fixture progress summary for milestone CI artifacts."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def status_counter(status: str, key: str) -> int:
    for marker in (f"{key}=", f"{key}:"):
        if marker not in status:
            continue
        tail = status.split(marker, 1)[1]
        value = tail.split(";", 1)[0].split(",", 1)[0].strip()
        try:
            return int(value)
        except ValueError:
            return 0
    return 0


def load_status_lines(progress_dir: Path) -> list[str]:
    path = progress_dir / "message_statuses.txt"
    if not path.exists():
        return []
    return [line.strip() for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]


def status_sum(statuses: list[str], key: str) -> int:
    return sum(status_counter(status, key) for status in statuses)


def pq19_next_blocker(status: str) -> str:
    hierarchy_rows = status_counter(status, "pq19_hierarchy_table_rows")
    contents_rows = status_counter(status, "pq19_contents_table_rows")
    pq18_rows = status_counter(status, "pq18_candidate_rows")
    pq17_successes = status_counter(status, "pq17_table_parse_successes")
    if hierarchy_rows > 0 or contents_rows > 0:
        return "table_membership_wiring"
    if pq18_rows > 0:
        return "table_row_semantic_classification"
    if pq17_successes > 0:
        return "table_row_matrix_or_row_count_decode"
    return "table_membership_signal_absent"


def pq19_metrics(status: str) -> dict[str, Any]:
    return {
        "pq19_status": "table_membership_measurement_visible",
        "pq19_hierarchy_table_rows": status_counter(status, "pq19_hierarchy_table_rows"),
        "pq19_contents_table_rows": status_counter(status, "pq19_contents_table_rows"),
        "pq19_table_linked_folders": status_counter(status, "pq19_table_linked_folders"),
        "pq19_table_linked_messages": status_counter(status, "pq19_table_linked_messages"),
        "pq19_next_blocker": pq19_next_blocker(status),
    }


def pq21_metrics(statuses: list[str]) -> dict[str, Any]:
    columns = status_sum(statuses, "subnode_table_columns")
    rows = status_sum(statuses, "subnode_table_rows")
    values = status_sum(statuses, "subnode_table_values")
    if rows > 0 and values > 0:
        next_blocker = "table_row_property_candidate_mapping"
    elif rows > 0:
        next_blocker = "table_row_value_extraction"
    else:
        next_blocker = "real_table_row_layout_decode"
    return {
        "pq21_status": "table_parser_counters_visible",
        "pq21_table_declared_columns": status_sum(statuses, "subnode_table_declared_columns"),
        "pq21_table_columns": columns,
        "pq21_table_declared_rows": status_sum(statuses, "subnode_table_declared_rows"),
        "pq21_table_rows": rows,
        "pq21_table_values": values,
        "pq21_table_omitted_values": status_sum(statuses, "subnode_table_omitted_values"),
        "pq21_next_blocker": next_blocker,
    }


def pq20_next_blocker(status: str, pq21: dict[str, Any]) -> str:
    parsed_rows = max(status_counter(status, "subnode_table_rows"), pq21["pq21_table_rows"])
    parsed_columns = max(status_counter(status, "subnode_table_columns"), pq21["pq21_table_columns"])
    table_successes = status_counter(status, "pq17_table_parse_successes")
    if parsed_rows > 0 and parsed_columns > 0:
        return "table_row_property_candidate_mapping"
    if table_successes > 0:
        return "subnode_table_parser_counter_wiring"
    return "table_row_matrix_signal_absent"


def pq20_metrics(status: str, pq21: dict[str, Any]) -> dict[str, Any]:
    parsed_rows = max(status_counter(status, "subnode_table_rows"), pq21["pq21_table_rows"])
    parsed_columns = max(status_counter(status, "subnode_table_columns"), pq21["pq21_table_columns"])
    return {
        "pq20_status": "table_row_matrix_measurement_visible",
        "pq20_row_matrix_decode_attempts": status_counter(status, "pq17_table_parse_successes"),
        "pq20_parsed_table_columns": parsed_columns,
        "pq20_parsed_table_rows": parsed_rows,
        "pq20_row_value_slots": parsed_columns * parsed_rows,
        "pq20_next_blocker": pq20_next_blocker(status, pq21),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--progress-dir", required=True)
    args = parser.parse_args()

    progress_dir = Path(args.progress_dir)
    inspect = load_json(progress_dir / "inspect.json")
    run = load_json(progress_dir / "run_summary.json")
    fixture = (progress_dir / "fixture_path.txt").read_text(encoding="utf-8").strip()
    fixture_size = int((progress_dir / "fixture_size_bytes.txt").read_text(encoding="utf-8"))
    extract_status = run.get("status", "")
    pq21 = pq21_metrics(load_status_lines(progress_dir))

    summary = {
        "fixture": fixture,
        "fixture_size_bytes": fixture_size,
        "root_diagnostic_condition": inspect.get("root_diagnostic_condition"),
        "root_selected_source": inspect.get("header", {}).get("root_diagnostics", {}).get("selected_source"),
        "root_candidate_count": inspect.get("header", {}).get("root_diagnostics", {}).get("candidate_count"),
        "bbt_status": inspect.get("bbt_status"),
        "bbt_entries": inspect.get("bbt_entries"),
        "bbt_pages_diagnosed": len(inspect.get("bbt_page_diagnostics", [])),
        "nbt_status": inspect.get("nbt_status"),
        "nbt_entries": inspect.get("nbt_entries"),
        "nbt_pages_diagnosed": len(inspect.get("nbt_page_diagnostics", [])),
        "extract_status": extract_status,
        **pq19_metrics(extract_status),
        **pq21,
        **pq20_metrics(extract_status, pq21),
        "folders_discovered": run.get("folders_discovered"),
        "messages_discovered": run.get("messages_discovered"),
        "messages_extracted": run.get("messages_extracted"),
        "attachments_extracted": run.get("attachments_extracted"),
        "tar_shards_written": run.get("tar_shards_written"),
        "bytes_written": run.get("bytes_written"),
    }

    (progress_dir / "progress_summary.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    lines = ["# Public PST Progress Artifact", "", "| Metric | Value |", "|---|---|"]
    for key, value in summary.items():
        lines.append(f"| `{key}` | `{value}` |")
    (progress_dir / "progress_summary.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
