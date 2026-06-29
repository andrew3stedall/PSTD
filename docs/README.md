# PSTD Documentation

This documentation tree separates planning, engineering, data, operations, UX, and user-facing information.

## PSTD v1 planning package

- [PSTD v1 MVP PRD](product/pstd-v1-mvp-prd.md)
- [PSTD v1 Roadmap](product/pstd-v1-roadmap.md)
- [Milestone M1: Extraction Foundation and Archive Contract](milestones/pstd-v1-m1-extraction-foundation.md)
- [Epic E1: Local Extraction Foundation and Archive Contract](epics/pstd-v1-e1-local-extraction-foundation.md)
- [M1 Ordered Issue Plan](issues/pstd-v1-m1-ordered-issue-plan.md)
- [M1 Dependency Map](architecture/pstd-v1-m1-dependency-map.md)
- [Milestone M2: PST Binary Foundation](milestones/pstd-v1-m2-pst-binary-foundation.md)
- [Epic E2: PST Binary Reader and Structure Foundation](epics/pstd-v1-e2-pst-binary-foundation.md)
- [M2 Ordered Issue Plan](issues/pstd-v1-m2-ordered-issue-plan.md)
- [M2 Dependency Map](architecture/pstd-v1-m2-dependency-map.md)
- [M2 PST Binary Implementation Plan](engineering/pstd-v1-m2-pst-binary-implementation-plan.md)
- [M2 Deferred Testing Plan](operations/pstd-v1-m2-deferred-testing-plan.md)
- [Milestone M3: Folder and Metadata Extraction](milestones/pstd-v1-m3-folder-metadata-extraction.md)
- [Epic E3: Logical PST Structures and Metadata Extraction](epics/pstd-v1-e3-folder-metadata-extraction.md)
- [M3 Ordered Issue Plan](issues/pstd-v1-m3-ordered-issue-plan.md)
- [M3 Dependency Map](architecture/pstd-v1-m3-dependency-map.md)
- [M3 Folder Metadata Implementation Plan](engineering/pstd-v1-m3-folder-metadata-implementation-plan.md)
- [M3 Deferred Testing Plan](operations/pstd-v1-m3-deferred-testing-plan.md)
- [PSTD v1 Output Contract Summary](data/pstd-v1-output-contract-summary.md)
- [PSTD v1 CLI and Rust Implementation Plan](engineering/pstd-v1-cli-rust-implementation-plan.md)
- [PSTD v1 Deferred Testing Plan](operations/pstd-v1-deferred-testing-plan.md)
- [M1 Milestone Execution Checklist](milestones/pstd-v1-m1-execution-checklist.md)

## Product

- [Planning council overview](product/council-overview.md)
- [Mobile operating model](product/phone-first-operating-model.md)
- [PRD intake template](product/prd-intake-template.md)
- [Planning labels](product/planning-labels.md)

## Engineering

- [Repository setup](engineering/repository-setup.md)

## Architecture decisions

- [ADR-0001: Codex planning council](decisions/ADR-0001-codex-planning-council.md)
- [ADR-0002: Mobile planning workflow](decisions/ADR-0002-phone-first-planning.md)
- [ADR-0003: Milestone execution mode](decisions/ADR-0003-milestone-execution-mode.md)

## Changelog

- [Unreleased](changelog/unreleased.md)

## Repo skills

Repo-scoped skill instructions are kept under `.agents/skills/` for future Codex runtimes and reusable planning/execution guidance.

## Documentation rule

Every meaningful planning or implementation change should update at least one relevant document. If no documentation update is required, the PR must explain why.
