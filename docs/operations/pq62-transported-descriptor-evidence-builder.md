# PQ62 — Build descriptor evidence from transported columns

## Context

PQ61 preserved the validated `TcColumnDescriptor` vector in `TcHeapResolutionReport`. The existing PQ59 evidence builder still accepted only a complete `TcInfo`, while the reporting layer intentionally does not retain or reconstruct the full parser object.

Reconstructing a synthetic `TcInfo` in reporting would duplicate parser state and risk introducing values that were not validated by the heap parser. PQ62 therefore closes the interface gap before public diagnostic integration.

## Requirements

PQ62 must:

1. build descriptor/bitmap evidence directly from a transported `&[TcColumnDescriptor]` and exact row masks;
2. preserve the existing `TcInfo` entry point as a compatibility wrapper;
3. validate mask width and binary contents before indexing;
4. independently reject duplicate or out-of-range bitmap indices so future callers cannot bypass the PQ58 invariant by constructing descriptors manually;
5. preserve descriptor order while sorting output by bitmap index;
6. produce evidence identical to the established `TcInfo` path;
7. avoid reading row value bytes, following HID/HNID references, or interpreting raw bit states as semantic property presence.

## Implementation evidence

`build_descriptor_bitmap_evidence_from_columns` is now the bounded implementation. `build_descriptor_bitmap_evidence` delegates to it using `tcinfo.columns`.

Focused tests cover:

- identical evidence from `TcInfo` and transported descriptors;
- duplicate bitmap-index rejection;
- out-of-range bitmap-index rejection;
- mask-width and binary-state validation;
- deterministic descriptor formatting.

## Extraction effect

This PQ is an interface and validation change. It should not alter folder, message, body, attachment, shard, or byte totals in the public PST fixture.

The last validated extraction baseline remains:

- 14 TCINFO columns;
- four 52-byte rows;
- four identical masks, `11111011000000`;
- seven set and seven unset bits per row;
- one extracted message;
- two body payloads;
- zero attachments.

## PQ63 boundary

PQ63 should integrate the transported-column builder and deterministic formatter into `TcHeapDiagnostic`.

It should publish either:

- a complete formatted descriptor record set with an explicit validated status;
- an explicit unavailable status when exact masks do not exist; or
- an explicit construction-failed status with no partial evidence.

PQ63 must retain existing row-layout and bitmap diagnostics and must not read or interpret row values.
