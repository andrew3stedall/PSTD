# Issue Quality Policy

## Purpose

Issues should be detailed enough that a developer can begin work without needing an additional briefing.

## Required sections

Each implementation issue should contain:

```md
## Goal

## Background

## In scope

## Out of scope

## User flow or system flow

## Acceptance criteria

## Technical notes

## Data considerations

## UX considerations

## Security and infrastructure considerations

## Dependencies

## Test expectations

## Documentation required

## Open questions
```

## Acceptance criteria rules

Acceptance criteria must be:

- Observable.
- Testable.
- Specific.
- Scoped to the issue.
- Independent where possible.

Avoid vague criteria such as:

- "Works well."
- "Is scalable."
- "Looks good."
- "Handles all edge cases."

Prefer concrete criteria such as:

- "Given a valid PST file, the extractor writes one `.eml` file per message."
- "Given a corrupt PST file, the command exits non-zero and records the failure reason."
- "Given duplicate message IDs, the output file names remain deterministic and collision-safe."

## Dependency rules

Every issue should state:

- What must happen first.
- What it blocks.
- Whether it can be handled in parallel.
- Which files or components are likely to be touched.

## Risk classification

Use one of:

- `risk:low` for isolated docs, tests, or small implementation work.
- `risk:medium` for work that affects user behaviour or multiple components.
- `risk:high` for work that affects data correctness, security, deployment, migrations, or large-scale processing.

## Ready definition

An issue is ready for implementation only when:

- The goal is clear.
- Scope is bounded.
- Acceptance criteria are testable.
- Dependencies are known.
- Risks are named.
- Required docs are listed.
