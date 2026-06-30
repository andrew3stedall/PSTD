# PSTD v1 M4: Recipients, Threading, and Address Resolution

## Goal

Extend the metadata-only PSTD foundation so message archives include recipient/address records and threading fields that can be used to reconstruct conversations and address graphs.

## Scope

M4 is still metadata-only. It does not extract message bodies, attachments, Snowflake outputs, or UI features.

## Delivered

1. Selected MAPI property registry entries for transport headers, internet message IDs, conversation fields, recipient/address fields, and Exchange/X.400 raw addresses.
2. Threading helpers for normalized subjects, message-reference splitting, and threading status calculation.
3. Recipient row conversion from `TableContext` into `RecipientRecord` rows with stable keys and ordinal ordering.
4. `data/recipients.jsonl` and `data/message_references.jsonl` emitted into the archive contract even when empty.
5. Status fields that distinguish available, absent, deferred, and unsupported recipient/threading data.
6. Unit tests for subject normalization, reference splitting, recipient type mapping, SMTP preference, and raw Exchange-style address preservation.
7. Fixture validation through existing CI when a `.pst` fixture is available.

## Out of scope

- Body extraction.
- Attachment extraction.
- Full BBT/NBT traversal beyond the current foundation.
- Full Exchange directory resolution.
- Snowflake ingestion, search indexing, or web UI.

## Implementation notes

- Current M4 converts recipient rows after they are represented as `TableContext` rows.
- Current BBT/NBT and table traversal remain skeleton-level, so broader real-world recipient extraction still depends on deeper PST traversal work.
- Exchange/X.400-style raw addresses are preserved when SMTP cannot be resolved.
- Explicit SMTP fields are preferred over raw Exchange-style addresses when both are present.

## Acceptance criteria

- Existing M1-M3 CI remains green.
- `data/recipients.jsonl` is present in TAR output.
- `data/message_references.jsonl` is present in TAR output.
- Message rows use M4 threading status values instead of only `deferred_to_m4` once fields are attempted.
- Unit tests cover normalized subject and references parsing.
- Unit tests cover recipient row conversion and address resolution status.
- Missing recipient/threading data is explicit rather than silent.

## Validation commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- inspect --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```
