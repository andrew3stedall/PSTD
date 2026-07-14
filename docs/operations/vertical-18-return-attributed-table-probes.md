# Vertical 18: Return attributed Table Context probes

## Context

Vertical 17 added a fail-closed conversion from validated complete recipient evidence into the existing `RecipientRecord` output model. The remaining production gap is not recipient decoding: the public fixture already exposes four complete role, display-name, and address records. The gap is preserving the typed Table Context result at the message extraction call site.

Before this change, `record_subnode_payload_probe` converted subnode payloads into a `TcSubnodeProbeReport`, stored that report only in the run-level aggregate collector, and returned nothing. The message loop therefore had no typed result it could associate with the current `message_key`. Recovering recipients later would require parsing the bounded diagnostic string, which is unsafe and couples extraction correctness to presentation formatting.

## Requirement revision

Directly appending recipients to `MetadataExtractionOutput.recipients` in this run would require either reparsing diagnostic text or independently decoding the same Table Context payload twice. Both options violate the fail-closed and no-duplicate-parsing requirements.

The smallest safe milestone is therefore to return the exact typed probe that is already recorded in the aggregate collector. This creates a message-attributed handoff without changing existing reporting behaviour.

## Implementation

`record_subnode_payload_probe` now:

1. builds one `TcSubnodeProbeReport` from the supplied `SubnodeReference` and payloads;
2. clones that report into the existing `TcRunProbeCollector`;
3. returns the original typed report to the message-level caller.

Existing callers may ignore the return value, so current extraction behaviour remains unchanged. The returned report preserves the root node ID, root subnode block ID, decoded payload count, Table Context diagnostics, and all current fail-closed status evidence.

## Regression evidence

Tests verify that:

- non-table payloads return the correct root node and subnode attribution without false table counts;
- failed table heaps retain the same attribution in both the returned probe and aggregate collector;
- the returned probe and collector aggregate agree on decoded payloads and resolved/failed table heap counts.

## Extraction impact

This milestone does not yet increase the number of structured recipient output records. It removes the unsafe handoff gap that previously prevented the message loop from consuming the already validated complete-recipient projection.

Current public-fixture baseline remains:

- one extracted message;
- four recipient roles;
- four display names;
- four native email addresses;
- four complete recipient records in production diagnostics;
- zero structured `MetadataExtractionOutput.recipients` records;
- zero attachments;
- zero EML files.

## Next milestone

The next run should retain the complete recipient projection as typed data on `TcHeapDiagnostic` rather than only embedding its status fragment. The message loop can then inspect the returned `TcSubnodeProbeReport`, require exactly one validated complete-recipient report for the current message, call `message_recipients_from_complete_records`, and append the resulting records to `MetadataExtractionOutput.recipients`.

Acceptance must require four structured recipient records from one public-fixture execution. It must fail closed for zero or multiple validated recipient tables and must not parse diagnostic strings.
