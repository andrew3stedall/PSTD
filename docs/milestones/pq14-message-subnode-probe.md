# PQ14 Message Subnode Probe

## Goal

Use the PQ13 result to perform bounded message-level subnode probing for true message candidates.

## Scope

PQ14 is bounded to issues #311 through #315.

| Issue | Scope | Status |
|---:|---|---|
| #311 | Parent scope | Implemented in this PR |
| #312 | Bounded message subnode decode attempts | Implemented |
| #313 | Subnode layout probe counters | Implemented |
| #314 | Docs and public PST progress | Implemented |
| #315 | Validation and merge | Completed after final-head CI is green |

## Public fixture result

CI #269 against `tests/fixtures/pst/sample.pst` produced 50 BBT entries, 63 NBT entries, 11 folders, 1 message, and 0 attachments.

The key PQ14 signal is 1 message-level subnode probe attempt, 1 decoded subnode block, 0 failed subnode blocks, and 0 unsupported layouts.

## Revised next milestone

PQ15 should inspect and interpret the loaded message subnode payload as a possible table/property source.
