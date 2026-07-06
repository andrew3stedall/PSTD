# PQ13 Payload Source Diagnostics

## Goal

Use the PQ12 result to decide whether the true message payload gap is data-block selection or subnode source selection.

## Scope

PQ13 is bounded to issues #305 through #309.

| Issue | Scope | Status |
|---:|---|---|
| #305 | Parent scope | Implemented in this PR |
| #306 | Data-block and subnode source diagnostics | Implemented |
| #307 | Bounded alternate source selection | Revised to measured source diagnostics |
| #308 | Docs and public PST progress | Implemented |
| #309 | Validation and merge | Completed after final-head CI is green |

## Public fixture result

CI #263 against `tests/fixtures/pst/sample.pst` produced 50 BBT entries, 63 NBT entries, 11 folders, 1 message, and 0 attachments.

The key PQ13 signal is 3 subnode references, 3 subnode decode plans, and 0 subnode decode attempts.

## Revised next milestone

PQ14 should add bounded message-level subnode source probing before dictionary/body/attachment expansion.
