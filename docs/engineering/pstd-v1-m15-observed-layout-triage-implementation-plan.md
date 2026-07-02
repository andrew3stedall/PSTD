# PSTD v1 M15 Implementation Plan

## Implementation intent

M15 introduces compatibility triage rather than broad parser expansion. The goal is to make future fixture-driven work repeatable: observed layouts should be classified as supported, partial, or needing parser work with a clear next action.

## Current foundation

- M13 records attachment table parse errors, offsets, reasons, and parsed table statuses.
- M14 classifies subnode layouts and performs bounded recursive child loading for known child-reference layouts.
- M14 keeps unsupported layouts explicit.

## Implemented M15 slice

1. Compatibility triage module:
   - Adds `LayoutCompatibilityCase`.
   - Adds `ObservedLayoutTriageReport`.
   - Adds `triage_observed_attachment_layouts`.
2. Triage categories:
   - `table_context_layout`
   - `known_child_reference_layout`
   - `unsupported_subnode_layout`
   - `unparseable_attachment_table`
   - `attachment_rows_without_payloads`
3. Triage statuses:
   - `observed_layouts_empty`
   - `observed_layouts_supported`
   - `observed_layouts_need_parser_triage`
   - `observed_layouts_need_payload_triage`
4. Synthetic tests:
   - Supported layouts.
   - Unsupported layouts and parse errors.
   - Missing payloads.
   - Empty reports.

## Safe fixture workflow

When a public or sanitized fixture is available:

1. Run inspect/extract using the existing validation commands.
2. Preserve structured statuses, offsets, and parse reasons.
3. Create a focused compatibility issue for each unsupported/partial category.
4. Add the smallest possible synthetic test before expanding parser logic.
5. Only add real fixture files when licensing, size, and privacy rules are satisfied.

## Remaining work

- Wire triage summaries into machine-readable run output if needed.
- Add fixture-backed compatibility tests once suitable samples are available.
- Add decoders only for layouts confirmed by focused fixtures and tests.

## Definition of done

- M15 branch keeps CI green.
- Compatibility triage is covered by unit tests.
- Docs explain how to use public/sanitized fixtures safely.
- Project status and changelog document M15 and remaining limitations.
