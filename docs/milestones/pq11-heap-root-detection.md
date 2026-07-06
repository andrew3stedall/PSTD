# PQ11 Heap Root Detection

## Goal

Improve heap-backed property-context detection for the public PST fixture and expose the measured fallback reason.

## Scope

PQ11 is bounded to issues #293 through #297.

| Issue | Scope | Status |
|---:|---|---|
| #293 | Parent scope | Implemented in this PR |
| #294 | Heap traversal diagnostics | Implemented |
| #295 | Safe heap root detection | Implemented |
| #296 | Docs and public PST progress | Implemented |
| #297 | Validation and merge | Completed after final-head CI is green |

## Delivered behaviour

- Bounded heap candidate scanning.
- Heap signature validation.
- Indexed heap BTH traversal support.
- PQ11 heap probe counters in run status.

## Public fixture result

CI #246 against `tests/fixtures/pst/sample.pst` produced 50 BBT entries, 63 NBT entries, 11 folders, 1 message, and 0 attachments.

The property-context signal stayed at 0 plausible property tags and 70 suspicious property keys. PQ11 reported 0 offset heap contexts, 1 candidate-not-found context, 0 heap-failed contexts, and 0 BTH-failed contexts.

## Next milestone

The measured next blocker is `heap_signature_or_block_payload_prefix_detection`.
