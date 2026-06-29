# Developer Onboarding

## First 10 minutes

1. Read the root [README](../../README.md).
2. Read the [Project Status](../product/project-status.md).
3. Read the [Developer Guide](../engineering/developer-guide.md).
4. Read the [Codebase Map](../engineering/codebase-map.md).
5. Read the [Local Validation](../operations/local-validation.md) guide.

## Current implementation baseline

M1-M3 are implemented on `main`.

The active command surface is:

```text
pstd inspect --input <small-approved-fixture.pst>
pstd inspect --input <small-approved-fixture.pst> --json
pstd extract --input <small-approved-fixture.pst> --output <tmp-output> --manifest-only
```

The implementation is not release-verified until the validation commands have run.

## Where to work

| Work type | Start with |
|---|---|
| Parser changes | `src/pst/` and relevant milestone docs |
| Output contract changes | `src/output/`, `docs/data/`, and output-contract skill reference |
| CLI changes | `src/cli.rs`, CLI design docs, developer guide |
| Extraction orchestration | `src/engine/` |
| Python wrapper changes | `python/src/pstd/` |
| Documentation changes | `docs/README.md`, changelog, affected audience docs |

## Current next milestone

M4: Recipients, Threading, and Address Resolution.

Before implementing M4, validate the current baseline locally or in CI.
