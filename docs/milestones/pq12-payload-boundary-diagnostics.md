# PQ12 Payload Boundary Diagnostics

## Goal

Expose whether the real public fixture message payload contains a heap signature beyond the original PQ11 scan window, and revise the next blocker from the measured result.

## Scope

PQ12 is bounded to issues #299 through #303.

| Issue | Scope | Status |
|---:|---|---|
| #299 | Parent scope | Implemented in this PR |
| #300 | Payload boundary diagnostics | Implemented |
| #301 | Safe alternate payload slices | Implemented as bounded scan/candidate selection |
| #302 | Docs and public PST progress | Implemented |
| #303 | Validation and merge | Completed after final-head CI is green |

## Delivered behaviour

- Bounded 4096-byte heap signature diagnostics.
- Structural distinction between no signature and signature without valid page map.
- PQ12 run-summary counters.
- Public fixture progress logging.

## Public fixture result

CI #256 against `tests/fixtures/pst/sample.pst` produced 50 BBT entries, 63 NBT entries, 11 folders, 1 message, and 0 attachments.

The property-context signal stayed at 0 plausible property tags and 70 suspicious property keys. PQ12 reported 1 no-signature payload, 0 signature-without-page-map payloads, 0 heap-failed payloads, and 0 BTH-failed payloads.

## Revised next milestone

PQ13 should focus on payload block selection and subnode resolution for the true message candidate, not more heap prefix scanning.
