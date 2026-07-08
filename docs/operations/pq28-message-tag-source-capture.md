# PQ28 Message-Level Tag Source Capture

## Goal

Surface whether descriptor tag-source values are available at message/public-artifact level.

## Scope

- Add `pq28_message_status_lines`.
- Add `pq28_tag_source_capture_gap`.
- Keep extraction output unchanged.

## Boundary

PQ28 does not decode or materialize properties. It makes the capture gap explicit so the next PQ can target propagation safely.
