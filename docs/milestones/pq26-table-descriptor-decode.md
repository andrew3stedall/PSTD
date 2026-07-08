# PQ26 Table Descriptor Decode

PQ26 implements issue #348.

## Expected decision

- Valid extents with unknown values should lead to table descriptor tag-source decoding.
- Omitted extents should lead to offset/width decoding.
- Valid extents without unknown values should allow materialization.
