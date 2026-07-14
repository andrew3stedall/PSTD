# PSTD Developer Guide

_Last reviewed: 14 July 2026._

## Purpose

Give developers and coding agents the minimum context needed to change PSTD safely without reintroducing invalid parser assumptions or confusing historical plans with current capability.

## Read first

1. [Root README](../../README.md)
2. [Project Status](../product/project-status.md)
3. [Public PST Progress Log](../operations/public-pst-progress-log.md)
4. [Roadmap](../product/pstd-v1-roadmap.md)
5. [Codebase Map](codebase-map.md)
6. [Local Validation](../operations/local-validation.md)
7. `AGENTS.md`

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

The M1-M25 milestone lane and PQ1-PQ74 parser-quality lane are complete. Active work uses vertical extraction milestones.

A vertical milestone must:

- expose one new observable extraction behaviour or remove one concrete blocker;
- reuse existing validated storage and parser components;
- preserve row order, property identity, address kind, encoding, and source boundaries;
- fail closed without partial evidence;
- remain tightly scoped;
- include focused regression tests;
- rerun the public fixture and update current-state documentation.

Do not add a new abstraction merely because a parser layer could be made more general. It must unlock a measured extraction need.

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
- Prefer synthetic byte fixtures for unit and regression tests.
- Use only approved public or sanitised PST files for integration checks.
- Keep CI artifacts bounded and free of full bodies or attachment bytes.
- Treat a passing fixture as evidence for that layout, not general compatibility.

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

On `main`, the public fixture validates 50 BBT entries, 63 NBT entries, 11 folders, one extracted message, two body payloads, zero attachments, four 52-byte Table Context rows, and four complete recipient records at the assembly boundary.

The current remaining recipient boundary is same-run production projection and publication. Draft PR #430 addresses the same-run projection but is not merged capability.

## Failure rules

- Use checked arithmetic for offsets, counts, and lengths.
- Reject partial or non-binary bitmap evidence.
- Reject duplicate/out-of-range descriptor mappings.
- Do not decode unsupported MAPI types as if they were known.
- Do not treat native Exchange or `PidTagEmailAddress` values as SMTP without authoritative evidence.
- Do not combine names and addresses from separate fixture executions.
- Suppress partial records when evidence counts or properties disagree.
- Preserve explicit unavailable, failed, unsupported, and partial states.

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

Update current truth in the root README, project status, public progress log, roadmap, and affected technical guide. Add a point-in-time vertical record for the implementation. Historical milestone/PQ files should remain accurate records of their original decision boundary rather than being rewritten to appear current.
