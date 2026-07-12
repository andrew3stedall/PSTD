# PQ63: publish validated TCINFO descriptor evidence

## Baseline

PQ57 exposed four exact 14-bit row masks from the public PST fixture. PQ58 validated that TCINFO bitmap indices are unique, in range, and complete. PQ59 and PQ60 added a bounded evidence builder and deterministic delimiter-safe formatter. PQ61 transported complete validated column descriptors through heap resolution, and PQ62 added a builder entry point for those transported descriptors.

The remaining gap was reporting: the real descriptor-to-bitmap mapping was not emitted by `TcHeapDiagnostic`.

## Revised requirement

PQ63 integrates the existing builder and formatter into the reporting path only. It does not access row value bytes.

For each resolved table heap, reporting now emits:

- `descriptor_evidence`: the complete PQ60-formatted descriptor record set, or `none`;
- `descriptor_evidence_status=tc_descriptor_evidence_validated` when complete evidence is built;
- `descriptor_evidence_status=tc_descriptor_evidence_unavailable` when descriptors or exact masks are unavailable;
- `descriptor_evidence_status=tc_descriptor_evidence_construction_failed` when validation fails.

Construction failure emits no partial descriptor records.

## Evidence contract

Each formatted record contains:

- bitmap index;
- original descriptor order;
- raw property tag;
- derived property type;
- data offset;
- data size;
- one raw state per validated row mask.

Records remain ordered by bitmap index and separated with `~`, preserving compatibility with the existing comma-, colon-, semicolon-, and pipe-delimited diagnostic.

## Safety boundary

PQ63 does not:

- read fixed-width property values;
- follow HID or HNID references;
- interpret a set bitmap bit as semantic property presence;
- change message, body, or attachment extraction.

The least-significant-bit-first mask convention remains structural evidence requiring broader fixture validation.

## Required validation

The final CI run must pass the complete Rust, Python, Docker, CLI, and public-PST workflow. The public fixture should retain the established extraction totals and expose 14 descriptor records with a validated evidence status.

## Proposed PQ64

After fixture evidence confirms the real 14-record mapping, PQ64 should select one bounded fixed-width descriptor whose offset and size fit wholly within the validated fixed-data region. It should report raw hexadecimal bytes for each row only when the corresponding raw bit is set. It must not interpret the bytes or generalize to variable-width properties.
