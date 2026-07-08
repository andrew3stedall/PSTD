# PQ25 Table Tag Interpretation

PQ25 implements issue #346.

## Expected decision

- Byte-swapped selected values should lead to property materialization using corrected tags.
- Byte-swapped plausible values should lead to dictionary expansion.
- Known type-code word signals should lead to tag word-order decoding.
- No signal should lead to descriptor-layout decoding.
