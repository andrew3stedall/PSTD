# Vertical 07: publish recipient identity diagnostics

## Objective

Create a bounded serializable diagnostic from the validated recipient identity projection so production reporting can publish readable recipient evidence without duplicating string-resolution logic.

## Change

The diagnostic records candidate, transport, and identity status. It publishes the selected property tag, canonical name, HNID values, HNID kinds, and decoded row strings only after complete validation. Failed or unavailable projections expose no partial property metadata or strings, and no payload bytes are serialized.

## Current extraction baseline

- messages extracted: 1
- recipient rows identified: 4
- recipient roles decoded: 4
- real recipient names confirmed: 0
- real recipient addresses confirmed: 0
- attachments extracted: 0
- EML files emitted: 0

## Next acceptance boundary

Integrate the diagnostic into `TcHeapDiagnostic`, run the public fixture, and record either at least one real recipient name or address, or the exact validated reason resolution cannot complete.

## Risks

Fixture references may be node-resident, legacy string values may require code-page decoding, and complete recipient records still require role and address association.
