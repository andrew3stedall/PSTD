---
name: deferred-testing
description: Use when PSTD code or docs are changed from ChatGPT mobile and tests cannot be run locally until the user sets up Codex on a laptop.
---

# Deferred Testing

## Purpose

Make unrun tests explicit rather than pretending validation happened.

## Rules

- State that local tests were not run.
- State why they were not run.
- List the commands that should be run later.
- Identify the highest-risk areas.
- Prefer adding tests when the expected behaviour is clear, even if they are run later.
- Do not claim a milestone is fully verified until local or CI validation has happened.

## PR wording

Use this wording when relevant:

`Local tests not run from the phone/GitHub connector workflow. Tests should be run later from the Codex laptop setup or CI before release.`

## Output

Return test commands, validation gap, risk notes, and follow-up actions.
