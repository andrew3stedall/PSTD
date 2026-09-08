---
name: continuation-checkpoint
description: Use when resuming or handing off non-trivial PSTD implementation. Maintains a compact branch-local checkpoint so agents continue from durable state instead of re-reading the repository.
---

# Continuation Checkpoint

## Purpose

Make long-running implementation resumable across agent turns with minimal context reload.

## Canonical file

`docs/operations/active-implementation-checkpoint.md`

Use `docs/operations/implementation-checkpoint-template.md` as the format.

## On resume

1. Read root `AGENTS.md`.
2. Read the active checkpoint and active issue.
3. Confirm the checkpoint branch/PR/latest durable commit still matches GitHub state.
4. Inspect only the current diff, source boundary named by the checkpoint, and any directly relevant failing check.
5. Begin with `Next exact change`.

If branch state has advanced beyond the checkpoint, update the checkpoint first. If the checkpoint accurately matches the branch, do not redo repository discovery.

## On progress

Update the checkpoint after any material change to:

- active issue/scope;
- established implementation conclusions;
- source boundary;
- latest durable implementation commit;
- focused test state;
- blocker;
- next exact change.

Keep it compact. Store code in commits, detailed requirements in issues, long evidence in durable docs/comments, and test logs in CI. The checkpoint should contain only the conclusions needed to resume.

## Before expensive work

Before broad research, full validation, environment/toolchain setup, or another specialist delegation:

1. make the current coherent implementation increment durable;
2. update the checkpoint;
3. then perform the expensive work.

This ordering prevents useful source changes and reasoning from being lost when an agent turn or token budget ends.

## On completion

After the implementation is merged or abandoned, delete/reset the branch-local active checkpoint. Never leave a stale active checkpoint on `main` as if it were current product truth.

## Output

Return active issue, latest durable commit, checkpoint freshness, validation state, blocker, and next exact change.