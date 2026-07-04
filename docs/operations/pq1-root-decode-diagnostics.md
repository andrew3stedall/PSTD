# PQ1 Root Decode Diagnostics

## Purpose

PQ1 is the first post-v1 parser-quality lane. It exists to make real PST fixture failures actionable before new extraction features or Snowflake ingestion work.

The recent public fixture runs were operationally safe but stopped before folder discovery. The common failure class was impossible root offsets relative to a small PST file size. PQ1 records that condition directly in inspect output.

## What PQ1 adds

`pstd inspect --json` now includes:

- `root_diagnostic_condition` at the inspect-summary level.
- `header.root_diagnostics` with file size, expected root page size, per-root diagnostics, and a recommendation.
- Per-root fields for:
  - `offset`
  - `file_size`
  - `offset_in_bounds`
  - `root_page_in_bounds`
  - `bytes_beyond_file_size`
  - `condition`

## Root conditions

| Condition | Meaning |
|---|---|
| `root_pointers_absent` | Both decoded root pointers are absent. |
| `root_offsets_out_of_bounds` | At least one decoded root offset is beyond the PST file size. |
| `root_pages_truncated` | A decoded root offset starts in the file but a full 512-byte root page would exceed file size. |
| `root_pages_in_bounds` | Both decoded root pages are safe to attempt for tree traversal. |
| `root_diagnostics_partial` | Mixed or incomplete root diagnostic state. |

Per-root conditions use:

| Condition | Meaning |
|---|---|
| `root_pointer_absent` | No offset was decoded for this root. |
| `root_offset_beyond_file_size` | The decoded offset is greater than or equal to the file size. |
| `root_page_truncated` | The offset is in the file, but the 512-byte page would be incomplete. |
| `root_page_in_bounds` | The offset and 512-byte page are within file bounds. |

## How to run

```bash
cargo run -- inspect --input tests/fixtures/pst/sample.pst --json
```

For a downloaded public fixture:

```bash
mkdir -p /tmp/pstd-public-fixtures
curl -L -o /tmp/pstd-public-fixtures/aspose-personalstorage.pst \
  https://raw.githubusercontent.com/aspose-email/Aspose.Email-for-Java/master/Examples/src/main/resources/outlook/PersonalStorage.pst
cargo run -- inspect --input /tmp/pstd-public-fixtures/aspose-personalstorage.pst --json
```

## How to interpret results

If `root_diagnostic_condition` is `root_offsets_out_of_bounds`, do not spend time on bodies, attachments, recipients, or Snowflake loading yet. The parser has not reached safe BBT/NBT traversal.

The next engineering question is whether the fixture is incomplete/unsupported, or whether the header/root-field decoding is reading the wrong bytes or applying the wrong structure for that PST variant.

## PQ1 boundary

PQ1 is diagnostic hardening. It does not claim to fix all root traversal failures. It creates a stable evidence model for deciding the next parser-quality milestone.
