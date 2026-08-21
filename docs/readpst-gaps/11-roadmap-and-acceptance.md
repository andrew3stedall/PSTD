# Parity roadmap and acceptance

The goal is full readpst capability coverage, not a collection of unrelated adapters. Work should proceed in vertical slices that add an observable behaviour and the fixture evidence that proves it.

## Phase 0 — parity contract and typed item envelope

Create the shared foundation before adding many output formats:

- item visibility and item-class classification;
- typed common metadata with source/provenance fields;
- contact, appointment, journal, report, and schedule records;
- attachment method/reference metadata;
- explicit output profile and policy records;
- per-item and per-folder complete/partial/skipped/unavailable counts;
- canonical raw-property and raw-payload retention rules.

Exit criterion: a mixed synthetic folder can be inspected without any item being silently labelled as an ordinary message.

### RP-M0-01 delivery

The initial evidence/control slice is implemented in `tests/readpst_diff/manifest.rs` and `tests/readpst_diff_contract.rs`. It validates the approved Apache Tika Unicode fixture (8 folders, 8 messages, 10 bodies, 9 recipients, 2 attachment records, and 12,315 payload bytes), pins readpst revision `cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89`, retains negative and ambiguous outcomes, and proves repeated serialization is deterministic. This is E2/Partial fixture evidence only; RP-M0-02 and RP-M0-03 remain required before feature rows can be promoted.

### RP-M0-03 delivery

The upstream-audit slice is implemented in `tests/readpst_diff/source_manifest.rs`. It validates the pinned `libpst/readpst` revision, all 22 direct ledger paths, all 28 work-unit source mappings, and the eight regression profiles; it emits a deterministic drift report and rejects revision/source-boundary changes until reviewed. It does not import GPL implementation code or require private fixtures/network access.

### RP-M0-02 delivery

The semantic differential slice is implemented in `tests/readpst_diff/{runner,normalize,compare,report}.rs` with the approved Tika fixture contract and a pinned readpst workflow. It records bounded execution, normalized records, explicit negative outcomes, artifact provenance, and repeat-run determinism; the CLI oracle is built with its optional Python interface disabled. This is E2/Partial control-plane evidence only, so feature rows remain unchanged until their own implementation and corpus gates pass.

### RP-M1-01 delivery

The first typed-core slice is implemented across `src/pst/capability.rs`, the header/index-family decoder, bounded reader/limits policy, inspect projection, and canonical extraction archive. Positive Unicode capability evidence and controlled ANSI/OST/unknown/crypt/short-header/budget negatives are covered by deterministic tests. The slice is deliberately a prerequisite contract; it does not promote broad Unicode traversal or any output-mode row.
### RP-M1-02 delivery

The typed folder/item envelope is integrated into metadata extraction and canonical TAR/JSONL output. It records folder and message source identity, ownership, visibility, item-kind confidence, parent/child links, provenance references, and explicit duplicate, path-collision, unknown, and ambiguous statuses. The current evidence is the approved Tika Unicode fixture plus deterministic synthetic ownership tests; no mixed-folder non-mail capability row is promoted by this slice.

### RP-M1-03 delivery

The classification slice is integrated into `src/pst/item_envelope.rs` and `src/pst/item_routing.rs`. It reads validated message-class evidence, maps the readpst item families, applies an immutable default visibility/type policy, and emits deterministic routing statuses in `data/items.jsonl`. The evidence includes a mixed synthetic class corpus, unsupported/missing/unknown classes, associated/deleted/filter decisions, and byte-identical repeated serialization. RP-M4-03 now adds the canonical non-mail evidence and Partial vJournal boundary; CLI flag breadth and broad real-PST mixed-folder evidence remain dependent work.

## Phase 1 — Unicode message breadth

Broaden the existing strongest path before adding legacy formats:

- complete sender/representing/received-by and date fields;
- message flags, importance, priority, sensitivity, and report controls;
- valid stored-header normalization and forensic fields;
- String8/Unicode/code-page conversion with `-C` and `-8` policies;
- generic HTML, RTF, report, and body-only cases;
- by-value, inline/CID, multiple, zero-length, size-mismatch, and non-ASCII attachments;
- nested embedded messages with recursion guards.

RP-M2-01 is the first metadata slice in this phase. It is integrated into the
canonical message JSONL path and preserves raw property provenance, but it does not
close the broader corpus, header, body, attachment-method, or output-adapter gates.

Exit criterion: a multi-producer Unicode corpus reaches E3 evidence for messages, bodies, headers, recipients, attachments, and embedded children.

## Phase 2 — non-mail and mixed folders

Implement the typed outputs that readpst already exposes:

- vCard and simple contact-list output;
- appointment/event iCalendar output;
- vJournal output;
- schedule/meeting MIME parts;
- delivery/disposition reports;
- mixed-folder routing and `-t` filtering;
- deleted/associated item policy and explicit skip records.

Exit criterion: the pinned `java-libpst-dist-list.pst` and a mixed synthetic fixture produce reconciled typed counts, with contacts, appointments, recurrence, and unsupported classes visible in the structured output.

RP-M4-01 now establishes the contact projection and vCard/contact-list adapter boundary.
The exit criterion is not promoted: the pinned java-libpst corpus remains explicit
negative/partial evidence until contact-class ownership and full MAPI field groups are
validated by PSTD.

RP-M4-02 now establishes the appointment `CalendarRecord` and deterministic
`icalendar` adapter boundary. The slice remains Partial: source-backed identity,
class, subject, and organizer values are preserved, while dates, timezone, recurrence,
exceptions, alarms, and categories remain explicit unavailable statuses until an
admitted appointment-property corpus exists. A synthetic `DTSTAMP` is marked in the
output solely to keep the projected component deterministic and parseable.

RP-M4-03 now establishes the `NonMailRecord` boundary for journals, tasks, sticky
notes, unknown classes, and missing message classes. Journal records have a
deterministic Partial `vjournal` profile; unsupported classes remain canonical typed
evidence with separate readpst and PSTD statuses and are never promoted to ordinary
mail. Full journal MAPI field coverage and dedicated task/sticky renderers remain
open.

RP-M5-01 now establishes the first mailbox adapter family over canonical records:
default mbox, recursive mbox, MH/rfc822, extended `.eml`, and separate numbered
message files. The profiles preserve deterministic folder-local ordering, mboxrd versus
message-file boundaries, safe relative paths, output hashes, and explicit skipped or
unavailable decisions. The exit criterion remains Partial until the pinned readpst
differential corpus, separate binary attachment/KMail/Thunderbird adapters, and later
MSG gate are complete.

## Phase 3 — output adapters

Add adapters over validated canonical records:

1. default per-folder mbox and recursive mbox;
2. MH/rfc822 and extended `.eml` separate output;
3. separate binary attachment files and extension filtering;
4. KMail layout;
5. Thunderbird `.type` and `.size` sidecars;
6. contact/calendar/journal file profiles;
7. `.msg` only after its own OLE writer and round-trip gate.

Exit criterion: a semantic differential harness can run the same fixture through readpst and PSTD and compare item counts, folder paths, body/attachment bytes, roles, MIME parts, and typed non-mail values.

## Phase 4 — legacy input and hardening

Add the remaining input breadth and semantic promotion:

- compressible encryption;
- strong encryption;
- broader ANSI NDB v14/v15 item and output corpus;
- broader OST 2013 item and output corpus;
- large-file and sparse-file handling;
- corrupt, truncated, and malicious derivatives;
- bounded parallel execution with deterministic output.

Exit criterion: each supported input family has a real or controlled fixture, exact hashes, independent inventory, repeat-run equality, and no silent loss under malformed derivatives.

RP-M6-01 has met the controlled structural-input slice: ANSI v14/v15 and OST 2013
header/root/page layouts are selected by the production path, with positive, strong-crypt,
truncated, malformed, hash, and repeat-run evidence in the dedicated workflow. The
matrix remains Partial until semantic item and output coverage is demonstrated for those
families; #520 owns encryption and #521 owns broader adversarial/resource evidence.

## Fixture and differential strategy

Each capability needs a fixture record containing:

```text
source and pinned revision
license/redistribution basis
file size and SHA-256
header family and crypt method
independent folder/item inventory
readpst command and version
PSTD command/profile and version
canonical record counts and hashes
output adapter counts and hashes
known unsupported or ambiguous evidence
```

The differential comparator must compare semantics, not unstable filenames or boundary strings. It should verify:

- folder hierarchy and item class;
- message identity and ownership;
- sender/recipient role and address-type values;
- decoded body bytes and source encodings;
- attachment metadata, method, order, Content-ID, and payload hash;
- calendar/contact/journal fields;
- skipped/unavailable reasons;
- total source versus emitted/partial/skipped counts.

## Definition of done for a row

A parity row can move to Implemented only when:

1. the feature has a Rust-native implementation;
2. the canonical structured output can represent success and failure states;
3. the relevant output adapter exists when readpst exposes one;
4. at least one E2 fixture passes, and broad claims have E3 evidence;
5. a malformed or ambiguous derivative fails closed;
6. output is deterministic across repeated runs and worker counts;
7. the matrix, current-state docs, and changelog are updated;
8. the validation gate and fixture workflow pass.

## Non-negotiable parity properties

- No readpst capability is declared complete solely because a field exists in a Rust struct.
- No output format is implemented by renaming another format.
- No unsupported item is silently counted as a successfully extracted message.
- No native Exchange address is rewritten as SMTP without authoritative evidence.
- No child message or attachment is assigned to a parent from an ambiguous reference.
- No raw body or attachment bytes are discarded because one higher-level projection failed.
- No private PST is committed as a fixture; controlled evidence must meet the repository’s provenance and redistribution rules.

## Planned implementation — `RP-11`

### Issue dependency graph

The phases above become issue clusters with a strict dependency order:

```text
RP-00 evidence/status
   ├─ RP-02 input/parser ── RP-03 typed envelope ── RP-04 metadata
   │                                      ├─ RP-06 attachments
   │                                      └─ RP-08 non-mail types
   ├─ RP-13 differential harness
   └─ RP-01 profiles/scheduler
                 RP-04 + RP-05 + RP-06 + RP-07 + RP-08
                                   -> RP-09 output adapters
                                   -> RP-10 matrix promotion
                                   -> parity release gate
```

The graph prevents a common failure mode: implementing an adapter against an incomplete `MessageRecord`, then discovering that contacts, encrypted bodies, reference attachments, or skipped classes have nowhere to go. An issue may be split for review, but it may not bypass the typed evidence and negative-status prerequisites.

### Readpst logic-to-issue mapping

| Issue cluster | Upstream functions/helpers reviewed | PSTD implementation result |
|---|---|---|
| `RP-01` | `main`, `process`, `create_enter_dir`, `mk_*_dir`, `mk_*_file`, `close_*`, regression `dopst` | typed profile, deterministic scheduling, output adapters, diagnostics |
| `RP-02` | `pst_open`, `pst_reopen`, `pst_load_index`, `pst_load_extended_attributes`, `pst_getTopOfFolders`, `pst_parse_item`, `pst_attach_to_file`, `pst_default_charset`, `vbuf.c`, `timeconv.c` | family/crypto/index/property/charset evidence |
| `RP-03` | `process`, `pst_process`, descriptor-tree and item-type constants | folder graph and `ItemEnvelope` |
| `RP-04` | `valid_headers`, header helpers, `write_normal_email`, email fields in `libpst.h`, `msg.cpp` properties | provenance-aware metadata/header model |
| `RP-05` | `write_body_part`, `write_normal_email`, `write_schedule_part_data`, `pst_lzfu_decompress`, `libstrfunc.c` | body set and semantic MIME tree |
| `RP-06` | `acceptable_ext`, `write_separate_attachment`, `write_inline_attachment`, attachment table/ID2 logic | attachment resolver and evidence graph |
| `RP-07` | `write_embedded_message`, report/schedule branches, encrypted/RTF branches, `pst_convert_recurrence` | bounded child graph and special MIME records |
| `RP-08` | `write_vcard`, `write_journal`, `write_appointment`, `write_extra_categories` | typed non-mail records and serializers |
| `RP-09` | `msg.cpp`, output directory/stream functions, regression profiles | interoperable output adapters |
| `RP-10`/`RP-13` | `regression-tests.bash`, `NEWS`, `ChangeLog`, all observable writers | semantic comparator, issue/matrix promotion |

### Per-issue implementation checklist

Every issue created from this register should follow this sequence:

1. Copy the issue template from [RP-13](13-issue-template-and-differential-harness.md).
2. Name the exact readpst function/helper, line anchor, and pinned revision.
3. Identify the canonical record, parser/output module, and documentation fan-out.
4. Add a positive fixture manifest and a malformed, ambiguous, or unsupported fixture.
5. Implement the smallest Rust-native vertical slice, retaining raw evidence and status.
6. Run unit, fixture, semantic differential, determinism, and resource-limit tests.
7. Update the matrix status and evidence level only after all gates pass.
8. Review every linked topic page recursively for changed assumptions, names, dependencies, and acceptance criteria.

### Release promotion gates

Before declaring “readpst parity” complete, the release issue must show:

- no applicable `Gap` or `Partial` matrix rows;
- an explicit `PSTD equivalent` or `unsupported-by-readpst` result for every capability;
- E4 semantic differential evidence for every output family and all supported input families;
- an approved corpus covering the upstream regression categories plus ANSI, OST, encrypted, malformed, mixed-folder, reference-attachment, nested, and large-file cases;
- identical canonical output across repeat runs and supported worker counts;
- no unbounded recursion, allocation, path escape, or diagnostic amplification in adversarial fixtures;
- synchronized README, topic pages, matrix, source ledger, current-state docs, and changelog.

The project may ship intermediate milestones, but their release notes must name the remaining plan IDs and must not claim full readpst capability coverage while the matrix contains unresolved rows.

## Agent execution plan

The phase outline above is operationalized by [the readpst parity agent orchestrator](14-agent-orchestrator.md). It groups the work into eight GitHub milestones (`RP-M0` through `RP-M7`) and 28 stable work-unit issues. Each issue is a vertical slice with a readpst source anchor, PSTD module boundary, positive/negative evidence, dependency links, a specialist role, and a recursive documentation fan-out. The global orchestrator must dispatch only ready units, serialize shared contract changes, and re-evaluate the graph after each green squash merge.

The exact work-unit register, agent roles, dispatch packet, issue state machine, and release checklist live in [14-agent-orchestrator.md](14-agent-orchestrator.md). This page remains the acceptance authority; the orchestrator is the execution authority.
