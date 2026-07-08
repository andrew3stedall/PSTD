# PQ27 Descriptor Tag Source

PQ27 implements issue #350.

## Expected decision

- If unknown tag values surface, PQ28 should classify those tags and decide whether the parser is using the wrong descriptor field.
- If tag values still do not surface, PQ28 should move descriptor capture closer to message-level subnode status.
