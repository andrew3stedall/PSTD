# Milestone PR Template

## Purpose

<What milestone or epic this PR delivers.>

## Scope

<What is included.>

## Out of scope

<What is explicitly not included.>

## Issues covered

- [ ] #<issue-number>: <issue title>
- [ ] #<issue-number>: <issue title>

## Files changed

- `<path>`: <why it changed>

## Behaviour changed

<Describe user-visible, CLI-visible, output-contract, or developer-visible changes.>

## Data impact

<Describe output schema, metadata, attachment, body, manifest, summary, or error-log impact.>

## Tests run

<Commands actually run, or `Not run`.>

## Tests deferred

Local tests not run from the phone/GitHub connector workflow. Tests should be run later from the Codex laptop setup or CI before release.

Suggested commands:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

## Known risks

- <risk>

## Follow-up

- <follow-up item>
