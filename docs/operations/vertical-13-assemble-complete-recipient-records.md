# Vertical 13: assemble complete recipient records

## Objective

Retain recipient role, display name, address, and authoritative address kind together for each validated Table Context row.

## Evidence entering this run

The public PST fixture has independently validated four role values (`to`, `to`, `cc`, `cc`), four `PidTagDisplayName` values (`Recipient 1` through `Recipient 4`), and four `PidTagEmailAddress` values (`to1@domain.com`, `to2@domain.com`, `cc1@domain.com`, `cc2@domain.com`). The existing record boundary retained only one identity string, so selecting addresses displaced display names.

## Implementation

`assemble_complete_recipient_records` accepts the validated role, display-name, and address diagnostics and joins them strictly by row order. It requires authoritative property names, equal non-zero row counts, and either `PidTagSmtpAddress` or `PidTagEmailAddress` for the address sequence.

The resulting record contains:

- row index
- recipient role
- display name
- address
- address kind (`smtp_address` or `native_email_address`)

Any missing evidence, property mismatch, or row-count mismatch produces no partial records. The existing two-input role/identity API remains available for compatibility.

## Expected fixture result

The validated fixture evidence can be represented as:

1. To — Recipient 1 — to1@domain.com — native email address
2. To — Recipient 2 — to2@domain.com — native email address
3. Cc — Recipient 3 — cc1@domain.com — native email address
4. Cc — Recipient 4 — cc2@domain.com — native email address

This run adds the assembly boundary and bounded status format. Production Table Context reporting still needs to retain display-name and address diagnostics concurrently before this result can be emitted from one fixture execution.

## Tests

Focused tests cover the four fixture-shaped complete records, authoritative property validation, address-kind classification, stable bounded publication, and fail-closed row-count mismatches.

## Next vertical milestone

Update production Table Context reporting to project `PidTagDisplayName` and the preferred address property independently during the same execution, then call this assembler and publish the four complete records. Do not infer or combine values from separate runs.
