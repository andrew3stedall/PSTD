# PSTD v1 M17 Implementation Plan

## Implementation intent

M17 improves reporting rather than parser breadth. It converts compatibility triage into a machine-readable backlog that can be reviewed after extraction runs and used to plan focused decoder work.

## Current foundation

- M15 produces compatibility triage cases for supported, partial, and parser-work scenarios.
- M16 exports `data/compatibility_triage.jsonl` and adds a focused compact attachment-table decoder.

## Implemented M17 slice

1. Decoder backlog model:
   - Adds `DecoderBacklogItem`.
   - Derives backlog rows from non-supported `CompatibilityTriageRecord` cases.
   - Skips supported cases.
2. Priority and status mapping:
   - High priority for unsupported subnode layouts.
   - High priority for unparseable attachment tables.
   - Medium priority for attachment rows without payload bytes.
   - Explicit backlog status for decoder work versus payload mapping work.
3. Extraction output:
   - Adds `data/decoder_backlog.jsonl`.
   - Adds manifest row for decoder backlog output.
   - Adds `decoder_backlog_items` to the extraction status string.

## Decoder backlog JSONL fields

Each backlog item includes:

- `run_id`
- `pst_id`
- `message_key`
- `message_node_id`
- `decoder_candidate_key`
- `category`
- `priority`
- `severity`
- `observed_count`
- `source_triage_status`
- `recommended_action`
- `backlog_status`

## Remaining work

- Promote backlog records into GitHub issues only after reviewing a safe fixture-backed run.
- Add candidate-specific issue templates if the backlog categories prove stable.
- Add new decoders only after the backlog item has a focused regression test.

## Definition of done

- M17 branch keeps CI green.
- Backlog generation tests pass.
- Extraction archive includes `data/decoder_backlog.jsonl`.
- Docs explain how backlog rows should guide future parser work.
