# PQ6 Property and Body Coverage

## Goal

Make property-context and body extraction coverage explicit for the true message candidate set produced by PQ5.

## Scope

PQ6 is bounded to issues #263 through #267.

| Issue | Scope | Status |
|---:|---|---|
| #263 | PQ6 parent scope | Implemented in this PR |
| #264 | Property and body coverage counters | Implemented |
| #265 | Body fallback status | Implemented |
| #266 | Docs and public PST progress | Implemented; final artifact logged before merge |
| #267 | Validation and merge | Completed after final-head CI is green |

## Delivered behaviour

- Property-context coverage is counted for true message candidates.
- Body-property presence is counted separately from extracted body payloads.
- Body fallback records distinguish absent body properties from present-but-unusable properties.
- Extraction status includes PQ6 counters for property, body payload, and body fallback coverage.
- PQ6 documentation records what this milestone does and does not solve.

## Explicit non-goals

- Attachment payload expansion.
- Recipient expansion.
- Full message-table row membership decoding.
- Downstream Snowflake, UI, search, or analytics work.

## Next milestone

PQ7 should target attachment and recipient coverage after PQ6 establishes the message/property/body baseline.
