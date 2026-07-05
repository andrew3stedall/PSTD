# Public PST Progress Log

## Purpose

Track conversion progress against the checked-in public PST fixture after every completed milestone.

This is the milestone-level quality signal for PST conversion coverage. It is separate from unit tests and release smoke tests because it records whether each milestone improves real PST extraction outcomes.

## Mandatory milestone rule

After every milestone PR is green and before the completion report is final:

1. Confirm the CI run uploaded the `public-pst-progress` artifact.
2. Download or inspect the artifact.
3. Add a new row to the progress table below.
4. Include the latest public-PST deltas in the user-facing milestone completion report.

Do not mark a conversion-quality milestone complete without reporting the public PST result.

## Artifact contents

The CI artifact `public-pst-progress` contains:

| File | Purpose |
|---|---|
| `fixture_path.txt` | Fixture selected by CI. |
| `fixture_size_bytes.txt` | Fixture size used for comparison. |
| `inspect.json` | Full `pstd inspect --json` output. |
| `run_summary.json` | Single-PST extraction summary. |
| `output_files.txt` | File inventory for the extraction output. |
| `progress_summary.json` | Compact machine-readable progress summary. |
| `progress_summary.md` | Compact human-readable progress summary. |

The artifact should contain summaries only. It must not include raw private PST inputs, message bodies, attachment payloads, or full archive shards.

## Metrics to log

Each milestone row should record:

- Root condition and selected root source.
- BBT/NBT entries and diagnosed page counts.
- Extraction status.
- Folder, message, and attachment counts.
- Whether this is progress, regression, or unchanged.
- Next blocker.

## Progress table

| Date | Milestone / PR | Commit or run | Fixture | Root condition | BBT entries | NBT entries | Folders | Messages | Attachments | Status | Notes |
|---|---|---|---|---|---:|---:|---:|---:|---:|---|---|
| 2026-07-05 | PQ7 / #274 | CI #201 | `tests/fixtures/pst/sample.pst` | `root_pages_in_bounds` via `unicode_root_bref_offsets` | 50 | 63 | 11 | 1 | 0 | `metadata_candidates_from_message_nodes; pq6_status=property_body_coverage` | No public-fixture extraction count delta: String8 selected-property aliases are now supported, but the public fixture still reports 0 selected properties, 74 unknown properties, 0 body payloads, and 1 body fallback row. This rules out ANSI/String8 aliases as the explanation for the current selected-property gap. Next blocker is PQ8 lower-level property-context layout/tag interpretation before body, attachment, or recipient expansion. |
| 2026-07-05 | PQ6 / #268 | CI #192 | `tests/fixtures/pst/sample.pst` | `root_pages_in_bounds` via `unicode_root_bref_offsets` | 50 | 63 | 11 | 1 | 0 | `metadata_candidates_from_message_nodes; pq6_status=property_body_coverage` | Measurement progress: the true message candidate has a loadable property context, but all 74 parsed properties are currently unknown to the selected MAPI dictionary. No supported body properties were found, so body payload records remain 0 and one body fallback row is emitted. Next blocker is PQ7 selected property dictionary expansion before attachment/recipient expansion. |
| 2026-07-05 | PQ5 / #262 | CI #186 | `tests/fixtures/pst/sample.pst` | `root_pages_in_bounds` via `unicode_root_bref_offsets` | 50 | 63 | 11 | 1 | 0 | `metadata_candidates_from_message_nodes; pq5_status=message_table_candidates_linked` | Fidelity progress: message output no longer counts every decoded NBT entry as a message. PQ5 identified 1 decoded message candidate and 11 table candidates, with 8 table candidates linked to decoded folder candidates and 3 unlinked. The message-count drop from 63 to 1 is an over-count correction, not a regression. Next blocker is PQ6 property context and body coverage for the true message candidate set. |
| 2026-07-05 | PQ4 / #247 | CI #179 | `tests/fixtures/pst/sample.pst` | `root_pages_in_bounds` via `unicode_root_bref_offsets` | 50 | 63 | 11 | 63 | 0 | `metadata_candidates_from_node_index; pq4_status=decoded_folder_candidates` | Progress: decoded 10 folder candidates, loaded properties for all 10, and moved folder output from root-only to 11 folder rows. Next blocker is PQ5 message table discovery and folder-message membership; body and attachment payload counts remain unchanged. |
| 2026-07-05 | PQ3 baseline / #200 | CI #173 | `tests/fixtures/pst/sample.pst` | `root_pages_in_bounds` via `unicode_root_bref_offsets` | 50 | 63 | 1 | 63 | 0 | `metadata_candidates_from_node_index` | First structured baseline after PQ3: BBT/NBT traversal works, messages are discovered/extracted as metadata candidates, and the next blocker is folder/message fidelity plus body and attachment payload coverage. |
| 2026-07-05 | PQ3 / #199 | CI #171 | `tests/fixtures/pst/sample.pst` | Captured by CLI smoke only | n/a | n/a | n/a | n/a | n/a | CI fixture inspect/extract passed | Full structured public-PST artifact was added immediately after PQ3 so future milestones can log comparable metrics. |

## Completion report format

Every milestone completion report should include a section like this:

```text
Public PST progress:
- Fixture: tests/fixtures/pst/sample.pst
- Root condition: <value>
- BBT entries: <value>
- NBT entries: <value>
- Folders/messages/attachments: <folders>/<messages>/<attachments>
- Change vs previous milestone: <progress|unchanged|regression>
- Next blocker: <short statement>
```
