# PSTD/readpst parity gap register

_Baseline reviewed: 21 August 2026._

This folder is the compatibility register for making PSTD capable of everything that the `readpst` program in `pst-format/libpst` can do. It is deliberately broader than the current PSTD email-to-EML milestone: `readpst` is a mature command-line extractor with multiple input variants, output modes, MIME behaviours, attachments, contacts, appointments, journals, and operational controls.

RP-M5-02 is now integrated as a Partial output capability: separate binary attachment
files and the KMail directory projection consume canonical records, with explicit filter,
payload, path, index-policy, and determinism evidence. Remaining output and input gaps
stay visible in the matrix rather than being promoted by profile recognition alone.

RP-M5-03 adds the Partial Thunderbird/typed-file capability: recursive mbox output,
canonical-identity `.type`/`.size` sidecars, and independent typed projections for
non-mail records. An unavailable folder type remains explicit rather than guessed.

RP-M5-04 adds the Partial MSG/OLE capability: a Rust-native CFB/OLE writer, a separate
readpst-compatible EML companion, supported MAPI properties, recipient/attachment
storages, explicit unsupported method-5/property decisions, and an independent reader
gate. Named properties, embedded-message breadth, and input/release gates remain open.

RP-M6-01 adds Partial ANSI v14/v15 and OST 2013 structural input coverage through the
canonical header/index path. Controlled positive fixtures prove family-specific roots,
page widths, BBT/NBT identities, explicit negative statuses, and repeat-run equality;
full semantic item/output corpora and hardening remain open. RP-M6-02 adds production
payload decoding for libpst crypt methods 1 and 2, method-2 capability readiness, and
explicit unknown-method failures; password validation is not applicable because these
pinned NDB methods derive their transform from the block ID.
RP-M6-03 adds bounded batch workers, symlink-safe recursive discovery, archive path
confinement, atomic TAR shard publication, and a diagnostic budget cap; malformed,
resource-heavy, and unsafe filesystem cases remain explicit failures.

## Baseline

The comparison is against the `pst-format/libpst` `master` source at commit [`cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89`](https://github.com/pst-format/libpst/tree/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89). The principal implementation is [`src/readpst.c`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c); the exposed item model and parser behaviour are in [`src/libpst.h`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.h) and [`src/libpst.c`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.c).

This is a behaviour comparison, not a proposal to link PSTD to libpst. PSTD remains a self-contained Rust implementation. GPL-covered source is not copied into PSTD; only observable behaviour, format requirements, and test ideas are used.

## Read this first

| Need | Document |
|---|---|
| Understand the comparison rules and evidence standard | [Method and parity guardrails](00-method-and-guardrails.md) |
| Compare commands, flags, and output modes | [CLI and output-mode parity](01-cli-and-output-parity.md) |
| Compare PST/OST variants, encryption, and parser foundations | [Input and parser compatibility](02-input-and-parser-compatibility.md) |
| Compare folder traversal and every item class | [Folders and item types](03-folders-and-item-types.md) |
| Compare email metadata, headers, flags, and character sets | [Message metadata and headers](04-message-metadata-and-headers.md) |
| Compare body decoding and MIME reconstruction | [Bodies, MIME, and RTF](05-body-mime-and-rtf.md) |
| Compare attachment metadata, payloads, references, and filters | [Attachment parity](06-attachments.md) |
| Compare embedded messages, reports, meetings, and encrypted bodies | [Embedded and special email items](07-embedded-and-special-email-items.md) |
| Compare contacts, appointments, journals, and unsupported classes | [Non-mail item outputs](08-contacts-calendar-journal.md) |
| Compare mbox, MH, KMail, Thunderbird, EML, and MSG results | [Storage and interoperability outputs](09-storage-and-interoperability.md) |
| See one authoritative checklist of all identified capabilities | [Parity matrix](10-parity-matrix.md) |
| Turn the gaps into implementation and fixture work | [Parity roadmap and acceptance](11-roadmap-and-acceptance.md) |
| Locate the exact upstream code used for each observation | [Upstream source notes](12-upstream-source-notes.md) |
| Create implementation issues and run semantic differentials | [Issue template and differential harness](13-issue-template-and-differential-harness.md) |
| Run the dependency-aware multi-agent implementation program | [Readpst parity agent orchestrator](14-agent-orchestrator.md) |

## Planned implementation architecture

The register is also an issue-ready implementation blueprint. The stable plan IDs below are the IDs to carry into GitHub issues, pull requests, fixture manifests, and matrix rows. They are deliberately stable even if a later issue is split into smaller pull requests.

| Plan ID | Work package | Primary document | Depends on |
|---|---|---|---|
| `RP-00` | Evidence, status, provenance, and differential guardrails | [Method and parity guardrails](00-method-and-guardrails.md) | — |
| `RP-01` | CLI translation, output profiles, scheduling, and diagnostics | [CLI and output-mode parity](01-cli-and-output-parity.md) | `RP-00`, `RP-03` |
| `RP-02` | Input families, encryption, indexes, charset, and parser limits | [Input and parser compatibility](02-input-and-parser-compatibility.md) | `RP-00` |
| `RP-03` | Folder walker, visibility, typed item envelope, and classification | [Folders and item types](03-folders-and-item-types.md) | `RP-02` |
| `RP-04` | MAPI projection, headers, identities, flags, and dates | [Message metadata and headers](04-message-metadata-and-headers.md) | `RP-02`, `RP-03` |
| `RP-05` | Body selection, MIME tree construction, charset, transfer encoding, and RTF | [Bodies, MIME, and RTF](05-body-mime-and-rtf.md) | `RP-04`, `RP-06` |
| `RP-06` | Attachment methods, references, filenames, CID, and payload evidence | [Attachment parity](06-attachments.md) | `RP-02`, `RP-03` |
| `RP-07` | Embedded-object graph, schedules, reports, encrypted bodies, and synthetic parts | [Embedded and special email items](07-embedded-and-special-email-items.md) | `RP-03`, `RP-05`, `RP-06` |
| `RP-08` | Contact, appointment, journal, task, sticky-note, and other typed outputs | [Non-mail item outputs](08-contacts-calendar-journal.md) | `RP-03`, `RP-04` |
| `RP-09` | mbox, recursive, MH, EML, KMail, Thunderbird, vCard/iCalendar, and MSG adapters | [Storage and interoperability outputs](09-storage-and-interoperability.md) | `RP-04`–`RP-08` |
| `RP-10` | Matrix maintenance and status promotion gates | [Parity matrix](10-parity-matrix.md) | all applicable plans |
| `RP-11` | Phased rollout, fixtures, acceptance, and release gate | [Parity roadmap and acceptance](11-roadmap-and-acceptance.md) | `RP-00`–`RP-10` |
| `RP-12` | Upstream source-review ledger and drift checks | [Upstream source notes](12-upstream-source-notes.md) | `RP-00` |
| `RP-13` | Issue body template and semantic differential harness | [Issue template and differential harness](13-issue-template-and-differential-harness.md) | `RP-00` |

The intended data flow is one bounded parse followed by typed projections:

```text
PST/OST bytes
  -> input/header/crypto reader
  -> indexes, folders, properties, subnodes, payload references
  -> ItemEnvelope { identity, visibility, type, provenance, status }
  -> typed message/contact/calendar/journal/report records
  -> canonical JSONL/TAR evidence and raw payloads
  -> independent output adapters (mbox, MH, EML, KMail, Thunderbird, vCard, iCalendar, MSG)
```

Every adapter consumes the envelope and evidence graph; none reparses the PST. Canonical records retain source values and statuses even when an output profile filters or cannot render them. This is the central mechanism for satisfying the readpst surface while improving on its global state, `chdir`-based traversal, unbounded embedded recursion, weak filename sanitation, and silent loss paths.

RP-M1-04 adds data/evidence.jsonl to the canonical evidence graph. It links decoded
properties, subnode references, body payloads, and attachment payloads to stable
owners with bounded raw bytes, hashes, and explicit unavailable/failed statuses.

RP-M2-04 adds `data/mime_parts.jsonl` as the canonical semantic MIME projection over
those evidence records. It is a production evidence boundary, not a claim that the
readpst output-adapter family or typed special-item exports are complete.

RP-M3-01 adds `data/embedded_graph.jsonl` as a bounded, source-keyed relationship
projection over method-5 embedded-message references. It preserves child ownership,
payload observations, evidence links, and explicit non-authoritative cycle, budget,
missing, ambiguous, and non-email outcomes; it does not promote special-item or
output-adapter parity.

RP-M3-02 adds typed `data/special_items.jsonl` records and special MIME projections
for reports, schedules/meetings, encrypted bodies, and synthetic RTF. Source raw
evidence remains separate from normalized or generated values, and encrypted or
unavailable content cannot be promoted to decoded authoritative output.

The execution control plane for implementing this architecture is [the readpst parity agent orchestrator](14-agent-orchestrator.md). It defines eight delivery milestones, 28 stable work-unit keys, dependency gates, specialist sub-agent roles, branch/PR rules, and recursive documentation fan-out. GitHub issues should use those keys verbatim so implementation, fixtures, matrix rows, and agent handoffs remain traceable.

## Issue slicing rules

Each implementation issue should name one plan ID, one observable capability, one source-review anchor, one affected PSTD module, one fixture or synthetic corpus, and one semantic acceptance command. A plan issue is not done when a struct or parser helper exists: it is done only when the canonical record, output projection where applicable, negative status, deterministic repeat run, and matrix update are all present. Cross-cutting changes must update every tangential document listed in the issue’s “Documentation fan-out” field.

## Status vocabulary

The status in this folder is intentionally stricter than “there is a field or helper for it”.

| Status | Meaning |
|---|---|
| **Implemented** | PSTD has an observable equivalent and current fixture or regression evidence for the stated boundary. |
| **Partial** | PSTD has a contract, parser primitive, or one validated layout, but not the breadth or output behaviour that readpst provides. |
| **Gap** | No equivalent user-visible behaviour is currently proven. |
| **Explicitly unsupported by readpst** | The upstream program classifies or skips the case rather than exporting it. PSTD must still classify it and report the outcome without silently dropping it. |

“Partial” and “Gap” are parity work, even when the existing code is useful groundwork. A capability is not complete merely because its property constant, record field, or parser probe exists.

## Current conclusion

PSTD currently has meaningful, fixture-validated Unicode message extraction, selected metadata, recipients, text/HTML/RTF evidence, by-value attachment evidence, one embedded-message layout, deterministic EML assembly, batch processing, and structured TAR/JSONL output. It does **not** yet have readpst parity.

The largest gaps are:

- ANSI traversal and extraction, OST 2013 coverage, and PST encryption handling;
- the remaining readpst output breadth: reduced typed streams, full vCard/list and calendar/journal fidelity, named MSG properties, and exact Thunderbird import compatibility;
- complete item-class routing for contacts, appointments, journals, reports, tasks, sticky notes, and other classes;
- broad message-property, charset, RFC header, and forensic metadata coverage;
- all attachment methods, reference resolution, OLE handling, inline CID correlation, and nested embedded-message recursion;
- exact vCard, vCalendar, vJournal, multipart/report, and meeting-request outputs;
- an acceptance corpus that proves these behaviours across Unicode, ANSI, OST, encrypted, malformed, mixed-type, and large-file inputs.

## Parity rule

For every capability that readpst exposes, PSTD must provide one of the following before the capability can be marked complete:

1. an equivalent PSTD command, API, or structured-output result;
2. an explicit, machine-readable unsupported result when readpst itself skips the case; or
3. a stronger equivalent that preserves the same information and does not silently discard data.

PSTD does not need to reproduce libpst’s legacy filenames or byte-for-byte output formatting as its canonical representation. It does need compatibility adapters or documented equivalent output for each readpst mode, with deterministic results and preserved source evidence.

RP-M5-01 is the first adapter delivery: default mbox, recursive mbox, MH/rfc822,
extended EML, and separate numbered message files now consume canonical PSTD records.
The delivery is intentionally Partial pending the pinned readpst differential corpus and
the downstream binary-attachment, KMail, Thunderbird, and MSG gates.

RP-M7-01 has completed the conservative matrix promotion review on main commit
`57fbcaf1a83e2ddc79fff300be812a23cc66bb53`. The current ledger contains 2 Implemented,
54 Partial, and 19 Gap rows. The Gap set and its release consequence are recorded in
`10-parity-matrix.md`; RP-M7-02 owns the independent semantic differential and RP-M7-03
owns the final compatibility decision. The project must not claim full readpst parity
while those rows remain unresolved.

RP-M7-02 has now run the pinned readpst oracle against the approved Unicode fixture.
Run `32512518536` passed the 18-test harness and uploaded artifact `9457584897`; the
result is E2/Partial differential evidence with explicit profile/input admissibility
blockers in [`rp-m7-02-e4-report.md`](rp-m7-02-e4-report.md), not a release-wide E4 pass.

RP-M7-03 has published the final decision in
[`rp-m7-03-parity-decision.md`](rp-m7-03-parity-decision.md): PSTD was **not
parity-complete** at the reviewed baseline. The post-decision output expansion has
since moved five rows to Partial, and the easiest-closure wave has promoted six
policy/projection rows to Implemented; the attachment metadata closure wave has now
promoted two additional rows, bringing the maintained ledger to 10 Implemented, 53
Partial, and 14 Gap rows. Future work must still add admissible corpora and semantic
differentials before making a full-parity claim.

RP-M7-03 remains the historical release decision for its reviewed baseline. The
current matrix is maintained by subsequent parity deliveries and must not be read as a
full-parity claim merely because a row moved from Gap to Partial.

## Maintenance rule

When a gap is implemented, update the matrix, the relevant topic document, the current-state project documentation, and the fixture evidence. Every row must retain:

- the upstream behaviour;
- the current PSTD status;
- the exact acceptance boundary;
- the fixture or synthetic test that proves it;
- the fail-closed behaviour for malformed or ambiguous input.

Do not promote a row from Partial to Implemented because it passes one PST. The project’s existing fixture and Purview admission rules remain binding.

## Planned implementation

1. Land `RP-00` and `RP-13` first so every later change has a shared status vocabulary, provenance shape, issue body, and semantic comparator.
2. Build `RP-02` and `RP-03` as the parser-to-envelope boundary. This is where ANSI/Unicode/OST/encryption evidence, folder ownership, deletion visibility, and item classification become reusable inputs to all output profiles.
3. Complete `RP-04`, `RP-05`, and `RP-06` as typed projections over that envelope. Their outputs must retain raw bytes and per-field status, so a failed header or body projection cannot erase a usable attachment or non-mail item.
4. Add `RP-07` and `RP-08` for graphs and non-mail item classes, then use `RP-09` to project the same records into each readpst mode. `.msg` is a separate gate because `msg.cpp` writes an OLE compound document rather than a text format.
5. Use `RP-10` and `RP-11` as the recursive documentation and release gate: every implementation or fixture change updates the matrix, source ledger, affected topic pages, current-state docs, and changelog before promotion. Use [the agent orchestrator](14-agent-orchestrator.md) to dispatch the resulting work units and re-evaluate dependencies after every merge.

The first implementation slice should therefore be an issue cluster, not an output-only patch: `RP-00` → `RP-02`/`RP-03` → `RP-04`/`RP-06` → `RP-05`/`RP-07`/`RP-08` → `RP-09`, with `RP-10`/`RP-11`/`RP-13` enforced throughout.
