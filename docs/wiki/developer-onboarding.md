# Developer Onboarding

_Last reviewed: 8 September 2026._

## First decision: fresh start or continuation?

### Continuing an active implementation

Do not repeat full onboarding. Read only:

1. `AGENTS.md`.
2. Branch-local `docs/operations/active-implementation-checkpoint.md`.
3. The active GitHub issue.
4. The current PR diff/latest durable implementation commit.
5. The source files/functions and directly relevant failing checks named by the checkpoint.

Begin from `Next exact change`. Do not reread the full parent epic, project history, public fixture history, PQ records, vertical records, or readpst-gap corpus unless the checkpoint/issue points to a specific evidence need.

### Starting new implementation

1. Read `AGENTS.md`.
2. Read the [root README](../../README.md), [Project Status](../product/project-status.md), and [Public PST Progress Log](../operations/public-pst-progress-log.md) once to establish the current baseline.
3. Use the [Developer Guide](../engineering/developer-guide.md) and [Codebase Map](../engineering/codebase-map.md) only as needed for the selected source area.
4. Check open PRs/active branches to avoid conflicting work.
5. Select exactly one active issue or smallest coherent vertical slice.
6. Create the branch/draft PR and branch-local active checkpoint immediately.
7. Switch to the continuation path above.

## Current baseline

Do not duplicate live counters or active-PR claims in onboarding. Use [Project Status](../product/project-status.md) for merged capability, [Public PST Progress Log](../operations/public-pst-progress-log.md) for fixture evidence, and the active checkpoint for in-progress implementation state.

## Command surface

```text
pstd version
pstd inspect --input <approved-fixture.pst>
pstd inspect --input <approved-fixture.pst> --json
pstd extract --input <approved-fixture.pst> --output <tmp-output>
pstd batch --input <approved-file-or-directory> --output <tmp-batch-output>
python -m pstd --help
```

## Where to work

| Work type | Start with |
|---|---|
| PST storage/traversal | `src/pst/reader.rs`, header/BBT/NBT/block/logical/subnode modules |
| Property Context | `src/pst/property_context.rs`, `src/pst/mapi.rs` |
| Table Context | `src/pst/tcinfo.rs`, `tc_heap.rs`, row transport and descriptor modules |
| Recipients | recipient identity modules, `tc_recipient_records.rs`, `tc_reporting.rs` |
| Bodies/messages/attachments | corresponding modules under `src/pst/` and `src/engine/` |
| Structured output | `src/output/`, output contract summary |
| CLI or batch | `src/cli.rs`, `src/config.rs`, `src/engine/` |
| Documentation | root README, project status, progress log, roadmap, affected technical guide, changelog |

## Development workflow

1. Keep one GitHub issue/slice active at a time.
2. Reuse validated components and keep scope narrow.
3. Fail closed rather than adding heuristic fallback logic.
4. Add focused regression tests.
5. Commit each coherent implementation increment before broad research, full validation, environment setup, or another delegation.
6. Update the active checkpoint with durable commit/test state and the next exact change.
7. During iteration, run focused validation rather than the full repository gate after every commit.
8. When scope is complete, run the complete exact-head validation gate and relevant approved fixture workflows.
9. Update final current-state/point-in-time documentation where behavior changed.
10. Merge only after the cleaned exact head is green and review conditions are resolved.

## Specialist agents

Do not fan out across roles by default. Delegate only a separable question whose result can be persisted as a commit, durable issue/PR evidence, or a concise checkpoint conclusion. Persist that result before launching overlapping follow-up investigation.

## Checkpoint format

Use [Active Implementation Checkpoint Template](../operations/implementation-checkpoint-template.md). The active checkpoint is branch-local temporary execution state and should be deleted/reset when its implementation PR is completed.
