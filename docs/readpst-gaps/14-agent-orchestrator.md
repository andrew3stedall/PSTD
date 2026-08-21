# Readpst parity agent orchestrator

_Baseline reviewed: 21 August 2026._

This document is the execution control plane for the `RP-00`–`RP-13` readpst parity register. It turns the source review into a dependency-aware program that can be operated by a global orchestrator and specialist sub-agents. The GitHub issue keys in this document are stable work-unit IDs; the issue and milestone links below are the provisioned workboard.

The authoritative technical baseline remains the pinned `pst-format/libpst` revision [`cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89`](https://github.com/pst-format/libpst/tree/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89), with the source-to-behaviour ledger in [upstream source notes](12-upstream-source-notes.md). The orchestrator may split or resequence implementation issues, but it may not weaken the parity rule, evidence boundary, or fail-closed requirements in [method and parity guardrails](00-method-and-guardrails.md).

## RP-M1-04 delivery note

RP-M1-04 adds the canonical data/evidence.jsonl provenance stream. Its evidence gate
is limited to stable source identity, bounded raw retention, payload linkage, and
explicit unavailable/failed statuses; it does not close later Unicode, output,
relationship, or input-breadth work.

## Operating contract

The program is complete only when every capability that readpst exposes has a PSTD equivalent, a stronger equivalent that preserves the same information, or a machine-readable unsupported result when readpst itself skips the case. A parser helper, property constant, or output filename is not completion evidence.

Every agent must:

1. work from one stable work-unit key and one coherent observable capability;
2. review the exact readpst function/helper and pinned revision before changing code;
3. preserve source identity, raw properties/payloads, provenance, and negative status;
4. add or identify positive and malformed/ambiguous/unsupported evidence before implementation;
5. run the smallest production-integrated vertical slice, not a detached helper or diagnostic-only patch;
6. update the parity matrix, affected topic pages, roadmap, source ledger, and changelog when assumptions change;
7. open a branch and draft PR, wait for green CI, obtain review, and squash-merge only through the repository’s normal gate;
8. report blockers in the issue and return the work unit to the orchestrator rather than silently bypassing a dependency.

The project’s fixture-admission rules remain binding. No private PST, unreviewed external output, or fixture with unclear redistribution rights may be committed merely to unblock an agent.

## Milestones

The milestone names below are exact. They are intentionally vertical: each milestone finishes a user-visible extraction/output surface and its evidence, while allowing independent specialist issues inside the phase.

| Milestone | Scope | Plan IDs | Entry dependency | Exit evidence | Suggested concurrency |
|---|---|---|---|---|---:|
| [`RP-M0 — Control plane and parity evidence`](https://github.com/andrew3stedall/PSTD/milestone/1) | Status/provenance model, differential runner, upstream drift, and agent protocol | `RP-00`, `RP-12`, `RP-13` | Existing validated Unicode baseline | One fixture runs through readpst and PSTD, produces normalized evidence, and is tracked by the orchestrator | 2–3 |
| [`RP-M1 — Typed extraction core`](https://github.com/andrew3stedall/PSTD/milestone/2) | Input capability envelope, bounded parser policy, folders, ownership, visibility, and typed item routing | `RP-02`, `RP-03` | `RP-M0` | A mixed fixture has reconciled folders/items and no silent ordinary-message fallback | 3–4 |
| [`RP-M2 — Unicode message fidelity`](https://github.com/andrew3stedall/PSTD/milestone/3) | Message metadata, headers, charset, bodies, MIME/RTF, and attachment evidence | `RP-04`, `RP-05`, `RP-06` | `RP-M1` | E3 Unicode evidence covers identities, headers, bodies, MIME parts, attachment methods, order, CID, and negative status | 3–4 |
| [`RP-M3 — Relationships and special items`](https://github.com/andrew3stedall/PSTD/milestone/4) | Embedded graph, bounded recursion, reports, schedules, encrypted-body status, and CLI policy | `RP-01`, `RP-07` | `RP-M2` | Nested and special email fixtures produce typed graph/MIME/status records with deterministic limits | 2–3 |
| [`RP-M4 — Non-mail item outputs`](https://github.com/andrew3stedall/PSTD/milestone/5) | Contacts, appointments/recurrence, journals, tasks, sticky notes, and explicit unsupported routing | `RP-08` | `RP-M1`, `RP-M2` | Mixed folders yield reconciled typed records and validated vCard/iCalendar/vJournal values | 2–3 |
| [`RP-M5 — Readpst output adapters`](https://github.com/andrew3stedall/PSTD/milestone/6) | mbox, recursive folder output, MH, EML, KMail, Thunderbird, separate files, typed files, and MSG | `RP-09` | `RP-M3`, `RP-M4` | Each readpst output family has a semantic adapter or explicit equivalent/unsupported result | 3–4 |
| [`RP-M6 — Input breadth and hardening`](https://github.com/andrew3stedall/PSTD/milestone/7) | ANSI, OST 2013, encryption, large/sparse inputs, malformed derivatives, fuzzing, and bounded parallelism | `RP-02`, `RP-13` | `RP-M0`, `RP-M1` | Supported input families and adversarial derivatives have reproducible, bounded evidence | 2–3 |
| [`RP-M7 — Readpst parity release gate`](https://github.com/andrew3stedall/PSTD/milestone/8) | Matrix promotion, E4 differential corpus, release decision, and synchronized documentation | `RP-10`, `RP-11` | `RP-M0`–`RP-M6` | No applicable unresolved matrix rows; repeat-run, output, negative, and documentation gates pass | 2 |

`RP-M6` can run partly in parallel with `RP-M3`–`RP-M5` after the parser envelope exists, but its results cannot be promoted until the differential and status contracts from `RP-M0` are stable.

Milestone provisioning and keyed issue assignment are idempotently automated by [the repository workboard workflow](../../.github/workflows/readpst-parity-workboard.yml). It ran successfully after PR #525 and can be rerun manually; it creates or reopens the exact milestone names above and assigns every issue whose title begins with an `RP-Mx-yy` work-unit key.

## Work-unit register

The `Depends on` column names stable work-unit keys. The global orchestrator must not dispatch a unit until all dependencies are merged or explicitly replaced by an approved equivalent with linked evidence.

### `RP-M0 — Control plane and parity evidence`

| Key | Work unit | Source/readpst anchors | PSTD boundary | Depends on | Specialist |
|---|---|---|---|---|---|
| [RP-M0-01](https://github.com/andrew3stedall/PSTD/issues/497) | Establish parity status, provenance, fixture-manifest, and evidence schemas | `readpst.c:main/process`; `regression/regression-tests.bash`; `00`, `10`, `11` | `tests/readpst_diff/manifest.rs`, canonical status/provenance records, matrix tooling | — | contract/corpus agent |
| [RP-M0-02](https://github.com/andrew3stedall/PSTD/issues/498) | Build the first semantic readpst/PSTD differential runner and report | regression `dopst` profiles; `13` comparator contract | `tests/readpst_diff/{runner,normalize,compare,report}.rs` and one approved fixture | `RP-M0-01` | differential-harness agent |
| [RP-M0-03](https://github.com/andrew3stedall/PSTD/issues/499) | Automate pinned upstream source review and drift detection | `12` ledger; `readpst.c`, `libpst.c/.h`, `msg.cpp`, helper sources, `Makefile.am` | source manifest/check script, revision/hash report, documented drift failure | `RP-M0-01` | upstream-audit agent |
| [RP-M0-04](https://github.com/andrew3stedall/PSTD/issues/500) | Install the global agent orchestration protocol and workboard rules | `00`, `11`, `13`, this document | GitHub issue/PR protocol, dependency state machine, documentation fan-out checklist | `RP-M0-01` | orchestration/release agent |

### RP-M0 delivery note

RP-M0-01, RP-M0-02, and RP-M0-03 are the serialized control-plane slices: contracts first, then the bounded semantic differential runner, then pinned-source drift evidence. The runner’s current accepted fixture evidence is E2/Partial; it records parity, intentional PSTD extension, unsupported, and failure outcomes without promoting downstream matrix rows. Feature dispatch remains subject to the merged-contract and dependency checklist below.

### RP-M1-01 dispatch note

RP-M1-01 is the serialized input-parser slice. Its accepted boundary is the typed capability record and bounded reader policy, with inspect and canonical extraction visibility. It must merge before the dependent typed extraction, charset, and hardening slices are dispatched; unsupported or partial capability states remain explicit and never establish false ownership.
### RP-M1-02 delivery note

RP-M1-02 adds the production `ItemEnvelope` contract and `data/items.jsonl` archive stream over the validated folder/message ownership path. Its evidence boundary is source identity, canonical folder relationships, visibility, ownership status, deterministic ordering, and explicit unknown/duplicate/path-collision outcomes. It does not claim mixed non-mail routing or deleted-item policy. #504 remains serialized until this shared envelope contract is merged and reconciled.

### RP-M1-03 dispatch note

RP-M1-03 owns the serialized class/routing boundary consumed by #504 and the downstream message/non-mail slices. Its accepted implementation must classify source message classes, preserve missing/unknown evidence, and emit explicit visibility/filter/unsupported statuses without changing raw-property ownership. The implementation is now integrated on the classification branch pending draft-PR review and CI; #504 remains blocked until that PR is merged and its evidence packet is recorded.

### RP-M2-01 delivery note

RP-M2-01 adds additive message metadata fields for native representing/received-by
identities, source date summaries, flags, importance/priority/sensitivity, and
receipt/report controls. The evidence gate covers positive unit values, absent-field
null semantics, deterministic canonical serialization, and the existing public
fixture regressions; broad header and producer parity remains with RP-M2-02 onward.

### RP-M2-02 delivery note

RP-M2-02 adds the production `data/headers.jsonl` projection and raw-property
evidence linkage. The gate covers readpst's stored-header authority decision,
folding/field validation, embedded-body truncation, Unicode/String8/default charset
policy visibility, decode/loss reporting, negative statuses, and deterministic
message-key coverage. Current String8 conversion is explicitly recorded as lossy
UTF-8 with the readpst ISO-8859-1 fallback visible; complete item/code-page/`-C`/`-8`
MIME conversion remains downstream work.

### RP-M2-03 dispatch note

RP-M2-03 owns attachment method/source, order, CID, filename/path safety, and raw
payload evidence over the existing attachment table and recursive Property Context
paths. Its acceptance gate requires production `AttachmentRecord` fields and
adapter-independent negative statuses; #508 remains serialized until this and the
header slice are merged.

### RP-M2-04 delivery note

RP-M2-04 is merged on main as the canonical body/MIME/RTF projection slice. It adds
bounded RTF validation/recovery and `data/mime_parts.jsonl` over the production body,
header, attachment, and embedded evidence graph. The repeat-run workflow verifies
stable part keys, ownership, raw/decoded hash linkage, and unresolved HTML safety.
Report, schedule, encrypted-body, recursive child, and output-adapter semantics stay
serialized behind RP-M3/RP-M5 rather than being silently inferred here.

### RP-M3-01 delivery note

RP-M3-01 is implemented as the bounded graph slice on the production canonical path.
`data/embedded_graph.jsonl` records method-5 parent/attachment/child edges with stable
IDs, source order, child evidence links, observed bytes, depth, and explicit
resolution/cycle/budget statuses. The Tika attachment workflow repeats extraction and
checks exact child ownership, payload/hash linkage, child MIME evidence, and graph
JSONL determinism. Special-item semantics, CLI policy, and `.msg` output remain
serialized behind RP-M3-02/RP-M3-03 and RP-M5-04.

### RP-M3-02 dispatch note

RP-M3-02 owns typed special-item projection over canonical body/header evidence.
Reports and schedules preserve raw readable payloads while refusing to guess absent
report-type, calendar-method, or recurrence properties; encrypted candidates are
opaque; validated RTF receives a synthetic non-authoritative MIME attachment. Its
repeat-run workflow is the gate for `data/special_items.jsonl`; CLI flag translation
and adapter wire policy remain downstream. The RTF evidence path is pinned to the
repository fixture `tests/fixtures/pst/sample.pst` (SHA-256
`ee997fc7dd5c40bef49b753b782f76b17109057b18c19232cc87e0b63e0711fe`), while
report/schedule/encrypted negative cases remain covered by synthetic unit evidence.

### RP-M3-03 dispatch note

RP-M3-03 owns the typed CLI policy boundary. `ReadpstPolicy` validates named output
profiles, visibility/type filters, charset and attachment policies, diagnostics,
collision/overwrite policy, and bounded jobs before extraction. The canonical path
records the policy and applies routing filters without removing provenance. Legacy
profiles fail explicitly as unsupported until their adapter work is complete.

### RP-M4-01 dispatch note

RP-M4-01 owns contact records and the first non-mail adapters. It may consume only
source-backed contact fields from canonical message metadata, must retain missing-field
statuses and evidence, and must not convert contact objects into email. The admitted
java-libpst distribution-list fixture is negative/partial for PSTD contact-class
authority; synthetic serializer unit evidence must declare its provenance.


### `RP-M1 — Typed extraction core`

| Key | Work unit | Source/readpst anchors | PSTD boundary | Depends on | Specialist |
|---|---|---|---|---|---|
| [RP-M1-01](https://github.com/andrew3stedall/PSTD/issues/501) | Add an input capability envelope and bounded parser policy | `pst_open`, `pst_reopen`, `pst_load_index`, `pst_load_extended_attributes`, `vbuf.c`, `timeconv.c` | `src/pst/header.rs`, `reader.rs`, `limits.rs`, input-family/crypt/status records | `RP-M0-01`, `RP-M0-03` | input-parser agent |
| [RP-M1-02](https://github.com/andrew3stedall/PSTD/issues/502) | Emit a typed folder/item envelope with ownership and provenance | `readpst.c:process`, `libpst.c:pst_process`, descriptor-tree constants | `src/pst/folder_tree.rs`, `folders.rs`, new `ItemEnvelope` projection | `RP-M1-01` | envelope/ownership agent |
| [RP-M1-03](https://github.com/andrew3stedall/PSTD/issues/503) | Route mixed, deleted, associated, and unsupported classes explicitly | `process`, item-type constants, `-t` filter branches, `write_*` dispatch | visibility policy, item-kind enum, skipped/filtered/unavailable records | `RP-M1-02` | classification agent |
| [RP-M1-04](https://github.com/andrew3stedall/PSTD/issues/504) | Preserve property, subnode, attachment-reference, and raw-payload provenance | `pst_parse_item`, `pst_attach_to_file`, ID2/subnode handling, `vbuf.c` | `property_context`, `subnodes`, `attachments`, canonical evidence graph | `RP-M1-01`, `RP-M1-02` | property/evidence agent |

### `RP-M2 — Unicode message fidelity`

| Key | Work unit | Source/readpst anchors | PSTD boundary | Depends on | Specialist |
|---|---|---|---|---|---|
| [RP-M2-01](https://github.com/andrew3stedall/PSTD/issues/505) | Complete message metadata, identities, dates, flags, and report controls | `write_normal_email`, header helpers, email fields in `libpst.h`, `msg.cpp` | `src/engine/metadata.rs`, `messages.rs`, recipient and flag records | `RP-M1-02`, `RP-M1-03`, `RP-M1-04` | message-metadata agent |
| [RP-M2-02](https://github.com/andrew3stedall/PSTD/issues/506) | Validate stored headers, RFC fields, String8/Unicode conversion, and charset policy | `valid_headers`, `pst_default_charset`, `libstrfunc.c`, `-C`, `-8` | header provenance/status, code-page policy, forensic raw header retention | `RP-M2-01` | header/charset agent |
| [RP-M2-03](https://github.com/andrew3stedall/PSTD/issues/507) | Resolve attachment methods, reference payloads, filenames, order, CID, and OLE evidence | `acceptable_ext`, `write_separate_attachment`, `write_inline_attachment`, attachment table/ID2 logic | `src/pst/attachments.rs`, `attachment_table.rs`, output attachment evidence | `RP-M1-03`, `RP-M1-04` | attachment agent |
| [RP-M2-04](https://github.com/andrew3stedall/PSTD/issues/508) | Build canonical body selection and semantic MIME/RTF projection | `write_body_part`, `write_normal_email`, `pst_lzfu_decompress`, `libstrfunc.c` | body set, MIME tree, transfer/charset status, raw-body fallback | `RP-M2-01`, `RP-M2-02`, `RP-M2-03` | body/MIME agent |

### `RP-M3 — Relationships and special items`

| Key | Work unit | Source/readpst anchors | PSTD boundary | Depends on | Specialist |
|---|---|---|---|---|---|
| [RP-M3-01](https://github.com/andrew3stedall/PSTD/issues/509) | Add nested embedded-message graph resolution with recursion and resource budgets | `write_embedded_message`, `msg.cpp` attachment storage, method-5 limitation | child edges, bounded graph traversal, cycle/depth/byte statuses | `RP-M1-03`, `RP-M2-03`, `RP-M2-04` | relationship/graph agent |
| [RP-M3-02](https://github.com/andrew3stedall/PSTD/issues/510) | Export reports, schedule/meeting parts, encrypted-body status, and special MIME branches | `write_schedule_part_data`, report branches, encrypted/RTF branches in `write_normal_email` | typed report/schedule/encrypted records and MIME parts | `RP-M2-02`, `RP-M2-04`, `RP-M3-01` | special-email agent |
| [RP-M3-03](https://github.com/andrew3stedall/PSTD/issues/511) | Translate CLI flags into typed profiles, filters, diagnostics, and deterministic scheduling | `readpst.c:main`, `create_enter_dir`, `close_enter_dir`, `mk_*`, `acceptable_ext`, regression profiles | `src/cli.rs`, `config.rs`, output policy/scheduler/diagnostic records | `RP-M1-03`, `RP-M2-01` | CLI/operations agent |

### `RP-M4 — Non-mail item outputs`

| Key | Work unit | Source/readpst anchors | PSTD boundary | Depends on | Specialist |
|---|---|---|---|---|---|
| [RP-M4-01](https://github.com/andrew3stedall/PSTD/issues/512) | Route contacts and produce vCard/simple contact-list records | `write_vcard`, `write_extra_categories`, contact fields in `libpst.h` | contact record, vCard serializer, skip/status evidence | `RP-M1-03`, `RP-M2-01` | contacts/vCard agent |
| [RP-M4-02](https://github.com/andrew3stedall/PSTD/issues/513) | Route appointments, recurrence, and iCalendar values | `write_appointment`, `pst_convert_recurrence`, appointment fields in `libpst.h` | calendar record, recurrence normalization, iCalendar serializer | `RP-M1-03`, `RP-M2-01`, `RP-M2-02` | calendar agent |
| [RP-M4-03](https://github.com/andrew3stedall/PSTD/issues/514) | Route journals, tasks, sticky notes, and unsupported non-mail classes | `write_journal`, item-class dispatch, `NEWS`/`ChangeLog` skip behaviour | journal/task/note records, explicit unsupported/filtered statuses | `RP-M1-03`, `RP-M2-01` | non-mail classification agent |

### `RP-M5 — Readpst output adapters`

| Key | Work unit | Source/readpst anchors | PSTD boundary | Depends on | Specialist |
|---|---|---|---|---|---|
| [RP-M5-01](https://github.com/andrew3stedall/PSTD/issues/515) | Add default/recursive mbox, MH/rfc822, and extended EML adapters | `create_enter_dir`, `close_enter_dir`, `mk_separate_dir`, `mk_separate_file`, regression profiles | output writers over canonical records; atomic paths and semantic comparison | `RP-M2-04`, `RP-M3-03` | mailbox/EML adapter agent |
| [RP-M5-02](https://github.com/andrew3stedall/PSTD/issues/516) | Add separate binary attachments, extension filters, KMail, and attachment-safe paths | `write_separate_attachment`, `acceptable_ext`, `mk_kmail_dir/file` | `src/output/paths.rs`, TAR/JSONL evidence, KMail adapter | `RP-M2-03`, `RP-M3-03` | attachment-output agent |
| [RP-M5-03](https://github.com/andrew3stedall/PSTD/issues/517) | Add Thunderbird sidecars and typed contact/calendar/journal file adapters | `mk_thunderbird_dir`, `write_vcard`, `write_appointment`, `write_journal` | `.type`/`.size` sidecars and typed output profiles | `RP-M4-01`, `RP-M4-02`, `RP-M4-03`, `RP-M3-03` | interoperability agent |
| [RP-M5-04](https://github.com/andrew3stedall/PSTD/issues/518) | Implement OLE `.msg` output with a round-trip gate | `src/msg.cpp`, property/recipient/attachment storages; method-5 limitation | Rust OLE writer, supported-property map, independent MSG reader | `RP-M2-01`, `RP-M2-04`, `RP-M3-01` | MSG/OLE agent |

### `RP-M6 — Input breadth and hardening`

| Key | Work unit | Source/readpst anchors | PSTD boundary | Depends on | Specialist |
|---|---|---|---|---|---|
| [RP-M6-01](https://github.com/andrew3stedall/PSTD/issues/519) | Add ANSI v14/v15 and OST 2013 traversal and extraction | `pst_open`, header/index family branches, regression input categories | header family dispatch, fixture admission, canonical output across families | `RP-M0-02`, `RP-M1-01`, `RP-M1-02` | legacy-input agent |
| [RP-M6-02](https://github.com/andrew3stedall/PSTD/issues/520) | Add compressible/strong encryption handling and explicit failure semantics | `pst_open` crypt methods, `libpst.c` decryption paths, `NEWS` | crypto policy/status, password/config plumbing, raw evidence retention | `RP-M1-01`, `RP-M0-02` | crypto agent |
| [RP-M6-03](https://github.com/andrew3stedall/PSTD/issues/521) | Harden malformed/large/sparse inputs, fuzz limits, and bounded parallel runs | `vbuf.c`, `debug.c`, regression/valgrind modes, `readpst` global traversal state | parser/output budgets, path safety, deterministic worker scheduling, fuzz derivatives | `RP-M0-02`, `RP-M1-01`, `RP-M3-03` | security/performance agent |

### `RP-M7 — Readpst parity release gate`

| Key | Work unit | Source/readpst anchors | PSTD boundary | Depends on | Specialist |
|---|---|---|---|---|---|
| [RP-M7-01](https://github.com/andrew3stedall/PSTD/issues/522) | Promote the parity matrix from evidence through all applicable rows | all topic docs; `regression-tests.bash` profiles | `10-parity-matrix.md`, fixture inventory, status promotion report | `RP-M0-02`, `RP-M1-03`, `RP-M2-04`, `RP-M3-02`, `RP-M4-03`, `RP-M5-04`, `RP-M6-03` | matrix/release agent |
| [RP-M7-02](https://github.com/andrew3stedall/PSTD/issues/523) | Run E4 semantic differentials across output families and input variants | regression profiles, `13` comparator contract | approved corpus, normalized reports, repeat-run/worker equality | `RP-M0-02`, `RP-M5-01`, `RP-M5-02`, `RP-M5-03`, `RP-M5-04`, `RP-M6-01`, `RP-M6-02`, `RP-M6-03` | differential/release agent |
| [RP-M7-03](https://github.com/andrew3stedall/PSTD/issues/524) | Publish the parity decision and recursively synchronize all documentation | `NEWS`, `ChangeLog`, `12` source ledger, all topic pages | README/status/roadmap/changelog release record and remaining-gap decision | `RP-M7-01`, `RP-M7-02`, `RP-M0-03`, `RP-M0-04` | documentation/release agent |

## Agent roles and boundaries

Agents are specialists, not independent project owners. The global orchestrator owns sequencing, issue state, dependency changes, and final promotion. A specialist may propose a dependency change, but cannot remove one without updating the work-unit register and matrix with evidence.

| Role | Owns | Must not do |
|---|---|---|
| Orchestrator/release | ready queue, dependency graph, milestone health, blocked decisions, merge sequencing | merge a feature that lacks its evidence or documentation fan-out |
| Contract/corpus | status vocabulary, provenance, fixture IDs/hashes, admission records | admit private or unclear-license payloads |
| Upstream-audit | pinned source anchors, drift checks, behavioural notes | copy GPL implementation into PSTD |
| Input-parser/crypto | header families, decryption, parser limits, malformed handling | infer a family/crypt mode from a display name alone |
| Envelope/ownership | folder graph, source IDs, visibility, item classification | silently coerce unknown items to messages |
| Property/evidence | raw property/subnode/reference retention | discard raw bytes because projection failed |
| Message/header/body | metadata, identities, headers, charset, MIME, RTF | rewrite authoritative native addresses as SMTP without evidence |
| Attachment/graph | method/reference/OLE/CID resolution and child edges | attach ambiguous children or unbounded recurse |
| Non-mail/interoperability | contact/calendar/journal outputs and adapters | implement a format by renaming another format |
| CLI/operations | profile mapping, filters, deterministic scheduling, diagnostics | let global `chdir` or shared mutable output state decide semantics |
| Security/performance | budgets, path safety, fuzz/large-file evidence, worker equality | hide a timeout or allocation failure as success |
| Matrix/reviewer | semantic differential, status promotion, docs fan-out | promote a row from one narrow fixture |

An agent may work in parallel only when the orchestrator confirms that its changed modules and fixture manifests do not overlap an in-flight unit. Shared contracts (`ItemEnvelope`, status enums, canonical output schema, comparator normalization) are serialized through the orchestrator.

## Global orchestrator loop

The orchestrator should run this loop after every issue creation, PR update, merge, new fixture, or upstream-drift signal:

```text
load README, matrix, roadmap, source ledger, and agent workboard
verify current main and CI state
for each work unit:
    recompute dependency status from merged issues/PRs and evidence links
    if dependency is missing, mark BLOCKED and do not dispatch
    if evidence boundary is not admitted, mark WAITING-EVIDENCE and do not invent a fixture
    if ready and no conflicting module/contract is active, dispatch one specialist
for each completed specialist result:
    require source-review note, implementation diff, fixture evidence, and docs fan-out
    run unit/fixture/differential/determinism/limit checks
    open or update draft PR and request review
    merge only after CI and review are green
after each merge:
    update matrix/status and all tangential docs
    re-evaluate downstream readiness recursively
    refresh milestone counts and the global orchestration issue
stop only at RP-M7-03 or an explicitly recorded blocker requiring maintainer input
```

### Issue state machine

Use these states in the issue body and in the orchestrator comment; GitHub open/closed state alone is insufficient:

`PLANNED` → `READY` → `IN PROGRESS` → `EVIDENCE REVIEW` → `PR REVIEW` → `CI GREEN` → `MERGED`.

Side states are `BLOCKED`, `WAITING-EVIDENCE`, `REJECTED-SCOPE`, and `SUPERSEDED`. A closed issue must name the resulting PR, evidence report, matrix rows, and any replacement work unit. A blocked issue must name the exact dependency or missing admissible evidence and the next recheck event.

### Dispatch packet

The orchestrator assigns an agent by adding a comment with:

```markdown
## Agent dispatch

- Work unit: `RP-Mx-yy`
- State: `IN PROGRESS`
- Specialist role: [role]
- Branch: `agent/readpst-rp-mx-yy-[short-name]`
- Readpst revision: `cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89`
- Source anchors: [functions and links]
- Dependencies verified: [issue links]
- Allowed module boundary: [paths]
- Positive evidence: [fixture IDs or approved synthetic construction]
- Negative evidence: [malformed/ambiguous/unsupported case]
- Documentation fan-out: [links]
- Exit command(s): [exact commands]
```

The agent returns the same packet with implementation commit/PR, test output, evidence hashes, observed gaps, and any matrix/documentation changes. This makes handoff reviewable without relying on private agent memory.

## Merge and promotion policy

- One issue maps to one bounded PR unless the orchestrator records why a split is required.
- PRs target `main`, are draft until the implementation and evidence are complete, and are squash-merged only after required CI is green.
- A PR may update a downstream issue’s text or dependencies only when it also updates this work-unit register and the affected topic documents.
- Shared schema changes must land before dependent projections. If a schema must change after consumers exist, the orchestrator pauses consumers, opens a migration issue, and updates every dependent issue before dispatch resumes.
- A merged implementation does not promote a matrix row automatically. Promotion belongs to `RP-M7-01` after E2/E3/E4 evidence and negative-status checks.
- “Parity” is never inferred from an empty output directory, a matching filename, a successful process exit, or a single fixture.

## Recursive documentation fan-out

Every merged work unit must check the following graph and update only the pages whose assumptions changed:

```text
implementation / fixture / comparator change
  -> affected topic document (01–09)
  -> 10-parity-matrix.md
  -> 11-roadmap-and-acceptance.md
  -> 12-upstream-source-notes.md (if source behaviour or revision changed)
  -> 13-issue-template-and-differential-harness.md (if issue/evidence schema changed)
  -> README.md
  -> docs/DOCUMENTATION_STATUS.md, docs/README.md, docs/changelog/unreleased.md
```

The agent must record “no change required” for each tangential page it inspected. This prevents a body or attachment implementation from leaving stale output, dependency, or acceptance claims in another document.

## Global completion checklist

- [x] all eight GitHub milestones exist with the exact names in this document;
- [x] all 28 work-unit issues exist, use their stable keys, and link dependencies;
- [x] every issue is assigned to one milestone and one specialist role;
- [x] RP-M0-01 status/provenance/fixture/evidence schema and E2 Unicode baseline contract are implemented and CI-validated;
- [x] RP-M0-03 pinned source manifest, 28 work-unit anchors, and deterministic drift report are implemented and CI-validated;
- [ ] `RP-M0` evidence and differential contracts are merged before feature dispatch;
- [ ] no active issue bypasses the typed envelope or raw-evidence boundary;
- [ ] every output adapter consumes canonical records rather than reparsing PST bytes;
- [ ] all supported input families have provenance, hashes, limits, and repeat-run evidence;
- [ ] matrix rows include equivalent/stronger/unsupported outcomes and negative statuses;
- [ ] E4 semantic differentials cover every readpst regression profile and every PSTD output profile;
- [ ] final documentation, changelog, and release status agree with the matrix.

The global orchestration issue should remain open until this checklist is either complete or replaced by an explicitly approved release decision that names every unresolved plan ID.
