# Active Implementation Checkpoint Template

Copy this file to `docs/operations/active-implementation-checkpoint.md` on a non-trivial active implementation branch. Keep the active checkpoint concise and update it whenever durable implementation state changes materially.

Delete or reset the active checkpoint when its implementation PR is completed so stale execution state cannot be mistaken for current work.

```markdown
# Active Implementation Checkpoint

Updated: <UTC timestamp or date>

## Active delivery

- Issue: #<number> — <title>
- Parent epic: #<number> or none
- Branch: `<branch>`
- PR: #<number> or not opened
- Base: `<base sha>`
- Latest durable implementation commit: `<sha>`
- Validation state: focused-green | focused-failing | unverified | merge-gate-green

## Scope

<2-5 lines describing exactly what this issue/slice is implementing.>

### Out of scope

- <explicit adjacent work not being implemented>

## Established conclusions — do not re-derive

- <specific conclusion and source/reference if needed>
- <specific conclusion>

Only revisit these when new repository/spec/fixture evidence contradicts them.

## Current source boundary

- `<path>` — `<function/type/symbol>` — <why it is being changed>
- `<path>` — `<function/type/symbol>` — <why it is being changed>

## Durable progress

- `<sha>` — <what this commit established>

## Focused validation

- PASS: `<command>` — <result>
- FAIL: `<command>` — <concise failure>
- NOT RUN: `<command>` — <specific blocker>

Do not paste long logs. Record only the failure that changes the next action.

## Blockers / unresolved evidence

- <concrete blocker, or `None`>

## Next exact change

<One concrete editing step that a fresh agent can begin immediately, naming the path/symbol and intended behavior.>

## Merge-only work remaining

- full repository validation gate
- relevant approved fixture workflow/artifact inspection
- final current-state/changelog updates if behavior changed
- final diff/review-thread inspection
- remove temporary runner/patch scaffolding
```

## Size rule

The active checkpoint should normally stay below roughly 100 lines. It is a handoff, not a research report. Link to durable issues, commits, tests, or historical documents instead of copying them.

## Context rule

A valid checkpoint replaces repository rediscovery on continuation. An agent should not bulk-read the parent epic, project-history docs, PQ records, parity-gap corpus, or unrelated CI when the checkpoint already identifies the active scope and next exact change.
