# Developer Onboarding

_Last reviewed: 14 July 2026._

## First 10 minutes

1. Read the [root README](../../README.md).
2. Read [Project Status](../product/project-status.md).
3. Read the [Public PST Progress Log](../operations/public-pst-progress-log.md).
4. Read the [Developer Guide](../engineering/developer-guide.md).
5. Read the [Codebase Map](../engineering/codebase-map.md).
6. Read [Local Validation](../operations/local-validation.md).
7. Read `AGENTS.md` and check open PRs and CI.

## Current baseline

The product foundation is complete through M25, the parser-quality foundation is complete through PQ74, and merged vertical extraction is complete through Vertical 13 / PR #429.

The public fixture currently validates:

- 50 BBT entries and 63 NBT entries;
- 11 folders and one extracted message;
- two body payloads and zero attachments;
- 16 selected and 19 unknown properties;
- four 52-byte Table Context rows;
- two To and two Cc recipient roles;
- four recipient display names and four native email-address values;
- fail-closed complete recipient record assembly.

Draft PR #430 is active but unmerged. It projects complete recipient records from the same validated rows and heap in one invocation.

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

1. Confirm no implementation or CI run is already in progress for the same boundary.
2. Identify one evidence-backed vertical extraction gap.
3. Reuse validated components and keep scope narrow.
4. Fail closed rather than adding heuristic fallback logic.
5. Add focused regression tests.
6. Run the complete validation gate.
7. Inspect the public fixture artifact and record the exact delta.
8. Update current-state and point-in-time documentation.
9. Open a PR and merge only after the exact head is green.

## Next boundary

Complete same-run recipient projection, then publish complete recipient records through production Table Context reporting. After that, choose the next highest-value missing email component from fixture evidence rather than following a precommitted infrastructure queue.
