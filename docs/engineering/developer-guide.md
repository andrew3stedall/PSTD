# PSTD Developer Guide

_Last reviewed: 21 July 2026._

## Purpose

Give developers and coding agents the minimum context needed to change PSTD safely without reintroducing invalid parser assumptions or confusing historical plans with current capability.

## Read first

1. [Root README](../../README.md)
2. [Project Status](../product/project-status.md)
3. [Public PST Progress Log](../operations/public-pst-progress-log.md)
4. [Roadmap](../product/pstd-v1-roadmap.md)
5. [Compatibility Matrix](../product/compatibility-matrix.md)
6. [Approved Attachment Fixture Gap](../operations/vertical-40-approved-fixture-gap.md)
7. [Codebase Map](codebase-map.md)
8. [Local Validation](../operations/local-validation.md)
9. `AGENTS.md`

Before starting work, check open pull requests, active branches, recent commits, and CI. Continue an existing vertical implementation when one is already underway.

## Repository shape

```text
src/                       Rust implementation
  cli.rs                   Command surface
  config.rs                Runtime configuration
  engine/                  Extraction orchestration
  output/                  TAR/JSONL records and writers
  pst/                     Storage, parser, projection, and extraction modules
python/                    Thin operator wrapper
docker/                    Container packaging
tests/                     Unit, regression, integration, and CLI tests
scripts/                   Fixture-progress and diagnostic helpers
docs/                      Current guidance and historical evidence
.agents/skills/            Repository-scoped reusable instructions
.github/workflows/          CI and public-fixture artifact generation
```

## Commands

```text
pstd --help
pstd version
pstd inspect --input <approved-fixture.pst>
pstd inspect --input <approved-fixture.pst> --json
pstd extract --input <approved-fixture.pst> --output <tmp-output>
pstd batch --input <approved-file-or-directory> --output <tmp-output>
python -m pstd --help
```

## Current development model

The M1-M25 milestone lane and PQ1-PQ74 parser-quality lane are complete. Active work uses vertical extraction milestones and evidence-led fixture qualification.

A vertical milestone must:

- expose one new observable extraction behaviour or remove one concrete blocker;
- reuse existing validated storage and parser components;
- preserve row order, property identity, address kind, encoding, and source boundaries;
- fail closed without partial evidence;
- remain tightly scoped;
- include focused regression tests;
- rerun every relevant approved fixture and update current-state documentation.

Do not add a new abstraction merely because a parser layer could be made more general. It must unlock a measured extraction need.

The active compatibility lane is dependency-free Unicode email expansion. The next parser or MIME change requires approved immutable fixture evidence for a second by-value attachment layout, multiple exactly owned attachments, or exact inline attachment and Content-ID behaviour. ANSI traversal and typed non-mail enrichment remain backlog-only.

## Validation

Run before claiming a branch is valid:

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

Approved fixture checks:

```text
cargo run -- inspect --input <approved-fixture.pst>
cargo run -- inspect --input <approved-fixture.pst> --json
cargo run -- extract --input <approved-fixture.pst> --output <tmp-output>
cargo run -- batch --input <approved-file-or-directory> --output <tmp-batch-output>
```

Inspect the public-progress and milestone-specific artifacts. Record the exact extraction delta, including unchanged counters when a milestone is structural or diagnostic only.

## Fixture policy

- Never commit private PST files.
- Prefer synthetic byte fixtures for focused unit and corruption tests.
- Use only approved public, redistributable, immutable or controlled synthetic PST files for integration checks.
- Record provenance, revision, path, redistribution basis, byte length, SHA-256, NDB version and crypt method before admission.
- Keep CI artifacts bounded and free of unapproved private bodies or attachment bytes.
- Treat a passing fixture as evidence for that exact layout, not general compatibility.
- Do not add another PST parser or converter as a build, runtime, test-runtime, CI or Docker dependency.

## Ownership boundaries

### Rust parser owns

- PST byte reading and bounds validation;
- header, BBT/NBT, block, node, subnode, heap, BTH, Property Context, and Table Context interpretation;
- selected MAPI decoding;
- fail-closed evidence objects.

### Rust extraction engine owns

- conversion of validated evidence into folders, messages, bodies, recipients, references, attachments, and completeness states;
- orchestration and progress reporting.

### Rust output layer owns

- deterministic stable IDs;
- JSONL records;
- raw body/attachment archive entries;
- TAR shards and summaries.

### Python owns

- operator convenience and invoking the Rust binary.

Python must not parse PST internals or duplicate the Rust extraction path.

### Future systems own

Snowflake, search, UI, tagging, graph, and LLM/RAG systems consume PSTD output. They do not parse source PST files.

## Current extraction baseline

The original public fixture validates 50 BBT entries, 63 NBT entries, 11 folders, one extracted message, two body payloads, four complete recipient records and one deterministic 956-byte plain/HTML EML.

The Tika attachment fixture validates eight messages: seven top-level messages with exact folder ownership plus one linked embedded child. It emits nine directly owned recipient records, ten body records, six valid body payloads totalling 271 bytes, two explicit unresolved HTML forms, one exact 11,862-byte method-`1` DOCX payload, one exact 453-byte method-`5` `message/rfc822` payload, the unchanged 17,035-byte parent EML and a separately emitted byte-identical 453-byte child EML.

Current approved fixture evidence does not demonstrate a second by-value attachment layout, multiple by-value attachments on one message, or an inline attachment with matching HTML `cid:` evidence. Do not implement those paths speculatively.

## Failure rules

- Use checked arithmetic for offsets, counts, and lengths.
- Reject partial or non-binary bitmap evidence.
- Reject duplicate/out-of-range descriptor mappings.
- Do not decode unsupported MAPI types as if they were known.
- Do not treat native Exchange or `PidTagEmailAddress` values as SMTP without authoritative evidence.
- Do not combine names and addresses from separate fixture executions.
- Suppress partial records when evidence counts or properties disagree.
- Preserve explicit unavailable, failed, unsupported, ambiguous and partial states.
- Do not infer inline attachment status from filenames or MIME types, and do not synthesize Content-ID values.

## Pull request checklist

Every PR should include:

- extraction objective;
- evidence entering the change;
- exact scope and exclusions;
- components reused;
- safety and fail-closed behaviour;
- tests and validation;
- public-fixture result and delta;
- output/data impact;
- remaining blocker and next vertical candidate;
- documentation updated.

## Documentation rule

Update current truth in the root README, project status, public progress log, roadmap, compatibility matrix and affected technical guide. Add a point-in-time vertical record for the implementation. Historical milestone/PQ files should remain accurate records of their original decision boundary rather than being rewritten to appear current.
