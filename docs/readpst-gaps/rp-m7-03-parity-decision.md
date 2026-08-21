# RP-M7-03 final readpst parity decision

Decision date: 21 August 2026. Reviewed main:
`83fdd051911410a8483e734d5890247d9b56bd65`. Pinned upstream:
`cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89`.

## Decision: NOT PARITY-COMPLETE

PSTD must not be released or described as behaviorally parity-complete with readpst at
this baseline. The decision is evidence-based, not a test escape:

- RP-M7-01 reviewed 75 matrix rows and retained 2 Implemented, 54 Partial, and 19 Gap.
- RP-M7-02 ran the pinned readpst/PSTD differential on the approved Unicode fixture;
  run `32512518536` passed 18 harness tests and uploaded artifact `9457584897`.
- The executable E4 controls are green, but release-wide E4 is `not_proven` because
  the remaining input families and output profiles lack admissible semantic corpora.
- CI, the differential workflow, hardening workflow, adapter workflows, and the
  dedicated E4 workflow are green at the reviewed implementation heads; green CI does
  not override the matrix’s explicit scope and evidence levels.

This is a final compatibility decision, not an assertion that the implementation work
is complete. The global issue may close because the release gate has an auditable result;
the 73 non-Implemented rows remain the roadmap for future parity work.

## Remaining matrix rows

### Partial — 54 rows

`CLI-01`, `CLI-03`, `CLI-04`, `CLI-05`, `CLI-07`, `CLI-11`, `CLI-12`, `CLI-13`,
`CLI-14`, `IN-01`, `IN-02`, `IN-03`, `IN-04`, `IN-05`, `IN-06`, `IN-07`, `IN-08`,
`IN-09`, `IN-10`, `ITEM-01`, `ITEM-02`, `ITEM-03`, `ITEM-05`, `ITEM-07`, `ITEM-08`,
`ITEM-09`, `ITEM-10`, `MSG-01`, `MSG-02`, `MSG-03`, `MSG-06`, `MSG-07`, `MSG-08`,
`BODY-01`, `BODY-02`, `BODY-03`, `BODY-04`, `BODY-07`, `ATT-01`, `ATT-02`,
`ATT-03`, `ATT-05`, `ATT-07`, `ATT-11`, `OUT-02`, `OUT-03`, `OUT-04`, `OUT-05`,
`OUT-06`, `OUT-07`, `OUT-08`, `OUT-09`, `OUT-10`, `OUT-11`.

These rows have production contracts and selected positive/negative evidence, but lack
the breadth, producer coverage, semantic differential, import compatibility, or full
field/property evidence required for an Implemented claim.

### Gap — 19 rows

`CLI-06`, `CLI-08`, `CLI-09`, `CLI-10`, `ITEM-04`, `ITEM-06`, `ITEM-11`, `MSG-04`,
`MSG-05`, `MSG-09`, `MSG-10`, `BODY-05`, `BODY-06`, `BODY-08`, `ATT-04`, `ATT-06`,
`ATT-08`, `ATT-09`, `ATT-10`.

These are readpst-exposed behaviors whose equivalent PSTD user-visible behavior is not
currently proven. They are not classified as `UnsupportedByReadpst`.

## Explicit stronger-equivalent improvements

PSTD intentionally improves safety and observability while preserving the default
compatibility boundary: source provenance and raw evidence are retained; malformed,
ambiguous, unsupported, unsafe, and resource-limited input receives explicit status;
archive paths are confined; TAR publication is close-then-rename; diagnostic output is
capped; recursive discovery is symlink-safe and bounded; worker results are restored to
source order; and repeated output is deterministic. These improvements are documented
and tested and do not silently promote a Partial or Gap row.

## Release limitations and next evidence

The next parity work must admit redistributable, provenance-labelled fixtures covering
the missing regression profiles and input families, then run independent semantic
comparisons for every applicable output adapter. Priority blockers are deleted/
associated filtering, type filters, fallback charset and RFC encodings, reports and
calendar MIME, reference/OLE attachments, flags/controls, attachment filtering/order,
broader ANSI/OST/encrypted item corpora, mixed-folder typed objects, and exact
Thunderbird/KMail/MSG interoperability.

No private PST bytes are admitted as a workaround. The pinned source ledger, matrix,
E4 report, issue packets, fixture hashes, negative outcomes, and deterministic workflow
artifacts are the authoritative evidence for this decision.
