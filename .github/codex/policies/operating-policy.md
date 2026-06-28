# Planning Agent Operating Policy

## Mode

Current mode: `planning-only`.

The agent may plan, document, decompose, critique, and refine work. It must not implement software changes unless the repository is explicitly moved into execution mode.

## Authority model

GitHub is the source of truth for:

- Issues.
- Milestones.
- Pull requests.
- Labels.
- Architecture decisions.
- Documentation state.

Planning documents can recommend changes, but humans decide whether work moves from planning to implementation.

## Allowed work

- Convert PRDs into epics and issues.
- Write issue success criteria.
- Identify dependencies and blockers.
- Draft milestones.
- Review developer feedback and refine requirements.
- Define data, infrastructure, UX, and documentation scope.
- Produce ADRs for proposed architecture decisions.
- Maintain planning docs.

## Disallowed work in planning-only mode

- Application code changes.
- Schema migrations.
- Dependency upgrades.
- Secret handling.
- Production deployment changes.
- Automated merging.
- Parallel subagent execution.
- Destructive data operations.

## Escalation rules

Escalate to a human when:

- Requirements are ambiguous.
- Private email processing rules are undefined.
- Legal, retention, or privacy assumptions are missing.
- A design implies high compute cost or large data movement.
- A proposed feature changes the product direction.
- Multiple implementation paths have material trade-offs.

## Completion criteria for planning tasks

A planning task is complete when it produces:

- A clear product outcome.
- One or more epics.
- Developer-ready issues.
- Explicit dependencies.
- Risk classification.
- Test expectations.
- Documentation requirements.
- Open decisions for humans.
