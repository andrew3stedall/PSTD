# Vertical 14: project complete recipient records in one run

## Context

The public PST fixture has separately validated four recipient roles, four display names, and four `PidTagEmailAddress` values. PR #429 added strict row-aligned assembly, but production-capable code still selected only one recipient identity property per projection.

Combining values from separate fixture runs is insufficient evidence because it does not prove that role, name, and address were recovered from the same rows and heap during one parser invocation.

## Requirement

Add one callable, fail-closed projection that:

1. uses the same validated row payload, row offsets, bitmap masks, and Table Context heap;
2. resolves `PidTagDisplayName` independently;
3. resolves the preferred available address property (`PidTagSmtpAddress`, then `PidTagEmailAddress`);
4. combines both projections with validated `PidTagRecipientType` evidence;
5. emits complete row-aligned recipient records only when all sequences are complete and equal in length.

## Implementation

`tc_complete_recipient_projection::project_complete_recipient_records` filters the authoritative descriptor set into two views while preserving each descriptor's original bitmap bit and row offset:

- display-name descriptors only;
- SMTP/native-address descriptors only.

Each view passes through the existing bounded row-to-HNID-to-Heap-on-Node string projection. The resulting diagnostics are then supplied to `assemble_complete_recipient_records`.

The output retains:

- display-name diagnostic evidence;
- address diagnostic evidence;
- complete role/name/address/address-kind records;
- bounded status publication.

No row or heap payload bytes are serialized.

## Safety behavior

The projection remains fail closed. Missing descriptors, incomplete bitmap coverage, invalid row transport, node-resident HNIDs, malformed strings, semantic property mismatch, or unequal row counts produce no partial complete records.

`PidTagEmailAddress` remains classified as `native_email_address`; it is not relabelled as SMTP based on its textual appearance.

## Regression evidence

The focused test constructs two recipient rows and one Table Context heap containing both display names and native addresses. A single invocation produces:

- `To / Recipient 1 / to1@domain.com / native_email_address`;
- `Cc / Recipient 2 / to2@domain.com / native_email_address`.

This proves the one-run projection boundary without combining evidence captured during separate runs.

## Extraction impact

Confirmed public-fixture baseline before this change:

- messages extracted: 1;
- recipient roles: 4;
- display names: 4;
- native email-address values: 4;
- attachments: 0;
- EML files: 0;
- output bytes: 35,332.

This milestone adds a single-call complete-recipient capability. It does not yet claim that `TcHeapDiagnostic` publishes the four complete records in the public fixture; that requires direct production reporting integration and CI fixture evidence.

## Next vertical milestone

Attach `project_complete_recipient_records` to `TcHeapDiagnostic` and publish its bounded status fragment during the public PST run. Acceptance requires all four role/name/address/address-kind records in one fixture execution. No further recipient helper or diagnostic type is justified before that integration.
