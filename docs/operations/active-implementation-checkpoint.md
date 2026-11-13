# Active implementation checkpoint

- Issue: #621 `[FIXTURE-01]` deterministic synthetic Unicode PST generator. Related: #618.
- Branch: `t3/synthetic-pst-generator`. PR: not opened yet.
- Scope: `tools/synthetic_pst.rs` (dependency-free emitter + manifest), an integration
  test that checks PSTD against the manifest, a large-run measurement, docs.
- Out of scope: Outlook-openable output (no allocation maps), ANSI, encryption.

## Established conclusions

- The emitter is written from the MS-PST layouts and shares no reader code. It is also
  usable as a module: `#[path = "../tools/synthetic_pst.rs"] mod synthetic_pst;`.
- User requirement: keep Drafts, Deleted Items, and corrupt attachments on otherwise
  valid emails, so robustness is exercised (scenarios are in `SPECIAL_SCENARIOS`).
- First run (60 messages, seed 7): output is byte-deterministic; PSTD finds 13 folders,
  60 message candidates, loads 59 (one deliberately missing block).

## Next exact change

Compare PSTD extract output with the manifest per scenario (subjects, folders, bodies,
recipients, attachment bytes, containment of each fault). Fix gaps in PSTD where they
are bounded; record the rest as follow-up issues. Then add `tests/synthetic_pst.rs`,
measure a large run, document in `docs/fixtures/`, changelog, PR, Sol review.
