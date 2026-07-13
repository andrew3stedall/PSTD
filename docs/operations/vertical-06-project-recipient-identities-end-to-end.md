# Vertical 06: project recipient identities end to end

## Goal

Join the already validated recipient-row transport, HNID reference extraction, and heap string decoding into one fail-closed vertical projection.

## Why this milestone

The previous two milestones could recover recipient identity references and decode heap-resident strings independently, but callers still had to assemble the pipeline themselves. That left room for duplicated candidate selection, mismatched row offsets, or partial recipient publication.

This milestone creates one bounded path from the validated recipient table rows to readable identity strings. It deliberately does not add another public reporting format.

## Implementation

`project_recipient_identity_strings`:

1. resolves exactly one row payload candidate;
2. obtains validated absolute row offsets and row width through the existing transport metadata path;
3. extracts one supported recipient identity HNID from every selected row;
4. parses the Table Context Heap-on-Node;
5. resolves heap-resident `PT_UNICODE` or `PT_STRING8` values;
6. publishes references and strings only when the complete path succeeds.

Unavailable, ambiguous, node-resident, malformed, or out-of-bounds evidence produces no partial recipient values.

## Regression evidence

Focused tests cover:

- two recipient rows resolving end to end to `Alice` and `Bob`;
- ordinal row references becoming bounded absolute row offsets;
- Unicode heap allocation resolution;
- fail-closed handling when a selected HNID is node-resident;
- suppression of both reference and string output after failure.

## Extraction impact

The parser now has a single callable vertical path that can turn validated recipient-table rows into readable recipient identity strings. The public PST fixture reporting path is not yet wired to this projection, so no real fixture recipient name or address is claimed by this change.

Current validated baseline remains:

- messages extracted: 1;
- recipient rows and roles identified: 4 (`To`, `To`, `Cc`, `Cc`);
- real fixture recipient names confirmed: 0;
- real fixture recipient addresses confirmed: 0;
- body payload records: 2;
- attachments extracted: 0;
- EML files emitted: 0.

## Next milestone

Integrate this projection into the existing Table Context reporting call site for subnode-backed recipient rows. Record the fixture's selected identity property, HNID kinds, and decoded values.

Acceptance must be one of:

1. at least one real recipient display name or address is published from the public fixture; or
2. the fixture proves the selected identities are node-resident, with the exact NIDs recorded for the following subnode-resolution milestone.

Do not add another independent recipient abstraction before obtaining this fixture evidence.
