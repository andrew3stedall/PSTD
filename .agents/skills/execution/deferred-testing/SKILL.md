---
name: deferred-testing
description: Use only when PSTD changes cannot be validated through a local checkout, a temporary GitHub Actions checkout-and-patch workflow, or existing repository CI.
---

# Deferred Testing

## Purpose

Make genuinely unrun tests explicit rather than pretending validation happened. Do not invoke this skill merely because work is being performed from ChatGPT mobile or through the GitHub connector.

## Preferred validation path

Before deferring tests:

1. Use existing pull-request CI and approved fixture workflows where they cover the change.
2. For large-file incremental edits without a usable local checkout, use the temporary same-repository GitHub Actions checkout-and-patch method in root `AGENTS.md`.
3. Run formatting and the most relevant focused tests before the runner commits the implementation.
4. Run the full required validation gate on the exact cleaned PR head before merge.

A whole-file connector API, truncated fetch result, or lack of a laptop is not sufficient reason to defer validation when the Actions checkout method is available.

## Rules

- State exactly which tests were not run.
- State the concrete technical or permission blocker.
- List the commands or workflows that must run later.
- Identify the highest-risk areas.
- Prefer adding tests when expected behaviour is clear.
- Do not claim a milestone is fully verified until local, temporary-runner, or CI validation has happened.
- Do not merge while required validation remains deferred.

## PR wording

Use wording specific to the actual blocker. Avoid the obsolete blanket statement that phone or connector workflows cannot run tests.

Example:

`Focused tests passed on the temporary checked-out Actions runner. The following required validation remains unavailable because <specific blocker>: <commands or workflows>. Merge is blocked until it passes.`

## Output

Return tests completed, tests deferred, validation mechanism used, blocker, risk notes, and required follow-up actions.