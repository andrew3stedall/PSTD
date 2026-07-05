# PQ2 Root Candidate Selection

## Purpose

PQ2 extends PQ1 diagnostics from a single decoded root pair into a candidate-selection model. The goal is to decide whether root traversal can safely proceed before downstream extraction or Snowflake loading work.

PQ2 keeps the legacy early-header root fields as a diagnostic candidate, then adds later Unicode root BREF offset candidates. If a candidate pair has both BBT and NBT root pages fully within file bounds, PSTD selects that source for traversal.

## Inspect output

`pstd inspect --json` now exposes:

- `root_diagnostic_condition`
- `header.bbt_root_offset` and `header.nbt_root_offset` as the selected traversal offsets, when available
- `header.root_diagnostics.selected_source`
- `header.root_diagnostics.candidate_count`
- `header.root_diagnostics.candidates[]`

Each candidate includes:

- `source`
- `bbt_root`
- `nbt_root`
- `selectable_for_traversal`
- `condition`

## Candidate sources

| Source | Meaning |
|---|---|
| `unicode_root_bref_offsets` | Later Unicode root BREF offset fields. Preferred when complete and in bounds. |
| `legacy_header_fields` | Earlier decoded offsets retained for compatibility and diagnostics. |

## Selection rules

1. Evaluate later Unicode root BREF offsets when present.
2. Evaluate the legacy early-header offsets.
3. Select the first candidate whose BBT and NBT root pages are both fully in bounds.
4. If no candidate is usable, leave traversal roots unavailable and preserve all candidate diagnostics.

## Conditions

| Condition | Meaning |
|---|---|
| `root_pages_in_bounds` | A candidate pair was selected and traversal can be attempted. |
| `root_candidates_unusable` | Candidates were decoded, but no full BBT/NBT root pair is safe for traversal. |
| `root_pages_truncated` | At least one candidate starts in the file but cannot provide a complete root page. |
| `root_pointers_absent` | No usable root pointers were decoded. |

## How to run

```bash
cargo run -- inspect --input tests/fixtures/pst/sample.pst --json
```

For public fixture work, inspect before extract:

```bash
cargo run -- inspect --input /tmp/pstd-public-fixtures/example.pst --json
```

Only proceed to extraction-quality work if `root_diagnostic_condition` is `root_pages_in_bounds` and `selected_source` is present.

## Boundary

PQ2 does not guarantee full BBT/NBT page decoding. It only selects a safe root candidate pair or classifies why traversal should not proceed.

The next quality step is PQ3: fixture-backed root traversal validation.
