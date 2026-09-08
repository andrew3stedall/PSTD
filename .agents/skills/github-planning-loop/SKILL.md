---
name: github-planning-loop
description: Use when operating PSTD from ChatGPT/mobile through the GitHub connector. Resume existing work from a branch-local checkpoint, persist bounded increments early, and defer full validation until merge readiness.
---

# GitHub Planning Loop Skill

## Purpose

Operate PSTD safely through ChatGPT and the GitHub connector without paying the full repository-discovery cost on every continuation.

## Continuation loop

When an implementation branch/PR already exists:

1. Read root `AGENTS.md`, `docs/operations/active-implementation-checkpoint.md`, and the active issue.
2. Inspect the current PR diff/latest implementation commit and only the source files named by the checkpoint.
3. Check CI only for directly relevant failures or when evaluating merge readiness.
4. Apply the checkpoint's `Next exact change`.
5. Run focused validation when readily available.
6. Commit the coherent increment before broad research, toolchain setup, full validation, or another delegation.
7. Update the checkpoint with the durable commit, test state, conclusions, blockers, and next exact change.
8. Repeat until implementation scope is complete.
9. Only then run full exact-head CI/fixtures, update final docs/PR narrative, clean temporary scaffolding, and merge when green.

Do not reread full current-state documents, the full epic, or historical parity/PQ/milestone documents on a valid continuation unless the checkpoint identifies a changed dependency or the active issue cites a specific section.

## Fresh-start loop

When no resumable implementation exists:

1. Read the minimum current-state sources required by root `AGENTS.md`.
2. Check open PRs/branches to avoid conflicting work.
3. Select exactly one active issue or coherent vertical slice.
4. Create one dedicated branch and draft PR.
5. Create `docs/operations/active-implementation-checkpoint.md` immediately.
6. Begin the first bounded implementation increment and persist it early.
7. Continue using the continuation loop.

## Allowed work

- Planning reports when genuinely required.
- Documentation and issue changes.
- Branch and pull-request creation.
- Small direct repository changes supported by the connector.
- Incremental large-file code changes through a temporary checked-out Actions runner when needed.
- Focused test execution and formatting.
- CI and fixture result inspection, failure diagnosis, and safe retry.
- Cleanup of temporary scripts and workflows before merge.

## Large-file working rule

A truncated connector response or whole-file contents API is not by itself a blocker. Prefer targeted reads and direct edits first. When no usable local checkout is available and an incremental large-file change genuinely requires it:

- use the existing same-repository branch/PR;
- add a temporary exact-match patch script and narrowly scoped workflow;
- guard the workflow to the named branch and use minimum permissions;
- check out the full repository on the runner;
- assert each expected source block occurs exactly once;
- apply the bounded patch;
- run focused formatting/tests;
- commit and push only intended non-workflow files;
- update the active checkpoint;
- remove temporary scaffolding as soon as it is no longer needed and always before merge.

Never build temporary CI/toolchain infrastructure merely to preserve work that can already be committed safely through the connector. Never run a write-capable patch workflow for a fork PR or untrusted ref. Never expose secrets to the patch job.

## Context discipline

- One active issue at a time.
- Parent epics provide ordering/dependencies only.
- Prefer exact symbols, targeted source ranges, diffs, and failing log excerpts.
- Delegate only separable questions with durable output.
- Persist delegated conclusions before asking another agent to investigate adjacent ground.
- Never repeat repository-wide discovery just because the conversation turn changed.

## Boundaries

This loop does not provide arbitrary external infrastructure, secret/settings changes, unrestricted deployment access, or work outside repository permissions. Required exact-head validation still gates merge.

## Output

Return active issue, branch/PR, latest durable implementation commit, checkpoint state, files changed, focused validation, exact observable result, remaining blocker, cleanup status, and next exact change.