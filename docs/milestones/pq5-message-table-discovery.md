# PQ5 Message Table Discovery

## Goal

Improve message-level conversion fidelity by distinguishing decoded message nodes from folders, tables, and other NBT entries.

## Scope

PQ5 is bounded to issues #248 through #261.

| Issue | Scope | Status |
|---:|---|---|
| #248 | PQ5 parent scope | Implemented in this PR |
| #249 | Message table discovery evidence | Implemented |
| #250 | Folder/message membership status | Implemented as explicit evidence/fallback status |
| #251 | Docs and public PST progress | Implemented; final artifact logged before merge |
| #252 | Tests and CI validation | Implemented |
| #253 | PQ6 next-blocker notes | Implemented |
| #254 | Public PST progress artifact | Completed from CI artifact before merge |
| #255 | Merge if green | Completed after final-head CI is green |
| #256-#261 | Scope and PR closure guardrails | Completed through this PR |

## Delivered behaviour

- Normal and associated message NBT entries are classified as message candidates.
- Contents, associated contents, search contents, and hierarchy table nodes are classified as table evidence.
- Non-message NBT entries are excluded from message output.
- Table candidates are linked to PQ4 folder candidates where the derived owner folder exists.
- Extraction status records PQ5 counters and membership evidence status.
- Message rows carry explicit `pq5_status` evidence in `metadata_status`.

## Explicit non-goals

- Full message table row decoding.
- Body payload completeness.
- Attachment payload completeness.
- Recipient expansion.
- Downstream Snowflake, UI, search, or analytics work.

## Next milestone

PQ6 should target property context and body coverage. After PQ5, message counts should be based on message-like NBT nodes rather than broad NBT enumeration, so the next useful quality measure is how many of those message candidates have usable properties and body payloads.
