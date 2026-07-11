# PQ52 — expose row-reference span evidence

## Evidence from PQ51

GitHub Actions run #428 passed the complete CI matrix. The public PST fixture retained 11 folders, one extracted message, two body payloads, zero attachments, one tar shard, and 29,806 output bytes. The table probe still resolved one 208-byte NID-backed row payload with four in-bounds references, but reported `tc_subnode_rows_variable_or_invalid_width`.

This disproves the prior working assumption that the payload contains four equal 52-byte rows beginning at offset zero. It does not prove that the references are invalid; it only proves that the fixed-width interpretation is unsupported.

## Revised requirement

PQ52 records the exact row-reference values and the derived consecutive spans in the existing bounded table diagnostic. It makes no attempt to reinterpret the references or decode row values.

## Changes

- Preserve the exact row-reference sequence in `TcSubnodeRowResolutionReport`.
- Preserve derived spans when references are unique, increasing, and in bounds.
- Publish both sequences in the existing table diagnostic status.
- Keep missing, ambiguous, and out-of-bounds behavior unchanged.
- Add regression coverage for exact references and equal spans.

## Safety boundary

This PQ is diagnostic only. It does not treat references as offsets, row IDs, or indices beyond the already tested bounds and ordering checks. It does not decode the presence bitmap, columns, or property values.

## Evidence required from CI

The public fixture must expose the four exact row-reference values and their derived spans while preserving all PQ51 extraction totals. These values will determine whether the next PQ should investigate a leading header region, variable row records, or a non-offset interpretation.

## Proposed PQ53 decision boundary

- If references are increasing with a consistent span after a non-zero first reference, investigate a bounded leading row-data header.
- If spans vary, inspect row-index key/value semantics and any row-size source before reading bytes.
- If references are non-monotonic or repeated, stop treating them as direct byte offsets and trace the BTH row-index value representation.
