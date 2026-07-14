# PSTD Documentation

_Last reviewed: 14 July 2026._

This documentation tree contains both current operating guidance and historical delivery evidence. Use the current-state pages below for present capability and next work. Milestone, PQ, vertical, issue-plan, and implementation-plan files record what was known at the time they were written.

## Start here

| Need | Authoritative page |
|---|---|
| Project intent, headline progress, and commands | [Root README](../README.md) |
| Current merged capability and active blocker | [Project Status](product/project-status.md) |
| Real-fixture evidence over time | [Public PST Progress Log](operations/public-pst-progress-log.md) |
| Current extraction roadmap | [PSTD Roadmap](product/pstd-v1-roadmap.md) |
| Documentation freshness and history policy | [Documentation Status](DOCUMENTATION_STATUS.md) |
| Architecture | [System Overview](architecture/system-overview.md) |
| Code navigation | [Codebase Map](engineering/codebase-map.md) |
| Developer workflow | [Developer Guide](engineering/developer-guide.md) |
| Validation commands | [Local Validation](operations/local-validation.md) |
| Structured output contract | [Output Contract Summary](data/pstd-v1-output-contract-summary.md) |
| Known gaps and deferred systems | [Unsupported and Deferred Areas](operations/v1-unsupported-deferred-areas.md) |

## Current extraction state

| Delivery phase | Current outcome |
|---|---|
| M1-M25 | Product foundation complete: CLI, Python wrapper, Docker, TAR/JSONL outputs, batch/resume, diagnostics, and operator handoff. |
| PQ1-PQ35 | Corrected traversal and discovered real folder, message, property, payload, and subnode structures. |
| PQ36 | Material body/property fidelity improvement and stricter structural admission. |
| PQ37-PQ57 | Validated the real Table Context heap and four bounded 52-byte rows. |
| PQ58-PQ74 | Validated column mapping, row transport, fixed-width value decoding, and production diagnostic publication. |
| Vertical 1-13 | Extracted and assembled recipient roles, display names, addresses, and address kinds into complete row-aligned records. |
| Draft PR #430 | Same-run complete recipient projection is under review and is not yet merged capability. |

The next merged behaviour should project complete recipient records from one validated execution and publish them through production Table Context reporting. The exact follow-up after that must be chosen from the resulting fixture evidence.

## Current guidance by audience

### Product and planning

- [Project Status](product/project-status.md)
- [PSTD v1 MVP PRD](product/pstd-v1-mvp-prd.md)
- [PSTD Roadmap](product/pstd-v1-roadmap.md)
- [Phone-first Operating Model](product/phone-first-operating-model.md)

### Engineering and architecture

- [System Overview](architecture/system-overview.md)
- [Codebase Map](engineering/codebase-map.md)
- [Developer Guide](engineering/developer-guide.md)
- [Table-led Extraction Note](architecture/table-led-extraction-note.md)
- [PST Parser Research](research/pst-parser-research.md)

### Operations and evidence

- [Public PST Progress Log](operations/public-pst-progress-log.md)
- [Local Validation](operations/local-validation.md)
- [Local and Docker Operator Handoff](operations/local-docker-operator-handoff.md)
- [Unsupported and Deferred Areas](operations/v1-unsupported-deferred-areas.md)
- [Vertical 13: Complete Recipient Records](operations/vertical-13-assemble-complete-recipient-records.md)

### Data contract

- [PSTD v1 Output Contract Summary](data/pstd-v1-output-contract-summary.md)

### Wiki

- [Wiki Home](wiki/Home.md)
- [Developer Onboarding](wiki/developer-onboarding.md)

### Change history

- [Unreleased Changelog](changelog/unreleased.md)

## Historical records

The following directories are retained as point-in-time evidence:

- `milestones/` — completed M1-M25 and PQ milestone reports;
- `issues/` — ordered issue plans written for earlier delivery phases;
- `engineering/` — implementation plans, alongside current engineering guides;
- `operations/` — fixture findings, PQ reports, vertical reports, and current operating guides;
- `epics/` — early epic definitions;
- `decisions/` — architecture and operating-model decisions.

A historical document can accurately describe an old blocker even when that blocker has since been resolved. Do not use those files alone to determine the current roadmap.

## Repository skills

Repo-scoped instructions live under `.agents/skills/`. Start with [the skills index](../.agents/skills/README.md). `AGENTS.md` and the current project-status documents override older skill wording when the delivery model has changed.
