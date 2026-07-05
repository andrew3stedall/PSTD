# PQ8 Property-Context Interpretation

## Goal

Improve diagnostics for the PQ7 blocker: a loaded message property context that still produces 0 selected properties and 74 unknown properties on the public fixture.

## Scope

PQ8 is bounded to issues #275 through #279.

| Issue | Scope | Status |
|---:|---|---|
| #275 | Parent scope | Implemented in this PR |
| #276 | Key-shape diagnostics | Implemented |
| #277 | Safe tag interpretation | Implemented |
| #278 | Docs and public PST progress | Implemented; final artifact logged before merge |
| #279 | Validation and merge | Completed after final-head CI is green |

## Delivered behaviour

- Added MAPI value-type shape helpers.
- Added counters for plausible property tags and suspicious property-context keys.
- Added conservative interpretation for one unambiguous key-shape case.
- Preserved unknown handling for unsupported property IDs.
- Added parser tests for the new diagnostics.

## Explicit non-goals

- Attachment payload expansion.
- Recipient expansion.
- Broad body expansion.
- Downstream Snowflake, UI, search, or analytics work.

## Next milestone

The next milestone should be chosen from the measured PQ8 public fixture result. If suspicious keys dominate, the next step should focus on heap and BTH traversal. If plausible unknown tags dominate, the next step should focus on safe dictionary expansion.
