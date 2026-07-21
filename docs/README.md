# PSTD Documentation

_Last reviewed: 21 July 2026._

This documentation tree contains both current operating guidance and historical delivery evidence. Use the current-state pages below for present capability and next work. Milestone, PQ, vertical, issue-plan, and implementation-plan files record what was known at the time they were written.

## Start here

| Need | Authoritative page |
|---|---|
| Install, inspect, extract, generate EML, and call PSTD from Python | [Quickstart](quickstart.md) |
| Project intent, headline progress, and commands | [Root README](../README.md) |
| Current merged capability and active blocker | [Project Status](product/project-status.md) |
| Real-fixture evidence over time | [Public PST Progress Log](operations/public-pst-progress-log.md) |
| Approved upstream fixtures, provenance, hashes, and development order | [Upstream PST Fixture Corpus](operations/upstream-pst-fixture-corpus.md) |
| Controlled ANSI fixture generation and admission | [ANSI PST Fixture Generation](fixtures/ansi-pst-generation.md) |
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
| PQ1-PQ74 | Validated parser foundation through bounded traversal, Table Context row transport, and fixed-width value decoding. |
| Recipient verticals | Four structured To/Cc recipient records with names and usable addresses are emitted from the original public fixture. |
| Readable message verticals | One deterministic 956-byte EML is emitted with sender, recipients, subject, Date, Message-ID, plain text, and recovered HTML. |
| Upstream fixture corpus | Three pinned public PSTs cover attachments, multiple folders/messages, body forms, appointments, recurrence, contacts, distribution lists, and legacy Exchange addresses. Non-mail objects remain outside the active email-to-EML milestone. |
| Tika attachment fixture | Eight messages include seven top-level messages assigned by exact contents-table rows, one linked method-`5` child, nine directly owned recipients, ten body records, two exact attachment payloads, the unchanged 17,035-byte parent EML, and one exact 453-byte child EML. |
| Current milestone | Qualify additional immutable Unicode email fixture evidence for a second by-value attachment layout, multiple attachments, or exact inline/Content-ID behaviour. ANSI traversal and typed non-mail enrichment remain backlog-only. |

The current roadmap, compatibility matrix, and approved-fixture gap record define the active evidence-led sequence. Historical milestone and PQ documents remain useful for implementation context but do not define the next task.

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
- [Table-led Extraction Note](architecture/table-led-extraction-note.md)
- [PST Parser Research](research/pst-parser-research.md)

### Operations and evidence

- [Public PST Progress Log](operations/public-pst-progress-log.md)
- [Upstream PST Fixture Corpus](operations/upstream-pst-fixture-corpus.md)
- [Approved Attachment Fixture Gap](operations/vertical-40-approved-fixture-gap.md)
- [ANSI PST Fixture Generation](fixtures/ansi-pst-generation.md)
- [Local Validation](operations/local-validation.md)
- [Local and Docker Operator Handoff](operations/local-docker-operator-handoff.md)
- [Unsupported and Deferred Areas](operations/v1-unsupported-deferred-areas.md)
- [Vertical 28: Plain-text and HTML EML](operations/vertical-28-emit-plain-html-eml.md)
- [Vertical 34: Tika embedded message](operations/vertical-34-recover-tika-embedded-message.md)
- [Vertical 35: Tika child plain-text EML](operations/vertical-35-emit-tika-child-eml.md)
- [Vertical 36: Method-5 child EML payload](operations/vertical-36-materialise-method5-eml-payload.md)
- [Vertical 37: Tika message-folder ownership](operations/vertical-37-resolve-tika-message-folder-ownership.md)
- [Vertical 38: Reject unresolved binary body references](operations/vertical-38-reject-unresolved-binary-body-references.md)

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
