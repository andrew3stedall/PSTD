# PSTD v1 M22: Body and Header Fidelity Expansion

## Goal

Reduce reachable v1 body/header fidelity gaps while preserving deterministic output contracts and fallback statuses.

## Selected fidelity gaps

| Gap | M22 change | Status impact |
|---|---|---|
| Unicode HTML body properties | Add selected MAPI coverage for `PR_HTML` as a Unicode/string property and emit an `html` body payload when present. | Preserves body archive path, size, hash, encoding, and `extracted` status. |
| Transport headers | Add `transport_message_headers` to `MessageRecord` and populate it from `PR_TRANSPORT_MESSAGE_HEADERS`. | Missing headers stay `None`; status rows stay deterministic. |

## Scope

M22 is intentionally narrow. It improves data already reachable through the existing property-context path. It does not add a new parser layout, RTF decompression, MIME reconstruction, or EML output.

## Deliverables

1. Unicode/string HTML body property support.
2. Tests for Unicode HTML body extraction.
3. Binary HTML precedence when both binary and Unicode HTML properties are present.
4. `transport_message_headers` in message JSONL records.
5. Tests for populated and absent transport-header paths.
6. Documentation updates for the output contract, milestone, implementation plan, issue plan, roadmap, PRD, project status, README, and changelog.

## Out of scope

- EML as the canonical output format.
- RTF decompression.
- Header parsing into structured From/To/CC rows.
- Attachment payload fidelity work reserved for M23.
- Snowflake, search, embeddings, graph, or web UI work.
- Broad PST parser rewrites unrelated to body/header fidelity.

## Acceptance criteria

- Unicode HTML body properties produce deterministic `html` body payloads.
- Existing binary HTML body extraction still works.
- Binary HTML remains preferred when both binary and Unicode HTML are present.
- Transport headers are surfaced on message records when available.
- Missing transport headers remain explicit as `None`.
- Existing M1-M21 behaviour remains CI green.

## Validation commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- batch --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```
