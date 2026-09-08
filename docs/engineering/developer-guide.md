# PSTD Developer Guide

_Last reviewed: 8 September 2026._

## Purpose

Give developers and coding agents the minimum context needed to change PSTD safely without reintroducing invalid parser assumptions, confusing historical plans with current capability, or repeatedly reloading repository history during long-running implementation.

## Continuation read path

When a branch/PR already contains `docs/operations/active-implementation-checkpoint.md`, use this minimal path:

1. `AGENTS.md`.
2. `docs/operations/active-implementation-checkpoint.md`.
3. The active GitHub issue.
4. The current PR diff/latest durable implementation commit.
5. Only the source files/functions and failing checks named by the checkpoint.

Continue from `Next exact change`. Do not reread the full current-state set, parent epic, roadmap, compatibility matrix, public fixture history, or `docs/readpst-gaps/` corpus unless the checkpoint identifies a changed dependency or the active issue cites a specific evidence need.

If the checkpoint is absent or stale, reconstruct it once from the branch/PR, persist it, and then continue from the minimal path.

## Fresh-start read path

Use this broader path only when selecting/starting new implementation:

1. `AGENTS.md`.
2. [Root README](../../README.md).
3. [Project Status](../product/project-status.md).
4. [Public PST Progress Log](../operations/public-pst-progress-log.md).
5. [Roadmap](../product/pstd-v1-roadmap.md) when issue ordering is not already defined.
6. [Compatibility Matrix](../product/compatibility-matrix.md) when capability classification is needed.
7. [Codebase Map](codebase-map.md) for unfamiliar source areas.
8. [Local Validation](../operations/local-validation.md) when preparing validation.

Before creating work, check open pull requests and active branches to avoid conflict. Inspect recent commits/CI only as needed to select or safely create the implementation branch. Once the branch/PR exists, create the active checkpoint immediately and switch to the continuation read path.

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

A vertical implementation must:

- expose one new observable extraction behaviour or remove one concrete blocker;
- use one GitHub issue or smallest coherent slice as the active implementation scope;
- reuse existing validated storage and parser components;
- preserve row order, property identity, address kind, encoding, and source boundaries;
- fail closed without partial evidence;
- remain tightly scoped;
- include focused regression tests;
- make each coherent implementation increment durable before broad research or full validation;
- rerun every relevant approved fixture and update current-state documentation before merge.

Do not add a new abstraction merely because a parser layer could be made more general. It must unlock a measured extraction need.

Parent epics guide dependencies and order. They should not be treated as a requirement to load all child issues into implementation context.

## Durable implementation checkpoints

Every non-trivial active implementation branch should maintain `docs/operations/active-implementation-checkpoint.md`.

The checkpoint is intentionally short. It records the active issue, latest durable implementation commit, established conclusions, exact source boundary, focused test state, blockers, and `Next exact change`.

After each coherent code/test increment:

1. run focused formatting/tests when readily available;
2. commit the increment;
3. update the checkpoint if state materially changed;
4. only then undertake broad research, environment/toolchain work, full validation, or specialist delegation.

Keep at most one bounded increment uncommitted. A coherent unverified checkpoint commit is preferable to losing implementation state; required validation still must pass before merge.

## Validation

### During implementation

Run the smallest focused test/formatting command that exercises the changed boundary. Inspect relevant failure output and iterate. Do not run the full repository gate after every checkpoint commit unless a failure specifically requires it.

### Before claiming merge readiness

Run on the exact cleaned PR head:

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

## Content output benchmarks

Run `python scripts/benchmark_content_output.py --baseline <commit>` from a checkout
with Rust and `/usr/bin/time`. It builds the same deterministic synthetic driver
against the selected revision and current checkout, resolving one shared Cargo lockfile
for both builds. It alternates three samples of each revision, checks byte counts and
SHA-256 equality, and reports median output-phase time and process peak RSS.

The `Content output performance` workflow runs this comparison on relevant PR changes
and can be dispatched with an explicit baseline. Timing is informational rather than
a noisy CI threshold. Cases cover message-count scaling, larger bodies, attachment
text joins and disk export. The text-join case uses unsupported binary attachments to
isolate lookup and serialization; it does not measure Office/PDF parser throughput.
RSS includes synthetic input setup and buffered JSONL; these are not end-to-end PST
or constant-memory extraction benchmarks.

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

The Tika attachment fixture validates eight messages: seven top-level messages with exact folder ownership plus one linked embedded child. It emits nine directly owned recipient records, ten body records, six valid body payloads totalling 271 bytes, two explicit unresolved HTML forms, one exact 11,862-byte method-`1` DOCX payload, one exact 453-byte method-`5` `message/rfc822` payload, an inline parent EML carrying both payloads, and a separately emitted byte-identical 453-byte child EML.

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

Every merge-ready PR should include:

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

Draft PRs may use the active checkpoint for working-state handoff instead of repeatedly expanding the PR body.

## Documentation rule

Update current truth in the root README, project status, public progress log, roadmap, compatibility matrix and affected technical guide when behaviour materially changes. Add a point-in-time vertical record when useful. Historical milestone/PQ/parity-gap files should remain accurate records of their original decision boundary rather than being rewritten or reread by default on continuation.
