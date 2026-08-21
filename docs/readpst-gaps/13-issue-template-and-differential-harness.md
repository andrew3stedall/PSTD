# Issue template and semantic differential harness

This document turns the `RP-00`–`RP-12` plans into GitHub-ready work and defines the evidence needed before a row can move from Partial or Gap to Implemented. It is intentionally written as a technical template rather than a promise that the harness or output adapters already exist.

## Issue template

Copy the following body into an issue and replace every bracketed value:

```markdown
## Workboard

- Work unit: `RP-Mx-yy`
- GitHub milestone: `RP-Mx`
- Specialist role: [role]
- State: [PLANNED | READY | IN PROGRESS | BLOCKED | ...]
- Depends on: [issue links or `—`]
- Blocks: [issue links or `—`]
- Allowed module boundary: [paths]

## Capability

- Plan ID: `RP-[nn]`
- Matrix rows: `[CLI-xx, ...]`
- User-visible replacement behaviour: [one observable sentence]
- Current status: [Gap | Partial]

## Readpst evidence

- Repository: `pst-format/libpst`
- Revision: `cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89`
- Source functions/helpers: [names and pinned line links]
- Observable algorithm: [ordered steps, including skip/failure behaviour]
- Regression categories: [fixture/script references]

## PSTD design

- Parser modules: [src/pst/...]
- Engine modules: [src/engine/...]
- Records/statuses: [canonical structs/enums]
- Output profiles: [canonical | mbox | eml | ...]
- Raw evidence retained: [properties/payloads/graph edges]
- Limits and concurrency: [budgets and worker policy]

## Implementation plan

1. [decoder/model step]
2. [projection/adapter step]
3. [diagnostic/status step]
4. [determinism/atomic output step]

## Evidence

- Positive fixtures: [IDs, provenance, SHA-256]
- Negative fixtures: [malformed/ambiguous/unsupported IDs]
- Readpst command/version: [exact invocation]
- PSTD command/profile: [exact invocation]
- Differential comparator: [command or test target]

## Acceptance

- [ ] canonical records preserve source identity and raw evidence
- [ ] success and every negative status have assertions
- [ ] semantic output matches readpst where applicable
- [ ] stronger PSTD behaviour is documented as an improvement
- [ ] repeat runs and worker counts are identical
- [ ] limits/path/atomic-write checks pass
- [ ] matrix, topic page, README, roadmap, source ledger, and changelog are updated
- [ ] agent dispatch/return packet and final issue state are recorded

## Documentation fan-out

- [ ] [list every linked page affected by this issue]
```

Do not close an issue with “tests pass” alone. The issue must identify the evidence level and the semantic boundary that the tests prove.

## Shared contract implementation

RP-M0-01 establishes the manifest and report types used by later differential work in `tests/readpst_diff/manifest.rs`, with the module exported from `tests/readpst_diff/mod.rs`. The contract captures source provenance, fixture admission, input family, crypt method, pinned upstream revision, tool executions, output profile, worker count, normalized outcomes, inventory counts, artifact digests, evidence level, parity status, and deterministic-repeat results. It also rejects unsafe fixture paths, malformed hashes, missing provenance, source-revision drift, and an Implemented claim below E4. The executable process runner and semantic comparator remain the bounded scope of RP-M0-02.

### RP-M0-02 delivery

The isolated differential slice is implemented in `tests/readpst_diff/{runner,normalize,compare,report}.rs` and exercised by `tests/readpst_diff_runner.rs`. It validates the approved Apache Tika Unicode fixture, runs bounded readpst/PSTD processes in separate roots, normalizes canonical TAR/JSONL and readpst output semantics, records explicit parity/extension/unsupported/failure findings, rejects unsafe paths and resource-limit violations, and compares repeated reports for determinism. The dedicated `readpst-differential.yml` workflow builds the pinned CLI revision with `--enable-python=no` (the optional binding is outside the oracle surface) and runs the configured differential test. Current evidence is E2/Partial; this harness does not promote a parity row without the downstream feature and corpus gates.

RP-M2-02 extends the canonical artifact set with `data/headers.jsonl`. Differential
comparisons may use `header_key`, `message_key`, `source`, `charset_policy`,
`validation_status`, `authoritative`, normalized header text, and the raw evidence
key; raw bytes are compared through the bounded evidence record rather than inferred
from filenames. Repeated extraction must preserve these fields byte-for-byte, while
malformed and lossy inputs remain explicit non-authoritative outcomes.

## Comparator contract

The harness should run a pinned readpst binary and PSTD against the same approved fixture, then normalize both results into a comparison document. Byte-for-byte output is useful for debugging but is not the primary contract because mbox boundaries, generated filenames, and MIME boundary tokens are implementation details.

### Normalized comparison schema

```text
ComparisonRun {
  tool, version, source_revision, command, fixture_sha256,
  input_family, crypt_method, charset_policy, output_profile,
  folders: [{ canonical_path, source_id, counts, status }],
  items: [{ source_id, folder_id, kind, visibility, status,
            subject, identities, dates, flags, body_hashes,
            attachment_keys, child_edges }],
  attachments: [{ key, method, name, mime, cid, sequence,
                  rendering_position, status, payload_hash }],
  mime_parts: [{ owner, path, media_type, params, disposition,
                 cid, decoded_hash, child_owner }],
  typed_outputs: [{ source_id, profile, normalized_fields, status }],
  diagnostics: [{ scope, code, severity, stable_message }],
}
```

Normalization rules:

- identify items by source node/descriptor identity when available, otherwise by a documented stable fallback; never use display name alone;
- normalize path separators, collision suffixes, and generated names separately from canonical folder/item identity;
- parse mbox into individual messages and remove only the mbox separator before comparing headers/body;
- parse MIME, vCard, iCalendar, vJournal, and MSG with independent readers, compare decoded values and payload hashes, and preserve raw output for diagnostics;
- compare header field names/values, address roles/types, body bytes, attachment ordering, CIDs, item class, and statuses;
- treat readpst’s current-time journal DTSTAMP and other synthetic values as synthetic compatibility fields;
- record intentional stronger PSTD behaviour as `pstd_extension` rather than failing the common parity comparison;
- never turn a missing readpst output file into a successful empty record.

## Differential execution stages

1. Verify fixture provenance, SHA-256, input limits, and the pinned readpst executable.
2. Run readpst in the smallest profile that exposes the capability, then run the equivalent PSTD profile with the same output root isolation and charset/deleted/filter policy.
3. Capture stdout, stderr, exit status, file inventory, raw output hashes, and tool resource counters.
4. Parse both outputs into the normalized schema. Capture canonical PSTD JSONL/TAR records separately; output adapters cannot hide canonical skips.
5. Compare folders, item classes, visibility, counts, identities, metadata, body bytes, attachment bytes, MIME trees, typed non-mail fields, and failure reasons.
6. Repeat with one worker and the maximum supported bounded worker count. Compare canonical records, paths, hashes, and statuses.
7. Run the malformed/ambiguous derivative and assert bounded failure, no path escape, no silent success, and retained raw evidence where available.
8. Store a compact comparison report and link it from the issue/fixture manifest. Do not commit private PST payloads or unreviewed tool output.

## Planned implementation — `RP-13`

### Harness modules

Add the harness in a test/support boundary rather than coupling the production parser to an external binary:

```text
tests/readpst_diff/manifest.rs       fixture and command metadata
tests/readpst_diff/runner.rs         isolated process execution and budgets
tests/readpst_diff/normalize.rs      MIME/mbox/vCard/ICS/MSG normalization
tests/readpst_diff/compare.rs        semantic equality and extension policy
tests/readpst_diff/report.rs         compact JSON/Markdown issue evidence
```

The production equivalent should expose a stable machine-readable inspection/export profile; the comparator should consume that contract and never scrape human diagnostics as the primary result. External readpst is an oracle only for the pinned fixture run; PSTD’s parser, raw evidence, and status model remain authoritative.

### Improvements over the upstream regression script

- replace shell `rm -rf output*` and broad glob cleanup with a unique temporary run root and validated explicit paths;
- replace output-file deletion and `grep -v iamunique` with typed normalization and a declared synthetic-value rule;
- replace positional fixture names with manifest IDs, license/provenance, hashes, family, crypt method, and expected categories;
- replace valgrind-only coverage with parser limits, sanitizer/fuzz derivatives, allocation budgets, and stable error codes;
- compare all output profiles and canonical records, not just whether files exist;
- preserve unavailable/skipped/filtered reasons instead of treating absent files as success;
- make readpst version/source drift visible before comparisons run.
- run the production batch boundary with one and the maximum allowed bounded worker
  count, normalize timestamps/run IDs, and compare sorted item status/output inventories;
  symlink, traversal, archive-confinement, and diagnostic-budget failures are separate
  negative outcomes rather than empty successes.

### Acceptance and maintenance

`RP-13` is complete when an issue can be created from this template, a fixture can be run through both tools, normalized semantic output can be compared, and the report can distinguish parity, intentional stronger behaviour, unsupported, and failure. Every implementation issue must use the harness or document why a local unit/synthetic test is the only applicable evidence. Changes to normalization rules must update [the matrix](10-parity-matrix.md), [the roadmap](11-roadmap-and-acceptance.md), and every topic page whose output semantics changed.

## Orchestrator handoff

Use the stable work-unit key and dependency graph in [the readpst parity agent orchestrator](14-agent-orchestrator.md). The global orchestrator owns readiness, specialist assignment, merge sequencing, and recursive documentation review. An issue may be closed only after its dispatch packet has a returned implementation/evidence packet and the linked matrix status is either unchanged for a documented reason or promoted with the required evidence level.
