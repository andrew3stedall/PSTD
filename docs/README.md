# PSTD Documentation

This tree is organised by audience.

## Start here

| Need | Page |
|---|---|
| Current status | [Project Status](product/project-status.md) |
| Public PST progress log | [Public PST Progress Log](operations/public-pst-progress-log.md) |
| PQ10 heap BTH property traversal | [PQ10 Heap BTH Property Traversal](operations/pq10-heap-bth-property-traversal.md) |
| PQ9 property tag-shape status | [PQ9 Property Tag-Shape Status Reporting](operations/pq9-tag-shape-status.md) |
| PQ8 property-context interpretation | [PQ8 Property-Context Layout and Tag Interpretation](operations/pq8-property-context-interpretation.md) |
| PQ7 selected property dictionary | [PQ7 Selected Property Dictionary Expansion](operations/pq7-selected-property-dictionary.md) |
| PQ6 property and body coverage | [PQ6 Property and Body Coverage](operations/pq6-property-body-coverage.md) |
| PQ5 message table discovery | [PQ5 Message Table Discovery](operations/pq5-message-table-discovery.md) |
| PQ4 folder hierarchy discovery | [PQ4 Folder Hierarchy Discovery](operations/pq4-folder-hierarchy-discovery.md) |
| PQ3 index entry decoding | [PQ3 Index Entry Decoding](operations/pq3-index-entry-decoding.md) |
| PQ2 root candidate selection | [PQ2 Root Decode Candidate Selection](operations/pq2-root-decode-candidate-selection.md) |
| PQ1 root diagnostics | [PQ1 Root Decode Diagnostics](operations/pq1-root-decode-diagnostics.md) |
| Release-candidate checklist | [v1 Release-Candidate Checklist](operations/v1-release-candidate-checklist.md) |
| Local/Docker operator handoff | [Local and Docker Operator Handoff](operations/local-docker-operator-handoff.md) |
| Unsupported/deferred areas | [Unsupported and Deferred Areas](operations/v1-unsupported-deferred-areas.md) |
| Developer onboarding | [Developer Guide](engineering/developer-guide.md) |
| Code navigation | [Codebase Map](engineering/codebase-map.md) |
| Architecture overview | [System Overview](architecture/system-overview.md) |
| Validation commands | [Local Validation](operations/local-validation.md) |
| Wiki | [Wiki Home](wiki/Home.md) |
| Roadmap | [PSTD v1 Roadmap](product/pstd-v1-roadmap.md) |

## Product and planning

- [Project Status](product/project-status.md)
- [PSTD v1 MVP PRD](product/pstd-v1-mvp-prd.md)
- [PSTD v1 Roadmap](product/pstd-v1-roadmap.md)
- [Planning council overview](product/council-overview.md)
- [Mobile operating model](product/phone-first-operating-model.md)
- [PRD intake template](product/prd-intake-template.md)
- [Planning labels](product/planning-labels.md)

## Milestones

| Milestone | Main doc | Epic | Issue plan | Dependency map |
|---|---|---|---|---|
| M1 | [Extraction Foundation](milestones/pstd-v1-m1-extraction-foundation.md) | [E1](epics/pstd-v1-e1-local-extraction-foundation.md) | [M1 issues](issues/pstd-v1-m1-ordered-issue-plan.md) | [M1 map](architecture/pstd-v1-m1-dependency-map.md) |
| M2 | [PST Binary Foundation](milestones/pstd-v1-m2-pst-binary-foundation.md) | [E2](epics/pstd-v1-e2-pst-binary-foundation.md) | [M2 issues](issues/pstd-v1-m2-ordered-issue-plan.md) | [M2 map](architecture/pstd-v1-m2-dependency-map.md) |
| M3 | [Folder and Metadata Extraction](milestones/pstd-v1-m3-folder-metadata-extraction.md) | [E3](epics/pstd-v1-e3-folder-metadata-extraction.md) | [M3 issues](issues/pstd-v1-m3-ordered-issue-plan.md) | [M3 map](architecture/pstd-v1-m3-dependency-map.md) |
| M4 | [Recipients and Threading](milestones/pstd-v1-m4-recipients-threading.md) | TBD | [M4 issues](issues/pstd-v1-m4-ordered-issue-plan.md) | TBD |
| M5 | [Bodies and Attachments](milestones/pstd-v1-m5-bodies-attachments.md) | TBD | [M5 issues](issues/pstd-v1-m5-bodies-attachments.md) | TBD |
| M6 | [Batch and Resume](milestones/pstd-v1-m6-batch-resume.md) | TBD | [M6 issues](issues/pstd-v1-m6-ordered-issue-plan.md) | TBD |
| M7 | [Parser Depth Hardening](milestones/pstd-v1-m7-parser-depth-hardening.md) | TBD | [M7 issues](issues/pstd-v1-m7-ordered-issue-plan.md) | TBD |
| M8 | [Traversal Expansion](milestones/pstd-v1-m8-traversal-expansion.md) | TBD | [M8 issues](issues/pstd-v1-m8-traversal-expansion.md) | TBD |
| M9 | [Payload and Subnode Traversal](milestones/pstd-v1-m9-payload-subnode-traversal.md) | TBD | [M9 issues](issues/pstd-v1-m9-payload-subnode-traversal.md) | TBD |
| M10 | [Payload Wiring](milestones/pstd-v1-m10-payload-wiring.md) | TBD | [M10 issues](issues/pstd-v1-m10-ordered-issue-plan.md) | TBD |
| M11 | [Extraction Path Integration](milestones/pstd-v1-m11-extraction-integration.md) | TBD | [M11 issues](issues/pstd-v1-m11-extraction-integration.md) | TBD |
| M12 | [Attachment Table and Subnode Integration](milestones/pstd-v1-m12-attachment-subnode-integration.md) | TBD | [M12 issues](issues/pstd-v1-m12-attachment-subnode-integration.md) | TBD |
| M13 | [Payload Fixture Expansion and Parser Compatibility](milestones/pstd-v1-m13-fixtures-compatibility.md) | TBD | [M13 issues](issues/pstd-v1-m13-ordered-issue-plan.md) | TBD |
| M14 | [Recursive Subnode Layout Exploration](milestones/pstd-v1-m14-recursive-subnode-layouts.md) | TBD | [M14 issues](issues/pstd-v1-m14-recursive-subnode-layouts.md) | TBD |
| M15 | [Observed Layout Compatibility and Public Fixture Triage](milestones/pstd-v1-m15-observed-layout-triage.md) | TBD | [M15 issues](issues/pstd-v1-m15-observed-layout-triage.md) | TBD |
| M16 | [Fixture-Backed Decoder Expansion](milestones/pstd-v1-m16-fixture-backed-decoders.md) | TBD | [M16 issues](issues/pstd-v1-m16-fixture-backed-decoders.md) | TBD |
| M17 | [Compatibility Triage Reporting and Decoder Backlog](milestones/pstd-v1-m17-triage-reporting-backlog.md) | TBD | [M17 issues](issues/pstd-v1-m17-triage-reporting-backlog.md) | TBD |
| M18 | [Decoder Backlog Review Workflow](milestones/pstd-v1-m18-backlog-review-workflow.md) | TBD | [M18 issues](issues/pstd-v1-m18-backlog-review-workflow.md) | TBD |
| M19 | [Focused Candidate Selection](milestones/pstd-v1-m19-candidate-selection.md) | TBD | [M19 issues](issues/pstd-v1-m19-candidate-selection-plan.md) | TBD |
| M20 | [Focused Candidate Implementation](milestones/pstd-v1-m20-focused-candidate-implementation.md) | TBD | [M20 issues](issues/pstd-v1-m20-focused-candidate-implementation-plan.md) | TBD |
| M21 | [Focused Decoder Evidence Expansion](milestones/pstd-v1-m21-focused-decoder-evidence-expansion.md) | TBD | [M21 issues](issues/pstd-v1-m21-focused-decoder-evidence-expansion-plan.md) | TBD |
| M22 | [Body and Header Fidelity Expansion](milestones/pstd-v1-m22-body-header-fidelity-expansion.md) | TBD | [M22 issues](issues/pstd-v1-m22-body-header-fidelity-expansion-plan.md) | TBD |
| M23 | [Attachment Payload Fidelity](milestones/pstd-v1-m23-attachment-payload-fidelity.md) | TBD | [M23 issues](issues/pstd-v1-m23-attachment-payload-fidelity-plan.md) | TBD |
| M24 | [Batch Scale, Performance, and Corruption Hardening](milestones/pstd-v1-m24-batch-hardening.md) | TBD | [M24 issues](issues/pstd-v1-m24-batch-hardening-plan.md) | TBD |
| M25 | [Release Candidate and Operator Handoff](milestones/pstd-v1-m25-release-candidate-operator-handoff.md) | TBD | [M25 issues](issues/pstd-v1-m25-release-candidate-plan.md) | TBD |
| PQ1 | [Root Decode Diagnostics](milestones/pq1-root-decode-diagnostics.md) | TBD | [PQ1 issues](issues/pq1-root-decode-diagnostics-issue-plan.md) | TBD |
| PQ2 | [Root Decode Candidate Selection](milestones/pq2-root-decode-candidate-selection.md) | TBD | [PQ2 issues](issues/pq2-root-decode-candidate-selection-issue-plan.md) | TBD |
| PQ3 | [Index Entry Decoding](milestones/pq3-index-entry-decoding.md) | TBD | [PQ3 issues](issues/pq3-index-entry-decoding-issue-plan.md) | TBD |
| PQ4 | [Folder Hierarchy Discovery](milestones/pq4-folder-hierarchy-discovery.md) | TBD | Issues #201-#207 | TBD |
| PQ5 | [Message Table Discovery](milestones/pq5-message-table-discovery.md) | TBD | Issues #248-#261 | TBD |
| PQ6 | [Property and Body Coverage](milestones/pq6-property-body-coverage.md) | TBD | Issues #263-#267 | TBD |
| PQ7 | [Selected Property Dictionary Expansion](milestones/pq7-selected-property-dictionary.md) | TBD | Issues #269-#273 | TBD |
| PQ8 | [Property-Context Interpretation Diagnostics](milestones/pq8-property-context-interpretation.md) | TBD | Issues #275-#279 | TBD |
| PQ9 | [Property Tag-Shape Status](milestones/pq9-tag-shape-status.md) | TBD | Issues #281-#285 | TBD |
| PQ10 | [Heap BTH Property Traversal](milestones/pq10-heap-bth-property-traversal.md) | TBD | Issues #287-#291 | TBD |

## v1 release-candidate handoff

| Need | Page |
|---|---|
| Final checklist | [v1 Release-Candidate Checklist](operations/v1-release-candidate-checklist.md) |
| Local/Docker runbook | [Local and Docker Operator Handoff](operations/local-docker-operator-handoff.md) |
| Unsupported/deferred boundary | [Unsupported and Deferred Areas](operations/v1-unsupported-deferred-areas.md) |
| Public PST progress log | [Public PST Progress Log](operations/public-pst-progress-log.md) |
| PQ10 heap BTH property traversal | [PQ10 Heap BTH Property Traversal](operations/pq10-heap-bth-property-traversal.md) |
| PQ9 property tag-shape status | [PQ9 Property Tag-Shape Status Reporting](operations/pq9-tag-shape-status.md) |
| PQ8 property-context interpretation | [PQ8 Property-Context Layout and Tag Interpretation](operations/pq8-property-context-interpretation.md) |
| PQ7 selected property dictionary | [PQ7 Selected Property Dictionary Expansion](operations/pq7-selected-property-dictionary.md) |
| PQ6 property and body coverage | [PQ6 Property and Body Coverage](operations/pq6-property-body-coverage.md) |
| PQ5 message table discovery | [PQ5 Message Table Discovery](operations/pq5-message-table-discovery.md) |
| PQ4 folder hierarchy discovery | [PQ4 Folder Hierarchy Discovery](operations/pq4-folder-hierarchy-discovery.md) |
| PQ3 index entry decoding | [PQ3 Index Entry Decoding](operations/pq3-index-entry-decoding.md) |
| PQ2 root candidate selection | [PQ2 Root Decode Candidate Selection](operations/pq2-root-decode-candidate-selection.md) |
| PQ1 root diagnostics | [PQ1 Root Decode Diagnostics](operations/pq1-root-decode-diagnostics.md) |

## Engineering

- [Developer Guide](engineering/developer-guide.md)
- [Codebase Map](engineering/codebase-map.md)
- [Repository setup](engineering/repository-setup.md)
- [PSTD v1 CLI and Rust Implementation Plan](engineering/pstd-v1-cli-rust-implementation-plan.md)
- [M2 PST Binary Implementation Plan](engineering/pstd-v1-m2-pst-binary-implementation-plan.md)
- [M3 Folder Metadata Implementation Plan](engineering/pstd-v1-m3-folder-metadata-implementation-plan.md)
- [M4 Recipients and Threading Implementation Plan](engineering/pstd-v1-m4-recipients-threading-implementation-plan.md)
- [M5 Bodies and Attachments Implementation Plan](engineering/pstd-v1-m5-bodies-attachments-implementation-plan.md)
- [M6 Batch and Resume Implementation Plan](engineering/pstd-v1-m6-batch-resume-implementation-plan.md)
- [M7 Parser Depth Hardening Implementation Plan](engineering/pstd-v1-m7-parser-depth-hardening-implementation-plan.md)
- [M8 Traversal Expansion Implementation Plan](engineering/pstd-v1-m8-traversal-expansion-implementation-plan.md)
- [M9 Payload and Subnode Traversal Implementation Plan](engineering/pstd-v1-m9-payload-subnode-traversal-implementation-plan.md)
- [M10 Payload Wiring Implementation Plan](engineering/pstd-v1-m10-payload-wiring-implementation-plan.md)
- [M11 Extraction Integration Implementation Plan](engineering/pstd-v1-m11-extraction-integration-implementation-plan.md)
- [M12 Attachment Subnode Integration Implementation Plan](engineering/pstd-v1-m12-attachment-subnode-integration-implementation-plan.md)
- [M13 Fixture Compatibility Implementation Plan](engineering/pstd-v1-m13-fixtures-compatibility-implementation-plan.md)
- [M14 Recursive Subnode Layout Implementation Plan](engineering/pstd-v1-m14-recursive-subnode-layouts-implementation-plan.md)
- [M15 Observed Layout Triage Implementation Plan](engineering/pstd-v1-m15-observed-layout-triage-implementation-plan.md)
- [M16 Fixture-Backed Decoder Implementation Plan](engineering/pstd-v1-m16-fixture-backed-decoders-implementation-plan.md)
- [M17 Triage Reporting Backlog Implementation Plan](engineering/pstd-v1-m17-triage-reporting-backlog-implementation-plan.md)
- [M18 Backlog Review Workflow Implementation Plan](engineering/pstd-v1-m18-backlog-review-workflow-implementation-plan.md)
- [M19 Candidate Selection Implementation Plan](engineering/pstd-v1-m19-candidate-selection-plan.md)
- [M20 Focused Candidate Implementation Plan](engineering/pstd-v1-m20-focused-candidate-implementation-plan.md)
- [M21 Focused Decoder Evidence Expansion Plan](engineering/pstd-v1-m21-focused-decoder-evidence-expansion-plan.md)
- [M22 Body and Header Fidelity Expansion Plan](engineering/pstd-v1-m22-body-header-fidelity-expansion-plan.md)
- [M23 Attachment Payload Fidelity Plan](engineering/pstd-v1-m23-attachment-payload-fidelity-plan.md)
- [M24 Batch Hardening Plan](engineering/pstd-v1-m24-batch-hardening-plan.md)
- [M25 Release Candidate Plan](engineering/pstd-v1-m25-release-candidate-plan.md)

## Architecture and data

- [System Overview](architecture/system-overview.md)
- [PSTD v1 Output Contract Summary](data/pstd-v1-output-contract-summary.md)

## Operations

- [Local Validation](operations/local-validation.md)
- [Public PST Progress Log](operations/public-pst-progress-log.md)
- [PQ10 Heap BTH Property Traversal](operations/pq10-heap-bth-property-traversal.md)
- [PQ9 Property Tag-Shape Status Reporting](operations/pq9-tag-shape-status.md)
- [PQ8 Property-Context Layout and Tag Interpretation](operations/pq8-property-context-interpretation.md)
- [PQ7 Selected Property Dictionary Expansion](operations/pq7-selected-property-dictionary.md)
- [PQ6 Property and Body Coverage](operations/pq6-property-body-coverage.md)
- [PQ5 Message Table Discovery](operations/pq5-message-table-discovery.md)
- [PQ4 Folder Hierarchy Discovery](operations/pq4-folder-hierarchy-discovery.md)
- [PQ3 Index Entry Decoding](operations/pq3-index-entry-decoding.md)
- [PQ2 Root Decode Candidate Selection](operations/pq2-root-decode-candidate-selection.md)
- [PQ1 Root Decode Diagnostics](operations/pq1-root-decode-diagnostics.md)
- [v1 Release-Candidate Checklist](operations/v1-release-candidate-checklist.md)
- [Local and Docker Operator Handoff](operations/local-docker-operator-handoff.md)
- [Unsupported and Deferred Areas](operations/v1-unsupported-deferred-areas.md)
- [Decoder Backlog Review Workflow](operations/decoder-backlog-review-workflow.md)
- [Candidate Selection Workflow](operations/candidate-selection-workflow.md)
- [Public and Sanitized Fixture Triage](operations/public-sanitized-fixture-triage.md)
- [PSTD v1 Deferred Testing Plan](operations/pstd-v1-deferred-testing-plan.md)
- [M2 Deferred Testing Plan](operations/pstd-v1-m2-deferred-testing-plan.md)
- [M3 Deferred Testing Plan](operations/pstd-v1-m3-deferred-testing-plan.md)

## Decisions

- [ADR-0001: Codex planning council](decisions/ADR-0001-codex-planning-council.md)
- [ADR-0002: Mobile planning workflow](decisions/ADR-0002-phone-first-planning.md)
- [ADR-0003: Milestone execution mode](decisions/ADR-0003-milestone-execution-mode.md)

## Wiki

- [Wiki Home](wiki/Home.md)
- [Developer Onboarding](wiki/developer-onboarding.md)

## Changelog

- [Unreleased](changelog/unreleased.md)

## Repo skills

Repo-scoped skill instructions live under `.agents/skills/`. Use `.agents/skills/README.md` as the skill index.
