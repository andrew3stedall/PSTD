# Active implementation checkpoint

- Active issue: #638 [ROBUST-08] Make the deterministic ANSI fixtures spec-valid: leaf BIDs and heap Property Contexts (follow-up to #633 / PR #637).
- Branch: `robust-08-ansi-fixtures` (from `main` at 6537aae). PR: not yet opened.
- Latest durable commit: none yet.

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

## Tests

- None run yet.

## Next exact change

Edit `make_property_context_heap` / `make_heap` in `tools/ansi_fixture.rs`, then replace the flat folder/message payloads and renumber BIDs.
