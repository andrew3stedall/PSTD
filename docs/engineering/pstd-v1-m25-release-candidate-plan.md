# PSTD v1 M25 Implementation Plan

## Implementation intent

M25 closes the planned v1 milestone lane. It is intentionally documentation and validation focused. It should not add new parser or output-contract surface unless a release-candidate blocker is found.

## Inputs

- M24 completed batch hardening.
- M1-M24 milestone docs and CI history exist.
- Current CLI surface:
  - `pstd version`
  - `pstd inspect`
  - `pstd extract`
  - `pstd batch`
- Current output contract:
  - single-PST `run_summary.json`, `progress.jsonl`, TAR shards;
  - batch `batch_summary.json`, `batch_checkpoint.jsonl`, `batch_progress.jsonl`;
  - structured JSONL records inside TAR shards.

## M25 work units

1. Add operator handoff docs:
   - local validation commands;
   - Docker build/run commands;
   - single-PST extraction runbook;
   - batch extraction runbook;
   - output review checklist.
2. Add release-candidate checklist:
   - CI pass required;
   - local checks recommended;
   - approved fixture checks recommended;
   - no private fixture commits;
   - unsupported/deferred areas acknowledged.
3. Add unsupported/deferred areas doc:
   - Snowflake ingestion;
   - UI/search/semantic search/graph/tagging;
   - distributed orchestration;
   - ANSI PST support unless evidence-backed later;
   - exact-preservation archive mode;
   - full MAPI property dump mode.
4. Update repo-wide status docs:
   - README;
   - documentation index;
   - PRD;
   - roadmap;
   - project status;
   - output contract notes;
   - changelog;
   - local validation guide.
5. Open PR, run CI, and squash merge when green.

## Validation gate

CI must pass:

```text
cargo build
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
cargo run -- --help
cargo run -- batch --help
cargo run -- inspect --help
fixture inspect/extract smoke checks when fixture exists
```

## Definition of done

- M25 PR merges with green CI.
- Issues #141, #177, #178, and #179 are closed as completed.
- Repo status states that M1-M25 are complete.
- Post-v1 begins with Snowflake ingestion planning.
