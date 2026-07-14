# Local Validation

_Last reviewed: 14 July 2026._

## Purpose

Define the checks required before treating a PSTD branch as valid and the additional evidence required before claiming extraction progress.

## Full repository gate

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

Do not claim validation if any command was skipped, failed, or ran against a different commit. Record deferred checks explicitly.

## Approved fixture checks

Use only approved public or sanitised fixtures:

```text
cargo run -- inspect --input <approved-fixture.pst>
cargo run -- inspect --input <approved-fixture.pst> --json
cargo run -- extract --input <approved-fixture.pst> --output <tmp-output>
cargo run -- batch --input <approved-file-or-directory> --output <tmp-batch-output>
```

Never commit private PST files or publish full body/attachment payloads in CI artifacts.

## Public fixture regression baseline

The checked-in public fixture should retain at least the current stable evidence unless the change intentionally and correctly revises it:

| Metric | Baseline |
|---|---:|
| BBT entries | 50 |
| NBT entries | 63 |
| Folders | 11 |
| True message candidates | 1 |
| Extracted messages | 1 |
| Body payloads | 2 |
| Attachments | 0 |
| Selected properties | 16 |
| Unknown properties | 19 |
| Table Context rows | 4 × 52 bytes |

Current recipient evidence includes two To and two Cc rows, display names `Recipient 1` through `Recipient 4`, and four native email-address values. A branch changing this path must show the exact previous and new bounded output.

## Milestone artifact review

For every extraction milestone:

1. open the complete CI run for the exact head SHA;
2. verify all jobs passed;
3. inspect `public-pst-progress` and the milestone-specific artifact;
4. compare extraction counts and bounded diagnostics with the previous merged baseline;
5. classify the change as material extraction progress, structural correction, diagnostic only, or regression;
6. update `docs/operations/public-pst-progress-log.md`;
7. revise the next milestone from the observed result.

A green unit suite without fixture evidence is not sufficient for a claim about real PST extraction.

## Expected single-PST output

```text
<output-root>/
  run_summary.json
  progress.jsonl
  archives/<pst-id>_000001.tar
```

The TAR may contain:

```text
_pstfast/summary.json
_pstfast/manifest.jsonl
_pstfast/errors.jsonl
_pstfast/folder_inventory.jsonl
_pstfast/extraction_warnings.jsonl
_pstfast/run_config.json
data/folders.jsonl
data/messages.jsonl
data/recipients.jsonl
data/message_references.jsonl
data/bodies.jsonl
data/attachments.jsonl
data/selected_mapi_properties.jsonl
bodies/
attachments/
```

Validate file presence against the command and extracted data. Do not assume every contract family is populated for every fixture.

## Expected batch output

```text
<batch-output-root>/
  batch_summary.json
  batch_checkpoint.jsonl
  batch_progress.jsonl
  <safe-pst-output-dir>/
    run_summary.json
    progress.jsonl
    archives/<pst-id>_000001.tar
```

Check discovered, attempted, completed, partial, failed, skipped, and not-run counters separately. Inspect checkpoint/progress streams before deleting failed-run output.

## Failure review

When a parser or fixture check fails:

- identify whether it is a code defect, stale expected artifact, environment issue, or unsupported structure;
- preserve fail-closed behaviour while fixing it;
- do not weaken validation or introduce fallback guessing merely to restore a green counter;
- add a regression test for the exact defect;
- rerun the full gate on the final head.

## Merge rule

Squash merge only when the exact PR head is green, required artifacts have been inspected, review threads are resolved, and current-state documentation reflects the measured result.
