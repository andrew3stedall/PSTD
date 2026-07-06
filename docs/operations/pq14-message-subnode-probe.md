# PQ14 Message Subnode Probe

## Purpose

PQ14 follows PQ13 by turning planned message-level subnode payload sources into bounded decode attempts.

## Implemented behaviour

- For true message candidates with a subnode reference and no decoded attachment metadata, performs a bounded message-level subnode probe.
- Counts probe attempts, decoded blocks, failed blocks, and unsupported layouts.
- Preserves the existing attachment-driven subnode extraction path.

## Public fixture result

CI #269 against `tests/fixtures/pst/sample.pst` produced:

| Metric | Value |
|---|---:|
| BBT entries | 50 |
| NBT entries | 63 |
| Folders | 11 |
| Messages | 1 |
| Attachments | 0 |
| Plausible property tags | 0 |
| Suspicious property keys | 70 |
| PQ14 probe attempts | 1 |
| PQ14 decoded subnode blocks | 1 |
| PQ14 failed subnode blocks | 0 |
| PQ14 unsupported layouts | 0 |

## Interpretation

PQ14 moved the public fixture from planned message subnode sources to an actual decoded message-level subnode block. Extraction counts and property tags remain unchanged because the loaded subnode payload is not yet interpreted as a property/table source for the message.

## Next blocker

PQ15 should interpret the loaded message subnode payload and decide whether it is a table context, property context, or another safe source candidate.
