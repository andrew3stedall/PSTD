# PSTD Wiki Home

_Last reviewed: 14 July 2026._

This folder provides a compact repo-hosted navigation layer. For current truth, prefer the root README, project status, and public fixture progress log over historical milestone or PQ reports.

## Main pages

| Need | Page |
|---|---|
| Project intent and headline progress | [Root README](../../README.md) |
| Current merged capability and blocker | [Project Status](../product/project-status.md) |
| Real-fixture evidence | [Public PST Progress Log](../operations/public-pst-progress-log.md) |
| Active extraction roadmap | [Roadmap](../product/pstd-v1-roadmap.md) |
| Developer onboarding | [Developer Onboarding](developer-onboarding.md) |
| Code structure | [Codebase Map](../engineering/codebase-map.md) |
| System architecture | [System Overview](../architecture/system-overview.md) |
| Output contract | [Output Contract Summary](../data/pstd-v1-output-contract-summary.md) |
| Validation | [Local Validation](../operations/local-validation.md) |
| Documentation history policy | [Documentation Status](../DOCUMENTATION_STATUS.md) |

## Current state

| Delivery lane | State |
|---|---|
| M1-M25 | Complete product foundation |
| PQ1-PQ74 | Complete validated parser/Table Context foundation |
| Vertical 1-13 | Complete on `main`; recipient role, name, address, and address-kind assembly validated |
| Vertical 14 / PR #430 | Draft and unmerged; same-run complete-recipient projection |
| Complete production recipient publication | Next boundary after Vertical 14 |
| General PST-to-EML conversion | Not yet complete |

## Working rule

Read `AGENTS.md`, inspect active PR/CI state, and use the current-state pages before changing the code. Historical files remain valuable evidence but may describe blockers that have since been resolved.
