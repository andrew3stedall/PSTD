# PSTD v1 M18: Decoder Backlog Review Workflow

## Goal

Turn decoder backlog rows into reviewable summaries and issue candidates so parser work can be selected deliberately, tested first, and implemented without removing fallback behaviour for unsupported layouts.

## Scope

M18 adds reporting and workflow outputs on top of M17. It does not expand PST decoding. It groups backlog rows into review artefacts that can be used to decide which future decoder work deserves an implementation issue.

## Deliverables

1. `DecoderBacklogReviewSummary` records.
2. `DecoderIssueCandidate` records grouped by decoder candidate key.
3. Priority-aware review status such as `decoder_backlog_review_high_priority_open`.
4. Checklist guidance embedded in issue-candidate records.
5. `data/decoder_backlog_review.jsonl` archive output.
6. `data/decoder_issue_candidates.jsonl` archive output.
7. Manifest rows for both M18 review outputs.
8. Extraction status counters for issue candidates and review status.
9. Operational documentation for reviewing backlog outputs.
10. Unit tests for backlog summaries and issue candidates.

## Out of scope

- New PST decoder expansion.
- Automatic GitHub issue creation from extraction outputs.
- Snowflake, search, or web UI work.
- Distributed execution.
- Treating unknown layouts as decoded.

## Review outputs

| Output | Purpose |
|---|---|
| `data/decoder_backlog_review.jsonl` | One run-level summary of backlog counts and priority mix. |
| `data/decoder_issue_candidates.jsonl` | Grouped issue-candidate records with affected counts, suggested title, recommended action, and checklist. |

## Acceptance criteria

- Existing M1-M17 CI remains green.
- Empty backlog outputs produce an explicit empty review status.
- Non-empty backlog outputs produce grouped issue candidates.
- Review outputs are written into extraction archives.
- Manifest and status strings mention the M18 review outputs.
- Docs describe how to review candidates before parser work begins.

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
