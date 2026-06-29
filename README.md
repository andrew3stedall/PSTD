# PSTD

PSTD is a Rust-first PST email data extraction tool. The v1 command is `pstd`.

The project is being built milestone-by-milestone through a phone-first GitHub workflow. M1, M2, and M3 are implemented on `main`, but local validation has not yet been run.

## Current status

| Milestone | Status | Delivered |
|---|---|---|
| M1: Extraction Foundation and Archive Contract | Implemented, validation deferred | Rust CLI, TAR/JSONL writers, output records, Python wrapper, Docker scaffold |
| M2: PST Binary Foundation | Implemented, validation deferred | Bounded byte reader, PST header parser, typed primitives, BBT/NBT skeletons, `pstd inspect` |
| M3: Folder and Metadata Extraction | Implemented, validation deferred | Logical metadata layer, heap/BTH/property/table scaffolds, metadata-only archive output |
| M4: Recipients, Threading, and Address Resolution | Next | Recipient rows, reference rows, conversation fields, address resolution |

## What works now

The repository contains an initial `pstd` command and parser scaffolding for:

```text
pstd inspect --input <pst-file>
pstd inspect --input <pst-file> --json
pstd extract --input <pst-file> --output <output-dir> --manifest-only
```

The current extraction path is metadata-only. It writes structured TAR + JSONL output and records unsupported or incomplete areas explicitly.

## What is not implemented yet

- Full recipient extraction.
- Full threading extraction.
- Email body extraction.
- Attachment extraction.
- Snowflake loading.
- Search or web UI.

## Required local validation

Run before treating the current implementation as release-ready:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd --help
pstd inspect --help
pstd inspect --input <approved-small-fixture.pst>
pstd extract --input <approved-small-fixture.pst> --output <tmp-output> --manifest-only
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

Do not commit private PST files. Use synthetic byte fixtures in tests and approved small PST fixtures only in local or secure fixture storage.

## Start here

- [Documentation index](docs/README.md)
- [Developer guide](docs/engineering/developer-guide.md)
- [Codebase map](docs/engineering/codebase-map.md)
- [System overview](docs/architecture/system-overview.md)
- [Validation guide](docs/operations/validation-guide.md)
- [Wiki home](docs/wiki/README.md)
- [Project status](docs/product/project-status.md)
- [PSTD v1 Roadmap](docs/product/pstd-v1-roadmap.md)
