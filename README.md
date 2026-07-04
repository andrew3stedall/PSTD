# PSTD

PSTD is a Rust-first PST email data extraction tool. The v1 command is `pstd`.

The project is being built milestone-by-milestone through a phone-first GitHub workflow. M1-M23 are implemented through milestone PRs and designed for CI validation. Two v1 milestones remain after M23: M24-M25.

## Current status

| Milestone range | Status | Delivered / purpose |
|---|---|---|
| M1-M6 | Implemented and CI validated | Extraction archive contract, PST binary foundation, folder/metadata extraction, recipients/threading, bodies/attachments foundation, batch orchestration, and resume support. |
| M7-M12 | Implemented and CI validated | Parser depth diagnostics, bounded traversal, payload/subnode traversal, payload wiring, extraction path integration, and attachment subnode integration. |
| M13-M23 | Implemented and CI validated after milestone merge | Fixture compatibility coverage, recursive layout exploration, observed layout triage, fixture-backed decoder expansion, decoder backlog reporting, backlog review workflow, candidate selection, focused candidate implementation, decoder evidence classification, Unicode HTML body extraction, transport-header metadata, and attachment metadata fidelity for unavailable payload rows. |
| M24-M25 | Remaining v1 roadmap | Batch scale/corruption hardening and v1 release-candidate handoff. |

## What works now

The repository contains the `pstd` command and supporting Rust/Python/Docker scaffolding for:

```text
pstd version
pstd inspect --input <pst-file>
pstd inspect --input <pst-file> --json
pstd extract --input <pst-file> --output <output-dir>
pstd batch --input <pst-file-or-directory> --output <output-dir>
python -m pstd --help
```

Current extraction outputs use structured TAR + JSONL records and explicit status fields. The implementation includes metadata, recipients, threading helpers, body and attachment output foundations, batch progress/checkpointing, parser diagnostics, compatibility triage outputs, decoder backlog review outputs, candidate selection outputs, focused compact attachment-table decoder coverage, compatibility evidence classification for both `CATB` and `CATW` compact decoder status families, Unicode HTML body extraction, raw transport-header metadata when available, declared attachment size/method fields, and metadata-only attachment rows for unavailable or deferred payloads.

## Remaining v1 milestones

| Milestone | Tracking issue | Purpose |
|---|---:|---|
| M24: Batch Scale, Performance, and Corruption Hardening | #139 | Harden realistic batch operation, resume behaviour, progress reporting, and recoverable failures. |
| M25: v1 Release Candidate and Operator Handoff | #141 | Close v1 with validation, documentation cleanup, and local/Docker operator handoff. |

## Out of scope for v1

- Snowflake ingestion.
- Snowpark Container Services deployment.
- Search or semantic search.
- Embeddings.
- Knowledge graph construction.
- React/web UI.
- Tagging UI or tagging storage.
- Byte-for-byte legal/archive preservation.
- External PST parsing libraries.

## Required validation gate

Run before treating the current implementation as release-ready:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- version
cargo run -- inspect --help
cargo run -- batch --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

For approved fixtures only:

```text
cargo run -- inspect --input <approved-small-fixture.pst>
cargo run -- extract --input <approved-small-fixture.pst> --output <tmp-output>
cargo run -- batch --input <approved-fixture-directory-or-file> --output <tmp-output>
```

Do not commit private PST files. Use synthetic byte fixtures in tests and approved small PST fixtures only in local or secure fixture storage.

## Start here

- [Documentation index](docs/README.md)
- [Project status](docs/product/project-status.md)
- [PSTD v1 Roadmap](docs/product/pstd-v1-roadmap.md)
- [Developer guide](docs/engineering/developer-guide.md)
- [Codebase map](docs/engineering/codebase-map.md)
- [System overview](docs/architecture/system-overview.md)
- [Validation guide](docs/operations/validation-guide.md)
- [Wiki home](docs/wiki/README.md)
