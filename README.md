# PSTD

PSTD is a Rust-first PST email data extraction tool. The primary command is `pstd`.

The M1-M25 v1 foundation is complete. Active development is now focused exclusively on increasing PST conversion coverage through evidence-led parser-quality (PQ) milestones. The merged implementation has reached **PQ37**.

## Current status

| Workstream | Status | Current result |
|---|---|---|
| M1-M25 product foundation | Complete and CI validated | CLI, structured TAR/JSONL outputs, folder/message records, body and attachment foundations, batch/resume, diagnostics, Docker support, and operator handoff. |
| PQ1-PQ35 parser discovery | Complete and CI validated | Corrected root/index traversal, real folder and message candidate discovery, property/subnode diagnostics, Unicode SLBLOCK decoding, and recursive SLENTRY target resolution. |
| PQ36 payload decoding and admission | Complete and CI validated | Decodes `NDB_CRYPT_PERMUTE` data blocks, classifies Heap-on-Node clients, and prevents structurally invalid payloads from entering the legacy table fallback. |
| PQ37 TCINFO root parser | Complete and CI validated | Adds a bounded parser for the 22-byte table-context root and exact TCOLDESC records while preserving unresolved HNID references. |
| PQ38 | Next | Resolve the real `hidUserRoot` heap allocation, parse the fixture TCINFO from that allocation, and emit evidence before row materialisation. |

## Public fixture progress

The checked-in public PST remains the primary conversion-quality signal.

The most important corrected result arrived in PQ36:

- selected properties increased from **0 to 16**;
- unknown properties decreased from **74 to 19**;
- text and RTF body payloads were recovered;
- fallback body rows decreased from **1 to 0**;
- false table declarations were rejected;
- BID `0x74` remains an unresolved 208-byte payload and must not be assumed to be the row matrix.

PQ37 intentionally adds parser primitives only, so it does not change extraction output. Its purpose is to make PQ38 reference resolution safe and specification-aligned.

## What works now

```text
pstd version
pstd inspect --input <pst-file>
pstd inspect --input <pst-file> --json
pstd extract --input <pst-file> --output <output-dir>
pstd batch --input <pst-file-or-directory> --output <output-dir>
python -m pstd --help
```

Current capabilities include:

- bounded PST header, BBT, NBT, block, subnode, Heap-on-Node, BTH, SLBLOCK, and table-context parsing;
- explicit parser limits and cycle guards;
- folder and true message-candidate discovery;
- selected MAPI property extraction and unknown-property diagnostics;
- text, HTML, RTF, transport-header, recipient, threading, and attachment output foundations;
- structured TAR + JSONL outputs with run and batch status;
- public-fixture CI artifacts that expose parser progress and the next measured blocker.

## Current limitation

PSTD is not yet an absolute-coverage PST-to-EML converter. The public fixture now yields materially better property and body extraction, but table-context reference resolution and row materialisation are still incomplete. Snowflake ingestion, UI, search, and downstream analytics remain parked until conversion fidelity is reliable.

## Required validation gate

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

Do not commit private PST files. Use synthetic byte fixtures in tests and approved public or sanitised PST fixtures only.

## Start here

- [Documentation index](docs/README.md)
- [Project status](docs/product/project-status.md)
- [Public PST progress log](docs/operations/public-pst-progress-log.md)
- [PSTD v1 roadmap](docs/product/pstd-v1-roadmap.md)
- [Developer guide](docs/engineering/developer-guide.md)
- [Codebase map](docs/engineering/codebase-map.md)
- [System overview](docs/architecture/system-overview.md)
- [Local validation guide](docs/operations/local-validation.md)
