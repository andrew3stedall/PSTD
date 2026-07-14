# Vertical 15 — Validate four complete recipient records in one projection

## Purpose

PR #430 proved that display names and addresses can be projected from the same validated rows and Table Context heap. Its regression fixture contained only two rows, while the public PST currently exposes four recipient rows: two To and two Cc.

This milestone tightens the regression boundary to the exact observed public-fixture shape before production reporting is changed.

## Observed extraction evidence

The current public fixture has independently validated:

- recipient roles: `to`, `to`, `cc`, `cc`;
- display names: `Recipient 1`, `Recipient 2`, `Recipient 3`, `Recipient 4`;
- native email addresses: `to1@domain.com`, `to2@domain.com`, `cc1@domain.com`, `cc2@domain.com`;
- address property: `PidTagEmailAddress`;
- heap-resident HNIDs for all selected strings.

## Change

A fixture-shaped regression now invokes `project_complete_recipient_records` once with four rows and one shared Heap-on-Node. It verifies row-order preservation across:

1. recipient role evidence;
2. `PidTagDisplayName` values;
3. `PidTagEmailAddress` values;
4. authoritative `native_email_address` classification.

The expected records are:

| Row | Role | Display name | Address | Address kind |
|---:|---|---|---|---|
| 0 | To | Recipient 1 | `to1@domain.com` | native email address |
| 1 | To | Recipient 2 | `to2@domain.com` | native email address |
| 2 | Cc | Recipient 3 | `cc1@domain.com` | native email address |
| 3 | Cc | Recipient 4 | `cc2@domain.com` | native email address |

## Safety properties

- Both string properties are resolved from the same rows and heap.
- Bitmap positions remain tied to their original descriptors.
- Row order is preserved through record assembly.
- `PidTagEmailAddress` is not mislabeled as SMTP.
- Existing fail-closed projection and assembly behavior is unchanged.

## Extraction impact

This milestone adds no new public-fixture field. It provides exact four-row regression evidence required to safely connect the complete projection to `TcHeapDiagnostic` production output.

## Next milestone

Integrate `project_complete_recipient_records` directly into `TcHeapDiagnostic`, publish its bounded status fragment during the public-PST run, and require all four complete records to appear in one execution. No additional recipient helper layer should be introduced before that integration.
