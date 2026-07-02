# PSTD v1 M15: Observed Layout Compatibility and Public Fixture Triage

## Goal

Make fixture-driven parser work safer by turning observed subnode and attachment-table diagnostics into explicit compatibility triage records and safe follow-up guidance.

## Scope

M15 builds on M13 and M14. It does not add new PST layout decoders unless a layout is already represented by tested synthetic structures. It provides a consistent way to decide what needs parser work, fixture coverage, or payload mapping follow-up.

## Deliverables

1. Observed layout triage structures for supported, partial, and unsupported layouts.
2. Compatibility cases with category, severity, status, observed count, and recommended follow-up.
3. Triage summary for subnode layout reports and attachment table wiring reports.
4. Synthetic tests for supported layouts, unsupported layouts, attachment table parse errors, missing payloads, and empty reports.
5. Safe fixture triage guidance for public or sanitized PST samples.
6. Documentation, issue plan, changelog, project status updates, and CI validation.

## Out of scope

- Snowflake ingestion.
- Search indexing.
- Web UI.
- Distributed execution.
- Committing mailbox content.
- Broad PST parser rewrites.
- Treating unknown layouts as decoded.

## Execution order

1. Add compatibility triage module.
2. Add synthetic triage tests.
3. Add safe fixture triage guidance.
4. Update docs and issue tracking.
5. Validate through CI.

## Acceptance criteria

- Existing M1-M14 CI remains green.
- Compatibility triage distinguishes supported layouts from parser-work cases.
- Attachment table parse errors are surfaced as parser triage cases.
- Attachment rows without payloads are surfaced as payload triage cases.
- Empty reports are handled explicitly.
- Public/sanitized fixture guidance is documented.

## Validation commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- batch --help
cargo run -- inspect --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```
