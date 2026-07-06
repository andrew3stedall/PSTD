# PQ15 Subnode Payload Interpretation

## Goal

Use the PQ14 result to classify the decoded message subnode block as a supported or unsupported payload source.

## Scope

PQ15 is bounded to issues #317 through #321.

| Issue | Scope | Status |
|---:|---|---|
| #317 | Parent scope | Implemented in this PR |
| #318 | Count message subnode payload layouts | Implemented |
| #319 | Report next blocker | Implemented |
| #320 | Docs and public progress | Implemented |
| #321 | Validation and merge | Completed after final-head CI is green |

## Public fixture result

CI #275 against `tests/fixtures/pst/sample.pst` produced 50 BBT entries, 63 NBT entries, 11 folders, 1 message, and 0 attachments.

The key PQ15 signal is 1 decoded subnode block, 1 supported subnode layout, and 0 unsupported subnode layouts.

## Revised next milestone

PQ16 should interpret the supported subnode payload as a possible table or property source for the message candidate.
