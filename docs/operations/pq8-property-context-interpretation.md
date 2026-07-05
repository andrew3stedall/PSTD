# PQ8 Property-Context Layout and Tag Interpretation

## Purpose

PQ8 targets the measured PQ7 blocker: the public PST fixture has one true message candidate with a loadable property context, but the parsed property keys still produce 0 selected MAPI properties and 74 unknown properties.

## What changed

PQ8 adds bounded tag-shape diagnostics to property-context parsing:

- `plausible_property_tag_count`: parsed keys whose low 16 bits match a known MAPI property value type.
- `suspicious_property_tag_count`: parsed keys whose low 16 bits do not look like a known MAPI value type.
- `byte_swapped_selected_property_count`: parsed keys safely recovered only when the direct key shape is invalid and the byte-swapped key maps to an already selected MAPI property.

## Safe interpretation rule

PQ8 preserves conservative unknown handling. A tag is reinterpreted only when:

1. The direct little-endian key does not have a known MAPI value-type shape.
2. The byte-swapped key maps to an existing selected MAPI property definition.

Unknown or unsupported property IDs are not mapped speculatively.

## Diagnostic value

These counters separate two different failure modes:

| Failure mode | Meaning | Likely next action |
|---|---|---|
| Mostly plausible unknown tags | The parser is probably reading property tags, but the selected dictionary is too narrow. | Expand selected MAPI dictionary using observed tags. |
| Mostly suspicious keys | The parser is probably reading the wrong structure or offset. | Fix heap/BTH/property-context layout traversal. |

## Explicit non-goals

- Attachment payload expansion.
- Recipient expansion.
- Body expansion except where improved property selection naturally exposes body tags.
- Snowflake, UI, search, or analytics work.

## Next blocker

The next blocker is determined from the PQ8 public PST artifact. If selected properties remain 0 and suspicious keys dominate, the next milestone should focus on real heap-on-node/BTH layout traversal rather than dictionary expansion.
