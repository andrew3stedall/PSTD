# Active implementation checkpoint

Updated: 2026-09-10

## Active delivery

- Issue #600; parent #599; branch agent/coverage-foundation; draft PR #611.
- Last fully checked source: 5452630362c1705927dfbbc16638f71470e9b0ee. Main CI and focused workflow pass.
- Current increment: correct ANSI fixture inline scalars (5297610, a4138fb), preserve attachment reference failure detail, and temporary bounded Tika diagnostics.
- Do not use old local /workspace/scratch/10c9672c1630/pstd: separate uncommitted work.

## Established conclusions

- Typed BTH leaves preserve inline/HID/NID identity and provenance; unresolved values are excluded from body output and semantic decoding.
- Production node loader and binary attachment loader use Unicode owner-scoped resolver and data trees.
- Regressions cover NID/data-tree binary attachments, owner isolation/ambiguity, HTML/RTF output, missing body references, overlapping subnode ranges, external index BIDs, and root NID aliases.
- ANSI generic owner layout and nonzero heap pages remain unsupported.
- The ANSI fixture generator incorrectly put scalar method/size values in heap allocations; 5297610 fixes inline encoding, a4138fb updates independent validator. Never restore heuristic scalar dereferencing to accommodate this fixture.
- Existing CI permits clippy::too_many_arguments; AGENTS strict command differs. No blanket lint suppression introduced.

## Validation

- 371 library tests passed before the root-alias regression; 5452630 focused workflow and main CI passed including that regression.
- 5452630: Tika attachment (34347463481), embedded graph (34347463868), reconstructible content (34347463428), ANSI attachment (34347463452) FAILED actual fixture checks after lint/format success.
- Tika: DOCX attachment size 0 vs expected 11862, missing embedded payload path. Reconstructible content fails because DOCX bytes unavailable.
- ANSI: method=160 and declared_size=192 instead of scalar values; fixture encoding corrected, awaiting validation.
- Other fixture workflows passed on 5452630.
- Local Rust absent; direct GitHub unavailable. Connector artifact ZIP download produces a reference, but local URL fetch is 403. Use existing temporary workflow diagnostics, not environment bootstrap.

## Next exact change

1. Inspect latest Coverage temporary build tools workflow; step "Inspect approved Tika attachment diagnostics" prints bounded attachment resolution statuses for the approved fixture. Fix the concrete typed resolver/owner failure.
2. Check ANSI fixture workflow after scalar correction; preserve indirect method/object compatibility.
3. Confirm Tika, embedded graph and reconstructible content recover their approved bytes; do not weaken fixture expectations to pass.
4. Complete remaining #600 acceptance (generic HID pages/ANSI context and deterministic diagnostics), current docs, remove temporary workflow, then exact-head merge gates.

Temporary workflow persists formatting only for named Rust files. Add tools/ansi_fixture.rs if cargo fmt changes it. Never force-push over a newer head. Keep this checkpoint updated before broad work.
