# Active Implementation Checkpoint

Updated: 2026-09-08

## Active delivery

- Issue: #600 — `[COVERAGE-01] Implement generic HNID/HID/NID property-value resolution`
- Parent epic: #599
- Branch: `agent/coverage-foundation`
- PR: #611
- Base: `7f5c197c1443dacba1f29c61c2f527a082f30e38`
- Latest durable implementation commit: none — no Rust implementation is committed yet
- Latest branch commit: `d9ff1faf2ebb33994d1550d6042fcb76fb103a97` — temporary build-tools workflow only
- Validation state: unverified implementation (implementation not started)

## Scope

Implement one reusable property-value reference boundary for issue #600. It must preserve property identity/reference provenance and distinguish inline bytes, heap/HID values, NID/subnode-backed values, data-tree payloads, and unresolved/fail-closed outcomes. The first delivery remains #600 only; #601-#603 consume the API later.

### Out of scope

- Generic MAPI datatype expansion (#601).
- Name-to-ID/named-property resolution (#602).
- Standards-compliant Table Context replacement (#603).
- Calendar/address/body/long-tail semantic work from later #599 children.
- Epic-wide differential completion (#610).

## Established conclusions — do not re-derive

- `src/pst/bth.rs::property_context_entry` already interprets a Property Context BTH leaf as `(prop_id, prop_type, value_hnid)`. For non-`PTYP_OBJECT` values it tries `HeapOnNode::try_allocation_by_hnid`; when that fails it currently collapses the unresolved reference back to the raw four-byte HNID. That loss of storage/provenance identity is the key generic #600 boundary.
- `src/pst/heap.rs::allocation_by_hid` / `try_allocation_by_hnid` are the existing bounded heap/HID primitives. Reuse them; do not implement a second heap allocator.
- `src/pst/tcinfo.rs::classify_hnid` already provides the repository's `HeapId` / `NodeId` / `Null` union classification. Reuse or centralize this semantics rather than inventing incompatible HNID classification.
- `src/pst/attachment_property_context.rs::resolve_attachment_payload`, `inline_object_heap_bytes`, and `resolved_subnode_data_reference` demonstrate existing special-case HID/NID/data-tree resolution. #600 should extract/reuse the generic mechanics rather than leave attachment-only copies as the architecture.
- `PropertyContext::from_bth_with_fallback_charset` currently receives only the flattened `BthEntry.value`; it therefore cannot distinguish a genuine four-byte inline value from an unresolved four-byte HNID after `property_context_entry` has discarded provenance.
- The branch contains no coverage implementation yet. Do not claim typed generic property resolution is implemented until Rust code/tests are committed.
- The temporary `.github/workflows/coverage-build-tools.yml` is fallback infrastructure only. It must not delay the first source checkpoint and must be removed before #611 is merged.
- Do not reread #599, #601-#610, the full project-status/progress history, PQ corpus, or `docs/readpst-gaps/` corpus during normal continuation. Fetch a specific historical section only if a concrete #600 code decision requires it.

## Current source boundary

- `src/pst/bth.rs` — `property_context_entry` — currently resolves HID opportunistically and otherwise loses HNID provenance.
- `src/pst/heap.rs` — `allocation_by_hid`, `try_allocation_by_hnid` — existing bounded heap resolution to reuse.
- `src/pst/tcinfo.rs` — `HnidKind`, `classify_hnid` — existing HNID union classification.
- `src/pst/property_context.rs` — `PropertyValue`, `from_bth_with_fallback_charset` — destination that needs typed source/provenance before semantic MAPI decoding.
- `src/pst/attachment_property_context.rs` — `resolve_attachment_payload`, `inline_object_heap_bytes`, `resolved_subnode_data_reference` — special-case consumers to migrate only after the generic resolver exists and focused tests are green.
- `src/pst/mod.rs` — register the new resolver module if a dedicated module is introduced.

## Durable progress

- `7f5c197c1443dacba1f29c61c2f527a082f30e38` on `main` — checkpoint-first continuation operating model is merged.
- `d9ff1faf2ebb33994d1550d6042fcb76fb103a97` — preserved temporary public Rust build-tools workflow after rebasing #611 onto the new operating model.

## Focused validation

- NOT RUN: #600 Rust focused tests — no #600 Rust implementation exists yet.
- Merge-only full validation must not be run as a substitute for the first source increment.

## Blockers / unresolved evidence

- Need to preserve the original PC `prop_type` + `value_hnid` + resolution status through BTH parsing without breaking existing `BthMap` consumers.
- NID/subnode lookup needs a generic source context (owner/subnode tree/reader/BBT/limits); do not fake this by searching unrelated payloads globally or by treating every four-byte value as a reference.

## Next exact change

Create the first bounded #600 Rust increment around `src/pst/bth.rs::property_context_entry`: introduce a typed property-reference/source record that preserves `prop_id`, `prop_type`, raw `value_hnid`, HNID kind, and the result of the existing heap/HID lookup instead of collapsing an unresolved reference to indistinguishable four-byte bytes. Expose that typed evidence through `BthMap` (or a dedicated `src/pst/property_value_resolution.rs` boundary registered in `src/pst/mod.rs`) while keeping the existing flattened `BthEntry` API compatible for current callers. Add focused unit tests proving: (1) inline/fixed values remain byte-identical, (2) valid HID values resolve byte-identically, and (3) an unresolved NodeId remains explicitly classified with its original HNID rather than being silently treated as decoded raw data. Commit this type + HID/NodeId provenance increment before attempting generic NID/data-tree loading or attachment migration.

## Merge-only work remaining

- implement generic NID/subnode and applicable data-tree resolution with cycle/resource-limit fail-closed statuses;
- wire Property Context and then safe attachment/body/recipient consumers to the generic resolver;
- add focused inline/HID/NID/data-tree/invalid-reference regressions;
- update #600 diagnostics/docs once observable behavior changes;
- remove `.github/workflows/coverage-build-tools.yml`;
- run the full repository validation gate on the cleaned exact head;
- inspect relevant approved fixture workflows/artifacts and record the delta;
- inspect final diff/review threads and merge #611 only when #600 acceptance is actually met.
