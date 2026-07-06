# PQ10 Heap BTH Property Traversal

## Goal

Add heap-backed property-context traversal and expose whether the public fixture uses it.

## Scope

PQ10 is bounded to issues #287 through #291.

| Issue | Scope | Status |
|---:|---|---|
| #287 | Parent scope | Implemented in this PR |
| #288 | Heap/BTH traversal records | Implemented |
| #289 | Property-context heap/BTH extraction | Implemented |
| #290 | Docs and public PST progress | Implemented |
| #291 | Validation and merge | Completed after final-head CI is green |

## Delivered behaviour

- Heap-on-node header and page-map parsing.
- Heap HID allocation resolution.
- Heap-backed BTH property-context entry parsing.
- MAPI property tag reconstruction from property ID plus property type.
- Existing flat BTH fallback preserved.
- PQ10 traversal selection visible in public progress status.

## Public fixture result

CI #235 against `tests/fixtures/pst/sample.pst` produced 50 BBT entries, 63 NBT entries, 11 folders, 1 message, and 0 attachments.

The property-context signal stayed at 0 plausible property tags and 70 suspicious property keys. Traversal status reported 0 heap BTH contexts, 1 legacy flat BTH context, and 0 unknown traversal contexts.

## Next milestone

The measured next blocker is `heap_hn_header_or_bth_root_detection`.
