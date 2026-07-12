# PQ66 — Validated row transport evidence

## Repository state reviewed

PQ65 established a fail-closed conversion from validated direct or ordinal row references into absolute byte offsets. The public fixture evidence remains four ordinal references (`0,1,2,3`), a 52-byte row width, and one 208-byte row payload, implying absolute offsets `0,52,104,156`.

The earlier PQ66 proposal combined address-mode selection inside row resolution, payload retention, reporting status, and public diagnostic publication. That scope would make it difficult to distinguish an addressing defect from a reporting defect. PQ66 therefore establishes the bounded transport object first.

## Change

`build_validated_row_transport` accepts:

- one already resolved row payload;
- validated row references;
- a validated non-zero row width;
- an explicit direct-offset or ordinal-index address mode.

It calls the PQ65 offset helper and returns `TcRowTransportEvidence` only when all rows fit completely inside the payload. The evidence retains:

- the complete payload bytes;
- the complete absolute row-offset vector;
- row width;
- address mode.

The payload is copied only after offset validation succeeds. Failures return no evidence object and therefore no retained payload or partial offset vector.

## Validation coverage

Tests cover:

- ordinal references `0,1,2,3` producing offsets `0,52,104,156` for a 208-byte payload;
- direct offsets remaining unchanged;
- out-of-bounds ordinal rows failing closed;
- empty references failing closed.

## Safety boundary

PQ66 does not:

- choose direct or ordinal mode from unvalidated parser state;
- locate a payload among multiple candidates;
- attach evidence to `TcSubnodeRowResolutionReport`;
- select a TCINFO descriptor;
- read or interpret a property value;
- change extraction counts.

## Requirements for PQ67

PQ67 should integrate the transport builder into `resolve_subnode_row_storage` only after the existing layout analysis has selected exactly one validated mode and exactly one matching payload. It should expose explicit validated, unavailable, or construction-failed status and retain neither payload nor offsets outside the validated path.

PQ68 can then connect the PQ64 fixed-width selector to reporting using transported descriptors, masks, payload bytes, absolute offsets, row width, and the fixed-data boundary.
