# Active implementation checkpoint

- Active issue: #638 [ROBUST-08] Make the deterministic ANSI fixtures spec-valid: leaf BIDs and heap Property Contexts (follow-up to #633 / PR #637).
- Branch: `robust-08-ansi-fixtures` (from `main` at 6537aae). PR: not yet opened.
- Latest durable commit: a2b7779 (stage B spec-valid; recipient restored).

## Scope

- `tools/ansi_fixture.rs`: leaf BIDs for every data block; SLBLOCK keeps an internal BID; folder and message nodes become heap-on-node Property Contexts (`bClientSig` 0xbc, BTH leaf sorted by property ID); the HN header is the full 12 bytes, so the first allocation starts at 0x0c.
- `scripts/validate_ansi_stage_b.py`, `scripts/validate_ansi_attachment.py`: read the folder and message as heap Property Contexts and expect the new BIDs.
- `.github/workflows/ansi-stage-b.yml`: restore the single `to` recipient from the recipient table.
- `docs/fixtures/ansi-pst-generation.md`: describe the layout.

Out of scope (separate spec gaps, not named by #638): 16-byte ANSI NBT entries with `nidParent`, ANSI block trailers and 64-byte alignment, BBT `cRef`, and folder hierarchy/contents/associated tables.

## Established conclusions

- `node_payload::is_data_tree_root` needs an internal BID **and** an XBLOCK/XXBLOCK header, so leaf BIDs keep the single-block path either way.
- `HeapOnNode` reads allocations from the page map, so a 12-byte HN header is compatible.
- Heap PC inline types in the parser: 0x0002, 0x0003, 0x0004, 0x000a, 0x000b, plus 0x000d (HNID); everything else is a heap allocation.

- Further spec deviations found and fixed in the emitter (needed to restore the stage-B recipient): recipient table was at subnode NID 0x32 (reader uses fixed `NID_RECIPIENT_TABLE` 0x692); SLBLOCK used the Unicode layout (8-byte header, 24-byte entries) inside an ANSI file; recipient-row CEB was LSB-first (`0x07`) but MS-PST and `tc_reader` are MSB-first (`0xE0`).
- **Blocker for stage C:** PSTD attachment discovery reads only Unicode-width SLBLOCKs (`subnodes::unicode_subnode_entries`, `is_unicode_slblock` in `subnodes.rs`, `tc_subnode_rows.rs`, `tc_row_payload_candidates.rs`; used by `attachment_property_context.rs`). With a spec-valid ANSI SLBLOCK the stage-C attachment is not found (0 payloads). Probe with the old Unicode-width SLBLOCK: direct/indirect/external_id2 all extract the attachment. So existing ANSI attachment evidence rests on the non-spec layout. Stage C also stores the attachment PC at subnode 0x31 with no attachment table (0x671).

## Tests

- Local runner `scratchpad/run_workflow.py` (runs workflow `run:` steps against `target/debug`, skipping cargo fmt/test/clippy and external readers).
- ansi-stage-b.yml: all steps OK locally at a2b7779, recipient `to Recipient <recipient@example.test>` restored.
- validate_ansi_attachment.py (uncommitted, expects ANSI SLBLOCK): passes all variants; ansi-attachment.yml extraction step FAILS (attachment not discovered) — see blocker.

## Next exact change

Awaiting user scope decision: (a) keep stage C on a documented Unicode-width SLBLOCK and file a follow-up for ANSI SLBLOCK support in attachment discovery, or (b) make the subnode/attachment pipeline width-aware in this PR.
