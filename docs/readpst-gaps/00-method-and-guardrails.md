# Method and parity guardrails

## Purpose

This register answers a narrow question: what observable behaviour must PSTD add or broaden so that a user can replace `readpst` without losing a capability that `readpst` currently provides?

The comparison includes the `readpst` executable and the libpst item/parser surface it uses. It does not silently expand the target to every utility in the repository. `lspst`, `pst2ldif`, `nick2ldif`, and `pst2dii` are recorded only where they expose an item or parser behaviour that readpst also relies on. Their utility-specific output formats are separate future products, not accidental readpst requirements.

## Evidence sources

The inventory was derived from:

| Source | What it establishes |
|---|---|
| [`src/readpst.c`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/readpst.c) | CLI flags, traversal, output modes, MIME generation, attachment handling, vCard, vJournal, and vCalendar output. |
| [`src/libpst.h`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.h) | Item classes, attachment methods, encryption modes, flags, email/contact/appointment/journal fields, and recurrence structures. |
| [`src/libpst.c`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/src/libpst.c) | Index loading, decryption, charset conversion, MAPI projection, attachment resolution, item classification, and recurrence decoding. |
| [`NEWS`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/NEWS), [`ChangeLog`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/ChangeLog) | Historical behaviours that are easy to miss when reading only the current command path: OST 2013, Content-ID, mixed item types, RFC 2047/2231, reports, and embedded messages. |
| [`regression/regression-tests.bash`](https://github.com/pst-format/libpst/blob/cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89/regression/regression-tests.bash) | The upstream fixture categories used to exercise HTML, text, appointments, recurrence, embedded messages, non-ASCII headers, MIME-signed mail, and journal archives. |
| PSTD `README.md`, project status, output contract, and approved fixtures | The current PSTD implementation boundary and evidence already available in this repository. |

## What counts as parity

Parity is measured at three layers:

### 1. Input and interpretation

PSTD must open the same relevant PST/OST families, interpret the same encryption and encoding forms, traverse the same folder and item relationships, and resolve the same attachment references. Unsupported or corrupt input may fail closed, but the result must say why and identify the affected scope.

### 2. Information extraction

PSTD must preserve the information readpst uses: folder paths, item classes, message fields, headers, bodies, recipients, attachment metadata and bytes, content IDs, calendar properties, contacts, journals, flags, and recurrence data. A normalized structured record is preferred over losing a field for the sake of matching a legacy output format.

### 3. User-visible outputs

PSTD must offer an equivalent for each readpst output family: mbox, recursive folder output, MH/rfc822, separate files and attachments, KMail, Thunderbird metadata, EML, vCard, vCalendar, vJournal, and MSG. The canonical TAR/JSONL output remains valuable, but it is not by itself proof of output-mode parity.

## Evidence levels

| Level | Use |
|---|---|
| E0 | Source-only observation from upstream code; useful for registering a requirement. |
| E1 | PSTD unit or synthetic structure test; proves a local decoder or writer boundary. |
| E2 | Approved public fixture with deterministic PSTD output; proves one real layout. |
| E3 | Multiple independent producers or a controlled corpus; supports a compatibility claim. |
| E4 | Differential run against readpst plus exact semantic comparison and malformed-input tests; required for final parity claims. |

No row should be marked Implemented for broad compatibility on E0 or E1 alone. The default promotion target is E3; the final parity release requires E4 for every applicable output family.

## Fail-closed requirements

PSTD’s existing correctness rules remain in force:

- do not guess property meaning, address type, encoding, ownership, or row alignment;
- preserve raw values when a stronger interpretation is not justified;
- distinguish absent, empty, unavailable, unsupported, corrupt, and skipped;
- do not combine values from different parser executions into one record;
- bound reads, recursion, allocation, diagnostics, and output filenames;
- retain item and folder counts so skipped content is visible;
- make repeated runs deterministic, including ordering, IDs, hashes, and error statuses.

These rules are a required part of parity. A converter that emits more files by silently inventing values is not a successful replacement for readpst.

## Licensing and implementation boundary

The libpst project is GPL-licensed. PSTD must not add libpst as a required dependency or copy its implementation. The comparison should inform Rust-native code, tests, and fixture design. Any external run used as an oracle must be pinned, isolated, and reported as supporting evidence; PSTD’s own parser and exact output remain authoritative for the project.

## Planned implementation — `RP-00`

### Readpst logic reviewed

The implementation baseline is the complete `readpst` execution path, not only its visible command-line options. `main` opens and indexes the file before changing directory, loads extended attributes, resolves the message-store root, and then calls `process`. `process` mixes folder recursion, item classification, output stream selection, attachment extraction, and diagnostics. The email writer reconstructs MIME from mutable `pst_item` fields; contact, journal, and appointment writers project the same item into vCard/iCalendar/vJournal. `libpst.c` supplies the parser, attachment ID2 resolution, charset conversion, and recurrence decoding. `msg.cpp` is a separate OLE writer used by `-m`. The helper path also includes LZFU RTF decompression, iconv-backed conversions, FILETIME conversion, and base64 encoding.

The source ledger in [Upstream source notes](12-upstream-source-notes.md) is the review record. No parity issue may cite “readpst supports X” without naming the function or helper that produces the observation.

### Rust-native design

Add a shared evidence contract rather than reproducing readpst’s global mutable state:

```rust
struct EvidenceEnvelope<T> {
    source: SourceIdentity,
    item: ItemIdentity,
    visibility: ItemVisibility,
    kind: ItemKind,
    value: Option<T>,
    raw: Vec<RawEvidenceRef>,
    status: ExtractionStatus,
    warnings: Vec<DiagnosticRef>,
}
```

The envelope is consumed by the current `src/engine/metadata.rs` and `src/output/metadata.rs` path, while specialized modules are added under `src/pst/` and `src/output/` only when a boundary is proven. `MessageRecord` remains compatible, but becomes one typed projection among `ContactRecord`, `AppointmentRecord`, `JournalRecord`, `ReportRecord`, and `ItemEnvelope`. `AttachmentRecord` remains the canonical attachment index; raw payloads remain content-addressed evidence.

### RP-M0-01 evidence-contract implementation

The shared parity contract is implemented in the test/support boundary at `tests/readpst_diff/manifest.rs` and exercised by `tests/readpst_diff_contract.rs`. It separates matrix `ParityStatus` (`implemented`, `partial`, `gap`, `unsupported_by_readpst`, `filtered`, `unavailable`, `failed`) from observed evidence statuses (`present`, `empty`, `skipped`, `filtered`, `unavailable`, `unsupported`, `ambiguous`, `malformed`, `corrupt`, `failed`). `FixtureManifest` records provenance/license, source revision/path, a safe local path, SHA-256, byte size, input family, crypt method, expected category/status, and admission state. `ComparisonRun` and `EvidenceReport` retain the pinned readpst revision, tool commands/versions, output profile, worker count, outcomes, inventory, artifacts, and deterministic-repeat evidence. The approved Tika Unicode fixture is represented as E2/Partial evidence; this does not promote any matrix row or claim readpst parity.

### RP-M0-03 source-drift implementation

`tests/readpst_diff/source_manifest.rs` is the pinned upstream-audit boundary. It covers the complete direct ledger, selected function/helper anchors, all 28 work-unit mappings, and the regression profile categories without importing or copying GPL implementation code. The check fails closed on revision drift, missing or duplicate source paths, unresolved symbols, changed repository/license boundary, and out-of-range work-unit mappings. Its report is stable JSON and is independent of private PST payloads or network access.

### RP-M1-01 delivery

The input boundary now emits a typed `InputCapability` before extraction: Unicode PST, ANSI PST, OST 2013, and unknown families; index type/version; crypt method; root readiness; the documented ISO-8859-1 fallback; index/extended-attribute readiness; and explicit unsupported, malformed, partial, unavailable, or budget-exceeded status. File size, single-read, candidate, property, diagnostic, and recursion budgets are serialized with the capability. Header classification never claims extraction support by itself.

### RP-M1-03 delivery

The canonical item stream now applies a source-class routing policy after ownership and visibility are known. `IPM.Note`, schedule, appointment, contact, journal/activity, report, task, sticky-note, and unknown message classes are classified without using display names or guessing missing properties. Each item carries a deterministic `routing_status` such as `routed_contact`, `filtered_associated`, `filtered_deleted`, `filtered_item_type`, `skipped_unsupported_by_readpst`, or `unavailable_missing_item_class`; the default policy excludes associated/deleted content while retaining the item envelope and its evidence. This is a production routing boundary, not a claim that the CLI flags or typed output adapters are complete.
### Implementation algorithm

1. Pin the upstream revision and record the source function, line anchor, and observable behaviour in a plan issue.
2. Define the canonical record and status before writing a decoder or adapter. Required statuses include `present`, `empty`, `unavailable`, `unsupported`, `filtered`, `ambiguous`, `corrupt`, and `failed`.
3. Parse once with bounded `ParserLimits`; assign stable source identities and provenance to every folder, item, property, body, attachment, and child edge.
4. Build typed projections only from validated evidence. Keep undecodable raw bytes and the reason for refusing a higher-level interpretation.
5. Project into output profiles through pure adapters. An adapter may omit content only when its policy says so and the manifest records the omission.
6. Run the semantic comparator against a pinned readpst invocation and a parser-validating reader. Compare decoded values, item classes, ownership, payload hashes, MIME trees, and status reasons rather than filenames or boundary strings.
7. Reconcile source, canonical, and adapter counts. Update the matrix and all tangential pages before the issue is closed.

### Improvements over readpst

- Replace process-wide globals and `chdir` with immutable run configuration and explicit output roots.
- Replace `fprintf`-only diagnostics and empty-file deletion with machine-readable per-item status and retained raw evidence.
- Replace `check_filename`’s `/\\:` replacement with platform-independent path confinement, reserved-name handling, and collision-proof stable paths.
- Bound embedded-message depth, graph expansion, attachment size, decompression output, and diagnostics; detect cycles before recursion.
- Keep native Exchange addresses and original encodings instead of flattening them prematurely into display strings.
- Make worker scheduling deterministic and safe for shared Rust readers; readpst’s fork/reopen strategy is a throughput reference, not a memory-safety requirement.
- Implement standards-correct MIME and calendar output with explicit loss reporting instead of relying on legacy formatting quirks.

### Issue and acceptance contract

Every `RP-*` issue must contain:

```text
Capability and user-visible replacement behaviour
Readpst source functions and pinned revision
PSTD modules/records to change
Positive fixture and malformed/ambiguous fixture
Canonical record and output-adapter assertions
Determinism and worker-count assertions
Documentation fan-out: README, topic page, matrix, roadmap, source ledger, changelog
```

`RP-00` is complete only when the status enum, provenance shape, source ledger format, differential comparison schema, fixture manifest fields, and documentation fan-out checklist are accepted by the repository. It is a prerequisite for promoting any other row.

See [Issue template and differential harness](13-issue-template-and-differential-harness.md) for the concrete issue body and comparator contract.
