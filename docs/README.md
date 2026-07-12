# PSTD Documentation

This tree is organised by audience. The M1-M25 foundation is complete; active work is the parser-quality sequence, currently validated through **PQ57** with **PQ58 next**.

## Start here

| Need | Page |
|---|---|
| Current implementation and next blocker | [Project Status](product/project-status.md) |
| Real-fixture conversion evidence | [Public PST Progress Log](operations/public-pst-progress-log.md) |
| Product scope and completed v1 foundation | [PSTD v1 Roadmap](product/pstd-v1-roadmap.md) |
| Local validation commands | [Local Validation](operations/local-validation.md) |
| Operator use | [Local and Docker Operator Handoff](operations/local-docker-operator-handoff.md) |
| Known unsupported/deferred areas | [Unsupported and Deferred Areas](operations/v1-unsupported-deferred-areas.md) |
| Developer onboarding | [Developer Guide](engineering/developer-guide.md) |
| Code navigation | [Codebase Map](engineering/codebase-map.md) |
| Architecture | [System Overview](architecture/system-overview.md) |
| Output schema | [PSTD v1 Output Contract Summary](data/pstd-v1-output-contract-summary.md) |

## Current parser-quality state

| Range | Outcome |
|---|---|
| PQ1-PQ5 | Corrected root/index traversal and real folder/message candidate discovery. |
| PQ6-PQ10 | Property/body coverage, selected dictionary expansion, tag-shape diagnostics, and Heap-on-Node/BTH foundations. |
| PQ11-PQ16 | Payload boundary, source selection, subnode decoding, interpretation, and classification. |
| PQ17-PQ23 | Table probes, counters, row-matrix measurement, and property candidates. |
| PQ24-PQ31 | Column, tag, descriptor, and source-propagation diagnostics. |
| PQ32-PQ35 | Invalid legacy descriptor assumption identified; Unicode SLBLOCK captured and recursively resolved. |
| PQ36 | `NDB_CRYPT_PERMUTE` decoding and structural payload admission; selected properties rose to 16 and body payloads were recovered. |
| PQ37-PQ48 | Resolved the real TC heap, row-index BTH, and end-to-end table probe reporting. |
| PQ49-PQ57 | Resolved subnode-backed row storage and validated row ordinals, 52-byte extents, bitmap boundaries, counts, and exact masks. |
| PQ58 next | Validate the TCINFO descriptor-to-bitmap index mapping without decoding row values. |

## Product and planning

- [Project Status](product/project-status.md)
- [PSTD v1 MVP PRD](product/pstd-v1-mvp-prd.md)
- [PSTD v1 Roadmap](product/pstd-v1-roadmap.md)
- [Planning council overview](product/council-overview.md)
- [Phone-first operating model](product/phone-first-operating-model.md)

## Milestone history

The detailed M1-M25 and PQ1+ milestone, issue-plan, and implementation documents remain under:

- [`milestones/`](milestones/)
- [`issues/`](issues/)
- [`engineering/`](engineering/)
- [`operations/`](operations/)

Historical milestone documents describe the decision and implementation at that point in time. For the current truth, use [Project Status](product/project-status.md) and the [Public PST Progress Log](operations/public-pst-progress-log.md).

## Architecture and data

- [System Overview](architecture/system-overview.md)
- [PSTD v1 Output Contract Summary](data/pstd-v1-output-contract-summary.md)

## Operations

- [Local Validation](operations/local-validation.md)
- [Public PST Progress Log](operations/public-pst-progress-log.md)
- [PQ57 bounded row bitmap mask evidence](operations/pq57-row-bitmap-mask-evidence.md)
- [PQ58 TCINFO column-to-bitmap index mapping](operations/pq58-column-bitmap-index-mapping.md)
- [v1 Release-Candidate Checklist](operations/v1-release-candidate-checklist.md)
- [Local and Docker Operator Handoff](operations/local-docker-operator-handoff.md)
- [Unsupported and Deferred Areas](operations/v1-unsupported-deferred-areas.md)
- [Decoder Backlog Review Workflow](operations/decoder-backlog-review-workflow.md)
- [Candidate Selection Workflow](operations/candidate-selection-workflow.md)
- [Public and Sanitized Fixture Triage](operations/public-sanitized-fixture-triage.md)

## Decisions

- [ADR-0001: Codex planning council](decisions/ADR-0001-codex-planning-council.md)
- [ADR-0002: Mobile planning workflow](decisions/ADR-0002-phone-first-planning.md)
- [ADR-0003: Milestone execution mode](decisions/ADR-0003-milestone-execution-mode.md)

## Wiki and changelog

- [Wiki Home](wiki/Home.md)
- [Developer Onboarding](wiki/developer-onboarding.md)
- [Unreleased changelog](changelog/unreleased.md)

## Repo skills

Repo-scoped skill instructions live under `.agents/skills/`. Use `.agents/skills/README.md` as the skill index.
