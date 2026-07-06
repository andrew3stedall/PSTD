# PQ9 Property Tag-Shape Status Reporting

## Purpose

PQ9 makes the PQ8 parser diagnostics visible in the public PST progress artifact.

PQ8 added parser-level counters, but the compact public artifact still only exposed the older PQ6 selected/unknown counts. PQ9 carries the tag-shape signal through message status and aggregates it into `run_summary.status`.

## Exposed counters

`run_summary.status` now includes:

| Counter | Meaning |
|---|---|
| `pq9_plausible_property_tags` | Parsed property-context keys whose low 16 bits look like known MAPI value types. |
| `pq9_suspicious_property_keys` | Parsed property-context keys whose low 16 bits do not look like known MAPI value types. |
| `pq9_byte_swapped_selected` | Selected properties recovered only through the conservative byte-swapped selected-tag rule. |
| `pq9_next_blocker` | Decision signal for the next conversion-quality milestone. |

## Decision rule

- If suspicious keys dominate, the next blocker is `heap_bth_layout_traversal`.
- If plausible tags dominate, the next blocker is `selected_mapi_dictionary_expansion`.
- If no tag-shape signal is visible, the next blocker is `property_context_signal_absent`.

## Public fixture result

CI #218 against `tests/fixtures/pst/sample.pst` produced:

| Metric | Value |
|---|---:|
| Plausible property tags | 0 |
| Suspicious property keys | 70 |
| Byte-swapped selected properties | 0 |

The measured decision signal is `pq9_next_blocker=heap_bth_layout_traversal`.

## Non-goals

- PQ9 does not expand the selected property dictionary.
- PQ9 does not repair heap/BTH traversal.
- PQ9 does not expand body, attachment, or recipient payload extraction.

## Follow-up

The next conversion-quality milestone should repair real heap-on-node/BTH/property-context traversal before additional selected property dictionary, body, attachment, or recipient expansion work.
