# Vertical 05: resolve recipient identity heap strings

## Goal

Turn validated recipient identity HNID references into bounded readable strings when the values are stored in the current Heap-on-Node.

## Scope

This increment adds a strict resolver for `PidTagDisplayName`, `PidTagSmtpAddress`, or `PidTagEmailAddress` reference evidence produced by Vertical 04.

The resolver:

- accepts only `PT_STRING8` and `PT_UNICODE` properties;
- requires complete row/reference-kind evidence;
- resolves only heap-resident HIDs through `HeapOnNode::allocation_by_hid`;
- decodes null-terminated UTF-16LE for `PT_UNICODE`;
- decodes bounded UTF-8 for `PT_STRING8`;
- fails closed for node IDs, malformed UTF-16, invalid UTF-8, missing allocations, or mixed incomplete evidence;
- publishes no partial recipient values after any row fails.

## Evidence

Focused tests demonstrate that two heap allocations referenced by recipient rows decode to `Alice` and `Bob`. A node-resident reference is rejected rather than guessed.

## Extraction impact

The parser now has a reusable end-to-end path from validated recipient table row bytes to readable identity strings for heap-resident values. Production fixture reporting is not yet connected, so no real fixture recipient name or address is claimed in this increment.

## Remaining gap

The next run must integrate this resolver with the production Table Context reporting path and execute the public PST fixture. If the observed references are heap IDs, it should publish the first real recipient names or addresses. If they are node IDs, the next vertical milestone must resolve the exact subnode values without adding unrelated infrastructure.
