# PQ2 Root Decode Candidate Selection

## Goal

Improve post-v1 PST root decoding by selecting safe root candidates before traversal.

## Context

PQ1 made impossible root offsets visible. PQ2 uses that evidence to classify whether a later Unicode root candidate can replace impossible legacy offsets, or whether the fixture remains blocked before tree traversal.

## Scope

- Add root candidate diagnostics.
- Prefer safe later Unicode root BREF offsets when available.
- Keep legacy root offsets as a diagnostic fallback candidate.
- Select traversal roots only when both BBT and NBT root pages are in bounds.
- Add regression tests for selected and blocked candidate states.
- Update parser-quality documentation.

## Out of scope

- Full BBT/NBT page compatibility expansion.
- Message, body, recipient, or attachment extraction changes.
- Snowflake ingestion implementation.

## Acceptance criteria

- Inspect JSON shows selected source and candidate diagnostics.
- Traversal roots are populated only from a safe selected candidate pair.
- Tests cover later Unicode candidate selection and no usable candidate states.
- Docs identify PQ3 as the next quality step.
