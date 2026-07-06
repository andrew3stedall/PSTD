# PQ13 Payload Source Diagnostics

## Purpose

PQ13 follows PQ12 by revising the next blocker from payload boundary probing to payload source selection.

## Implemented behaviour

- Reads existing subnode source counters from the extraction status.
- Surfaces PQ13 payload-source counters in the run summary.
- Separates planned subnode decode sources from actual decode attempts.
- Preserves existing parsing and fallback behaviour.

## Public fixture result

CI #263 against `tests/fixtures/pst/sample.pst` produced:

| Metric | Value |
|---|---:|
| BBT entries | 50 |
| NBT entries | 63 |
| Folders | 11 |
| Messages | 1 |
| Attachments | 0 |
| Plausible property tags | 0 |
| Suspicious property keys | 70 |
| PQ12 no-signature payloads | 1 |
| PQ13 subnode references | 3 |
| PQ13 subnode decode plans | 3 |
| PQ13 subnode decode attempts | 0 |

## Interpretation

The public fixture contains subnode references and decode plans, but the message extraction path performs no subnode decode attempts for the true message candidate. The current implementation only decodes subnodes through the attachment path after attachment metadata is visible.

## Next blocker

`pq13_next_blocker=message_subnode_payload_selection`
