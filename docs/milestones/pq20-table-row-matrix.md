# PQ20 Table Row Matrix Measurement

PQ20 starts the new repeatable PQ cycle SOP.

## Scope

- Surface row-matrix measurement counters in the public PST progress artifact.
- Keep extraction output unchanged until row data is decoded and safely mapped.
- Use the artifact to revise PQ21 requirements.

## Expected result

The likely result is measurement visibility without extraction-count lift. If the artifact reports table parse success but zero parsed rows, PQ21 should wire parser-level table row/column counts into run status or decode the row matrix layout more deeply.

## Next blocker decision

- Rows and columns present: move to row-to-property candidate mapping.
- Table success but rows absent: fix table parser counter propagation or row matrix layout decoding.
- No table success: return to table context source selection.
