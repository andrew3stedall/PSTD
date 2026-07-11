# PQ48 — table-probe engine integration

## Decision

PQ48 wires the existing PQ45–PQ47 reporting path into both successful recursive subnode loads in metadata extraction. No additional abstraction is introduced.

## Changes

- Instantiate one `TcRunProbeCollector` per extraction.
- Record decoded payloads from the message-level and attachment-level recursive subnode paths.
- Finalize once after message processing.
- Add the bounded attributed issue only when a table heap is observed.
- Append the aggregate status under `pq48_table_probe_status`.

## Before versus after

| Measure | Before PQ48 | After PQ48 |
|---|---:|---:|
| Engine call sites recording table probes | 0 of 2 | 2 of 2 |
| Run-level table counters in extraction status | No | Yes |
| Attributed table evidence in issues | Helper only | Emitted when observed |
| Extracted messages, bodies, attachments, EMLs | Existing baseline | Expected unchanged |
| Diagnostic fragments | Capped at 16 | Capped at 16 |

## Evidence required from CI/public fixture

Capture folders, messages, bodies, attachment payloads/records, output bytes, recursive decoded blocks, detected/resolved/failed table heaps, row references and attributed node/BID diagnostics. Compare these values directly with the last green public fixture.

## Risks and next decision

A zero table-heap count would prove the current recursive paths do not reach the expected table context in the public fixture. In that case PQ49 should investigate reachability and fixture selection rather than adding row materialisation. A positive resolved count with in-bounds row references would justify bounded row-width and column-boundary validation.
