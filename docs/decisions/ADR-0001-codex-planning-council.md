# ADR-0001: Use a planning-first Codex council

## Status

Accepted

## Context

PSTD needs structured product and technical planning before autonomous implementation. The repo is currently minimal and the validation commands are not yet known.

The desired agent is broad: business analysis, product ownership, UX, development feasibility, data, systems, and documentation. That breadth is useful for planning, but risky if it directly changes code before the repo has stable guardrails.

The current user workflow is phone-first: ChatGPT mobile plus the GitHub connector. There is no local Codex install available, and API-key GitHub Actions should not be used as the default planning path.

## Decision

Adopt a planning-first Codex council.

The council may:

- Turn draft PRDs into epics and issues.
- Add success criteria.
- Identify dependencies and paused decisions.
- Document data, UX, systems, and engineering scope.
- Maintain planning documentation.
- Operate through ChatGPT and the GitHub connector.

The council may not implement application code until a later ADR explicitly enables execution mode.

## Consequences

### Positive

- Reduces project drift.
- Produces developer-ready issues.
- Avoids premature implementation.
- Makes missing decisions visible.
- Creates documentation early.
- Works from a phone-first setup.

### Negative

- Slower initial delivery.
- Requires humans to review planning output.
- Does not yet perform coding tasks.
- Does not provide unattended scheduling.

## Future decision points

- When to enable single-task execution.
- Whether to allow scheduled planning.
- Whether to allow parallel subagents.
- What validation commands define a complete implementation.
- Whether to introduce API-backed automation later.
