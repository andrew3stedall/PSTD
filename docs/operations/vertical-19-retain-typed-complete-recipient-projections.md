# Vertical 19 — Retain typed complete-recipient projections

## Context

Vertical 17 added a fail-closed conversion from validated complete recipient records into the existing `RecipientRecord` output contract. Vertical 18 then returned each message-attributed `TcSubnodeProbeReport` to its caller instead of retaining it only in the run-level aggregate collector.

The remaining blocker was that `TcHeapDiagnostic` still discarded the typed `TcCompleteRecipientProjectionReport` after formatting it into diagnostic status text. Emitting structured recipients at that point would have required parsing presentation text or decoding the same Table Context twice.

## Revised milestone

Retain the already-computed complete-recipient projection as typed data on the resolved heap diagnostic while preserving current diagnostic output exactly.

This is intentionally narrower than writing recipient output records. The next run must first prove that the message-attributed caller can select exactly one validated recipient table before mutating `MetadataExtractionOutput.recipients`.

## Implementation

`TcHeapDiagnostic` now includes:

```rust
pub complete_recipients: Option<TcCompleteRecipientProjectionReport>
```

For a resolved subnode-backed Table Context heap:

1. `project_complete_recipient_records` is invoked once;
2. the returned report is retained on the diagnostic;
3. the same report is used to produce the existing bounded status fragment;
4. no diagnostic string is parsed and no second projection is performed.

Failed heap resolution stores `None` and preserves the existing fail-closed diagnostics.

## Expected fixture evidence

The public fixture should continue to report:

- one extracted message;
- four recipient roles;
- four display names;
- four native email-address values;
- four complete diagnostic records;
- unchanged output bytes and zero attachments.

This milestone does not yet claim structured recipient JSONL output.

## Safety properties

- Existing progress formatting is derived from the retained typed report, preventing divergence between typed and textual evidence.
- Failed or non-resolved table heaps cannot expose a typed complete-recipient projection.
- Existing row, bitmap, HNID, heap, string and semantic validation remains unchanged.
- Native `PidTagEmailAddress` values remain distinct from authoritative SMTP values.

## Acceptance criteria

- Rust build, tests, formatting and Clippy pass.
- Python and Docker jobs remain green.
- Public fixture metrics do not regress.
- The exact PR head is green before squash merge.

## Following milestone

Consume the returned message-attributed `TcSubnodeProbeReport`, require exactly one heap with a validated `complete_recipients` report, convert it with `message_recipients_from_complete_records`, and append the resulting records to `MetadataExtractionOutput.recipients`.

Acceptance for that run should require four structured recipient output records associated with the correct message key. Ambiguous or multiple validated recipient tables must fail closed without emitting partial recipients.
