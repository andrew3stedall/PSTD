---
name: github-planning-loop
description: Use when operating PSTD from ChatGPT on mobile through the GitHub connector. This skill defines a phone-first loop for planning, implementation, validation, documentation, and PR delivery without requiring a local checkout.
---

# GitHub Planning Loop Skill

## Purpose

Use this skill when work is being coordinated from ChatGPT on a phone rather than from a local Codex app.

## Loop

1. User provides a PRD, idea, issue, implementation request, or feedback in ChatGPT.
2. Assistant reads relevant repository files, pull requests, branches, commits, and CI through the GitHub connector.
3. Assistant continues existing work when present or creates one dedicated branch and draft PR.
4. Assistant makes small safe changes directly through the connector.
5. For large existing files, assistant uses the temporary same-repository GitHub Actions checkout-and-patch method defined in root `AGENTS.md`.
6. Assistant runs focused validation on the checked-out repository and then the repository's required exact-head CI and fixture workflows.
7. Assistant removes temporary patch scaffolding, inspects the final PR diff, and updates documentation.
8. User reviews from GitHub mobile or ChatGPT.
9. Assistant refines, resolves failures, and merges only when the required checks and review conditions pass.

## Allowed work

- Planning reports.
- Documentation and issue changes.
- Branch and pull-request creation.
- Small direct repository changes supported by the connector.
- Incremental large-file code changes through a temporary checked-out Actions runner.
- Focused test execution and formatting on the temporary runner.
- CI and fixture result inspection, failure diagnosis, and safe retry.
- Cleanup of temporary scripts and workflows before merge.

## Large-file working rule

A truncated connector response or whole-file contents API is not by itself a blocker.

When no usable local checkout is available:

- create a dedicated same-repository branch and draft PR;
- add a temporary exact-match patch script and narrowly scoped workflow;
- guard the workflow to the named branch and use minimum permissions;
- check out the full repository on the runner;
- assert each expected source block occurs exactly once;
- apply the incremental patch;
- run formatting and focused tests;
- commit and push only intended non-workflow files from the runner;
- change workflow files through the connector when necessary;
- remove all temporary patch scaffolding;
- inspect the cleaned final diff and exact-head checks.

Never run a write-capable patch workflow for a fork PR or untrusted ref. Never expose secrets to the patch job.

## Boundaries

This loop does not provide arbitrary external infrastructure, secret or settings changes, unrestricted deployment access, or work outside repository permissions. Scheduled execution is available only when separately configured by an approved automation mechanism.

## Output

Return repository state, branch and PR, files changed, validation performed, exact observable result, remaining blocker, cleanup status, and next highest-value step.