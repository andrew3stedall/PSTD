# PQ15 Subnode Payload Interpretation

## Purpose

PQ15 follows PQ14 by interpreting whether the decoded message-level subnode block is a supported or unsupported layout.

## Implemented behaviour

- Reads decoded subnode block and unsupported-layout counters from the run status.
- Reports supported and unsupported message subnode layout counts.
- Routes the next blocker from the measured layout signal.

## Public fixture result

CI #275 against `tests/fixtures/pst/sample.pst` produced:

| Metric | Value |
|---|---:|
| BBT entries | 50 |
| NBT entries | 63 |
| Folders | 11 |
| Messages | 1 |
| Attachments | 0 |
| Decoded subnode blocks | 1 |
| Supported subnode layouts | 1 |
| Unsupported subnode layouts | 0 |

## Interpretation

The message-level subnode probe from PQ14 reached a supported decoded subnode block. Extraction counts remain unchanged because the supported payload is not yet wired as a table or property source for message extraction.

## Next blocker

`pq15_next_blocker=subnode_table_or_property_payload_interpretation`
