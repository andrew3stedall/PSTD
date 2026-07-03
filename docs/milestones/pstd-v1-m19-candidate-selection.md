# PSTD v1 M19: Focused Decoder Candidate Selection

## Goal

Convert reviewed issue candidates into a deterministic candidate selection plan so the next parser milestone starts from one focused, testable slice rather than broad decoder expansion.

## Scope

M19 adds selection outputs on top of M18 review records. It does not add new PST parser support. It ranks candidates, marks high-priority candidates as selected for next planning, and records implementation scope, test expectation, and fallback requirements.

## Deliverables

1. `DecoderCandidateSelection` records.
2. Priority-aware candidate ordering.
3. Selection statuses:
   - `selected_for_next_planning`
   - `hold_for_high_priority_review`
   - `hold_for_more_evidence`
4. Candidate-specific implementation scope text.
5. Candidate-specific test expectation text.
6. Candidate-specific fallback requirement text.
7. `data/decoder_candidate_selection.jsonl` archive output.
8. Manifest row for candidate selection output.
9. Extraction status counters for candidate selection and selected candidates.
10. Documentation, ordered issue plan, changelog, project status updates, and CI validation.

## Out of scope

- New PST decoder expansion.
- Automatic GitHub issue creation.
- Search, Snowflake, or web UI work.
- Distributed execution.
- Treating unknown layouts as decoded.

## Selection rules

Candidates are sorted by:

1. Priority: high before medium before low.
2. Observed total: larger first.
3. Affected message count: larger first.
4. Candidate key: lexical tie-breaker.

## Acceptance criteria

- Existing M1-M18 CI remains green.
- Empty candidate lists produce empty selection output.
- High-priority candidates are marked selected for next planning.
- Medium-priority candidates are held while high-priority candidates exist.
- Selection records include scope, test expectation, and fallback guidance.
- Extraction archives include `data/decoder_candidate_selection.jsonl`.
- Project status and changelog describe M19 status.

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
