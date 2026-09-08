# PSTD Documentation

_Last reviewed: 8 September 2026._

This documentation tree contains current operating guidance, temporary branch-local execution state, and historical delivery evidence. Do not load all three classes for routine continuation work.

## Start here

| Need | Authoritative page |
|---|---|
| Resume an active implementation branch | `AGENTS.md` + branch-local `operations/active-implementation-checkpoint.md` + active GitHub issue |
| Create an active implementation checkpoint | [Checkpoint template](operations/implementation-checkpoint-template.md) |
| Install, inspect, extract, generate EML, and call PSTD from Python | [Quickstart](quickstart.md) |
| Project intent, headline progress, and commands | [Root README](../README.md) |
| Current merged capability and active blocker | [Project Status](product/project-status.md) |
| Real-fixture evidence over time | [Public PST Progress Log](operations/public-pst-progress-log.md) |
| Approved upstream fixtures, provenance, hashes, and development order | [Upstream PST Fixture Corpus](operations/upstream-pst-fixture-corpus.md) |
| Controlled ANSI fixture generation and admission | [ANSI PST Fixture Generation](fixtures/ansi-pst-generation.md) |
| Current extraction roadmap | [PSTD Roadmap](product/pstd-v1-roadmap.md) |
| Documentation freshness, history, and context-loading policy | [Documentation Status](DOCUMENTATION_STATUS.md) |
| readpst compatibility gaps and durable parity evidence | [readpst parity gap register](readpst-gaps/README.md) |
| Architecture | [System Overview](architecture/system-overview.md) |
| Code navigation | [Codebase Map](engineering/codebase-map.md) |
| Developer workflow | [Developer Guide](engineering/developer-guide.md) |
| Validation commands | [Local Validation](operations/local-validation.md) |
| Structured output contract | [Output Contract Summary](data/pstd-v1-output-contract-summary.md) |
| Known gaps and deferred systems | [Unsupported and Deferred Areas](operations/v1-unsupported-deferred-areas.md) |

## Context-loading rule

### Continuing existing implementation

Use the continuation fast path in `AGENTS.md`. Read the active checkpoint and issue, inspect the current diff/latest durable commit and only the named source boundary, then continue from `Next exact change`.

Do **not** bulk-read the root README, project status, public fixture history, parent epic, PQ records, vertical history, or `readpst-gaps/` corpus when a valid checkpoint already captures the active scope.

### Starting new implementation

Read the minimum current-state sources required by `AGENTS.md`, inspect active PRs/branches to avoid conflict, select one issue/slice, create the branch/PR and checkpoint, then switch to the continuation fast path.

## Documentation classes

### Current truth

Current product, capability, roadmap, architecture, validation, developer workflow, and output-contract pages describe the maintained repository state. See [Documentation Status](DOCUMENTATION_STATUS.md) for exact authority.

### Temporary execution state

`operations/active-implementation-checkpoint.md` exists only on an active non-trivial implementation branch. It is a concise handoff, not permanent product truth. Delete/reset it when its PR is completed.

### Historical evidence

Milestone, PQ, vertical, issue-plan, parity-plan, and implementation-plan documents record what was known at the time. Fetch them only for a specific evidence question. Do not use them alone to determine the current roadmap and do not ingest the historical corpus by default on continuation.

## Current guidance by audience

### Product and planning

- [Project Status](product/project-status.md)
- [PSTD v1 MVP PRD](product/pstd-v1-mvp-prd.md)
- [PSTD Roadmap](product/pstd-v1-roadmap.md)
- [Phone-first Operating Model](product/phone-first-operating-model.md)

### Engineering and architecture

- [Quickstart](quickstart.md)
- [System Overview](architecture/system-overview.md)
- [Codebase Map](engineering/codebase-map.md)
- [Developer Guide](engineering/developer-guide.md)
- [Checkpoint template](operations/implementation-checkpoint-template.md)
- [Table-led Extraction Note](architecture/table-led-extraction-note.md)
- [PST Parser Research](research/pst-parser-research.md)

### Operations and evidence

- [Public PST Progress Log](operations/public-pst-progress-log.md)
- [Upstream PST Fixture Corpus](operations/upstream-pst-fixture-corpus.md)
- [ANSI PST Fixture Generation](fixtures/ansi-pst-generation.md)
- [Local Validation](operations/local-validation.md)
- [Local and Docker Operator Handoff](operations/local-docker-operator-handoff.md)
- [Unsupported and Deferred Areas](operations/v1-unsupported-deferred-areas.md)
- [readpst Parity Gap Register](readpst-gaps/README.md)

### Data contract

- [PSTD v1 Output Contract Summary](data/pstd-v1-output-contract-summary.md)

### Wiki

- [Wiki Home](wiki/Home.md)
- [Developer Onboarding](wiki/developer-onboarding.md)

### Change history

- [Unreleased Changelog](changelog/unreleased.md)

## Repository skills

Repo-scoped instructions live under `.agents/skills/`. Start with [the skills index](../.agents/skills/README.md). Root `AGENTS.md`, the active checkpoint, and the active issue define continuation behavior. Current-state documents override older skill wording for durable capability truth.
