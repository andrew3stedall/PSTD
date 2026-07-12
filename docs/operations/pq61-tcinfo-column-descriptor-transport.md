# PQ61: preserve validated TCINFO column descriptors

## Evidence basis

PQ57 exposed four exact 14-bit row masks (`11111011000000`). PQ58 validated that TCINFO bitmap indices are unique, in range, and complete. PQ59 added a bounded descriptor-to-bitmap evidence builder, and PQ60 defined a deterministic diagnostic encoding.

The remaining integration gap is not formatting. `TcHeapResolutionReport` previously retained only property tags, so downstream reporting could not safely construct PQ59 evidence without reparsing the heap or reconstructing descriptors from incomplete fields.

## Revised scope

PQ61 is limited to preserving the already validated `TcColumnDescriptor` values in the heap-resolution report. It retains, in parser order:

- raw property tag;
- data offset;
- data size;
- bitmap index.

The existing `property_tags` field remains for compatibility.

## Safety boundary

This change does not:

- build or publish public descriptor evidence;
- read fixed-width row values;
- follow HID or HNID references;
- interpret bitmap states as semantic property presence;
- change extraction counts.

## Required validation

The full repository workflow must pass on the final branch head. The public PST fixture must retain the PQ57 bitmap masks and existing extraction totals.

## PQ62 decision point

PQ62 should consume `column_descriptors` together with exact row masks in `tc_reporting.rs`, build evidence through the PQ59 helper, and publish the PQ60 encoding with an explicit complete, unavailable, or failed status. It must fail closed and emit no partial descriptor payload.
