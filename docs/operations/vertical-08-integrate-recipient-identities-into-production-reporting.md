# Vertical 08: integrate recipient identities into production reporting

## Objective

Connect the validated recipient identity projection to the production Table Context reporting path so the public PST fixture publishes real recipient strings or an exact bounded failure reason.

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

## Public fixture acceptance evidence

GitHub Actions run #538 completed successfully on commit `0c28984a3b35acf0d4c21ff4ec9ceb51fb763843` and produced validated recipient identity evidence:

- identity status: `tc_recipient_identity_validated`
- property tag: `0x3001001f`
- property name: `PidTagDisplayName`
- HNIDs: `0x000000a0`, `0x00000140`, `0x000001e0`, `0x00000280`
- reference kinds: `HeapId`, `HeapId`, `HeapId`, `HeapId`
- decoded strings: `Recipient 1`, `Recipient 2`, `Recipient 3`, `Recipient 4`
- failure reason: none

Combined with the already validated recipient-role sequence, the fixture contains:

1. `To` — `Recipient 1`
2. `To` — `Recipient 2`
3. `Cc` — `Recipient 3`
4. `Cc` — `Recipient 4`

This is the first production-fixture recovery of readable recipient identity values from Table Context rows. The values remain diagnostic evidence; complete recipient output records are not yet emitted.

## Before-versus-after extraction result

| Measure | Before | After |
|---|---:|---:|
| Messages discovered | 1 | 1 |
| Messages extracted | 1 | 1 |
| Body payload records | 2 | 2 |
| Recipient roles decoded | 4 | 4 |
| Readable recipient names | 0 | 4 |
| Readable recipient addresses | 0 | 0 |
| Complete recipient records emitted | 0 | 0 |
| Attachments extracted | 0 | 0 |
| EML files emitted | 0 | 0 |
| Output bytes | 33,961 | 35,332 |

The 1,371-byte increase is bounded progress and diagnostic output carrying the recipient identity evidence. Message, body, attachment, and EML counts did not regress.

## Safety properties

- Existing fixed-width, bitmap, descriptor, row-layout, and aggregate reporting fields are preserved.
- Recipient metadata is published only after complete reference and string validation.
- Node-resident references continue to fail closed rather than being interpreted heuristically.
- No row payload or heap allocation bytes are added to diagnostic output.
- Property semantics come from the exact `PidTagDisplayName` descriptor and are not inferred from string contents.

## Remaining blockers

- Assemble role and identity values into complete recipient records associated with the extracted message.
- Resolve and expose `PidTagEmailAddress` and, where present, SMTP address values.
- Emit To/Cc/Bcc fields in structured output and eventual EML reconstruction.
- Normalize HTML or RTF content.
- Extract attachment tables, payloads, and embedded messages.

## Next vertical milestone

Emit four structured recipient records by pairing the validated role sequence (`To`, `To`, `Cc`, `Cc`) with the four validated display names. The same change should attempt exact address-property resolution using the existing HNID path, but it must not block display-name record emission when address evidence is absent.