# Vertical 08: integrate recipient identities into production reporting

## Objective

Connect the validated recipient identity projection to the production Table Context reporting path so the public PST fixture publishes either real recipient strings or an exact bounded failure reason.

## Repository evidence reviewed

- Vertical 03 identified four recipient rows and decoded their roles as two To and two Cc entries.
- Verticals 04–06 added bounded HNID extraction, heap string decoding, and an end-to-end row-to-string projection.
- Vertical 07 added a fail-closed diagnostic representation but deliberately did not connect it to `TcHeapDiagnostic`.
- PR #423 merged only after its exact head passed CI.

## Implementation

`TcHeapDiagnostic` now retains `TcRecipientIdentityDiagnostic` alongside fixed-width evidence.

For every resolved subnode-backed Table Context heap, `report_table_heaps` now:

1. reuses the existing row-resolution report and bitmap masks;
2. calls `project_recipient_identity_strings` with the exact table heap bytes and base offset;
3. converts the result only through `build_recipient_identity_diagnostic`;
4. appends the bounded recipient status fragment to production progress output.

Failed heap resolution and non-subnode paths publish the explicit unavailable recipient diagnostic. No row payload or heap allocation bytes are serialized.

## Acceptance boundary

The public fixture must now produce one of two evidence-backed outcomes:

- `tc_recipient_identity_validated`, with a canonical property name, HNIDs, kinds, and decoded row strings; or
- `tc_recipient_identity_failed` / `tc_recipient_identity_unavailable`, with no partial property or string output and an exact failure reason where available.

A green build without fixture evidence is not sufficient to claim readable recipients.

## Extraction baseline

- Messages extracted: 1
- Body payload records: 2
- Recipient rows identified: 4
- Recipient roles decoded: 4 (`To`, `To`, `Cc`, `Cc`)
- Real recipient names confirmed before this change: 0
- Real recipient addresses confirmed before this change: 0
- Attachments extracted: 0
- EML files emitted: 0

## Safety properties

- Existing fixed-width, bitmap, descriptor, row-layout, and aggregate reporting fields are preserved.
- Recipient metadata is published only after complete reference and string validation.
- Node-resident references continue to fail closed rather than being interpreted heuristically.
- No payload bytes are added to diagnostic output.

## Evidence required for the following run

Inspect the public-PST CLI fixture output for:

- `recipient_identity_status`
- `recipient_property_tag`
- `recipient_property_name`
- `recipient_references`
- `recipient_reference_kinds`
- `recipient_values`
- `recipient_failure`

If real strings are published, the next vertical milestone should assemble role and identity values into complete recipient records. If the fixture reports node-resident HNIDs, the next milestone should resolve those exact subnodes. If the selected property is absent, inspect the descriptor and bitmap evidence before changing property-selection order.
