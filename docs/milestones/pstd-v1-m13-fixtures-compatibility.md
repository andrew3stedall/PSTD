# PSTD v1 M13: Payload Fixture Expansion and Parser Compatibility

## Goal

Increase confidence in payload extraction by expanding synthetic fixture coverage and improving compatibility diagnostics for attachment subnode/table layouts that cannot yet be fully decoded.

## Scope

M13 builds on M12. It remains local, bounded, and privacy-safe.

## Deliverables

1. Broader synthetic body payload coverage across text, HTML, and RTF payload paths.
2. Broader synthetic attachment payload coverage across complete, partial, missing-payload, and unparseable subnode table blocks.
3. Compatibility diagnostics for attachment subnode table parse failures, including block offsets and parse reasons.
4. Compatibility diagnostics for parsed table statuses so truncated or otherwise imperfect synthetic table layouts remain visible.
5. Documentation of remaining real-world PST compatibility risks.
6. M13 issue plan, changelog, project status updates, and CI validation.

## Out of scope

- Snowflake ingestion.
- Search indexing.
- Web UI.
- Distributed execution.
- Private PST fixtures.
- Committing any real PST with personal or confidential content.
- Full recursive child-subnode decoding beyond existing M12 bounded root-block handling.

## Execution order

1. Extend attachment subnode wiring diagnostics.
2. Add synthetic mixed-block attachment compatibility tests.
3. Add synthetic all-supported-body payload tests.
4. Update docs and issue tracking.
5. Validate through CI.

## Acceptance criteria

- Existing M1-M12 CI remains green.
- Synthetic tests cover text, HTML, and RTF body payload construction.
- Synthetic tests cover fully parsed attachment table blocks.
- Synthetic tests cover partial attachment compatibility where some blocks parse and others fail or lack payloads.
- Attachment subnode compatibility reports expose parse-error offsets and parse-error reasons.
- Project docs clearly state the remaining real-world fixture/compatibility work.

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
