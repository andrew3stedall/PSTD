---
name: deferred-testing
description: Use only when PSTD changes cannot currently be validated through a local checkout, temporary targeted runner, or existing repository CI. Deferred focused validation may accompany checkpoint commits but never merge readiness.
---

# Deferred Testing

## Purpose

Make genuinely unrun tests explicit without sacrificing durable implementation progress. Do not invoke this skill merely because work is being performed from ChatGPT mobile or through the GitHub connector.

## Validation phases

### During implementation

1. Prefer the smallest focused formatting/test command that covers the changed boundary.
2. If a usable local checkout or existing CI can run it cheaply, use that path.
3. Use a temporary same-repository Actions runner only when targeted validation or large-file editing actually requires it.
4. If focused validation is temporarily unavailable, commit a coherent implementation increment rather than leaving it only in transient context.
5. Record the exact unrun command and blocker in `docs/operations/active-implementation-checkpoint.md` and mark the commit unverified.

Do not build a broad portable toolchain merely to avoid making a checkpoint commit.

### Before merge

1. Remove temporary patch/workflow scaffolding.
2. Run the full required validation gate on the exact cleaned PR head.
3. Run relevant approved fixture workflows and inspect the required artifacts.
4. Resolve all deferred focused validation that remains material to the changed boundary.
5. Do not merge while required validation remains deferred.

A whole-file connector API, truncated fetch result, or lack of a laptop is not by itself a reason to abandon a coherent implementation increment.

## Rules

- State exactly which tests were not run.
- State the concrete technical or permission blocker.
- Record the unrun commands in the active checkpoint.
- Identify the highest-risk areas.
- Prefer adding tests when expected behaviour is clear.
- Do not claim a test or milestone is verified until its result was inspected.
- Do not use repeated full-gate runs as a substitute for focused iteration.
- Required merge validation may not remain deferred.

## PR wording

During implementation:

`Checkpoint commit <sha> is durable but not yet fully verified. Focused command <command> was not run because <specific blocker>. The active checkpoint records the remaining validation.`

Before merge, replace WIP wording with the exact validation results.

## Output

Return durable commit, tests completed, tests deferred, validation mechanism used, blocker, checkpoint update, risk notes, and required pre-merge follow-up.