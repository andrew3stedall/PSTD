# PQ4 Folder Hierarchy Discovery

## Goal

Move conversion coverage from a synthetic root-only folder output toward deterministic folder hierarchy discovery derived from decoded PST indexes.

## Scope

PQ4 implements only issues #201 through #207:

| Issue | Scope | Status |
|---:|---|---|
| #201 | PQ4 parent scope | Implemented in this PR |
| #202 | Classify folder-like NBT entries | Implemented |
| #203 | Emit folder hierarchy progress rows | Implemented |
| #204 | Update PQ4 docs and public PST progress | Implemented; public PST row is completed from the PR artifact |
| #205 | Add folder hierarchy regression tests | Implemented |
| #206 | Re-run public PST and log PQ4 delta | Completed from CI artifact before merge |
| #207 | Prepare next conversion blocker notes | Implemented |

## Delivered behaviour

- Folder-like NBT entries are classified only from decoded node-type evidence.
- Normal folder and search folder node types emit folder candidate rows.
- Loaded property contexts can provide display name and item counts.
- Unavailable property contexts still produce deterministic fallback folder rows.
- Root-only fallback remains when no folder candidates are decoded.
- Extraction status includes PQ4 counters for folder candidates and property-context availability.

## Explicit non-goals

- Full message table fidelity.
- Body payload extraction completeness.
- Attachment payload extraction completeness.
- Recipient expansion.
- Downstream Snowflake, UI, search, or analytics work.

## Next milestone

PQ5 should target message table discovery and folder-message membership. This is the next blocker before PSTD can claim meaningful message-level conversion coverage rather than broad NBT metadata candidate coverage.
