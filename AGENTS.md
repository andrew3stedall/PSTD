# AGENTS.md

## Repository purpose

PSTD is a Rust-first PST email extraction engine. The current priority is end-to-end extraction coverage and correctness: reliably recover folders, messages, metadata, bodies, recipients, threading information, and attachments before starting downstream Snowflake, UI, search, analytics, or graph work.

## Current operating mode

Mode: `vertical-extraction`.

The M1-M25 product-foundation lane is complete. The former PQ sequence established validated parser boundaries through PQ74. New work should implement the smallest coherent vertical slice that exposes new observable extraction behaviour rather than adding parser infrastructure for its own sake.

## Required workflow

1. Review `README.md`, `docs/product/project-status.md`, and `docs/operations/public-pst-progress-log.md`.
2. Check open pull requests, branches, recent commits, and current CI before creating work. Continue an existing implementation when one is already in progress instead of creating conflicting changes.
3. Compare the current parser behaviour with the PST specification and measured fixture evidence.
4. Identify the single highest-value extraction gap that can be completed safely.
5. Implement exactly one coherent vertical milestone on a dedicated branch.
6. Reuse validated components and avoid duplicate parsing logic.
7. Fail closed on ambiguity, malformed structures, unsupported types, or incomplete evidence.
8. Add focused regression tests and preserve existing behaviour.
9. Run the full validation gate and inspect the public-PST artifact.
10. Update the current-state documentation, point-in-time milestone record, and changelog.
11. Open a pull request with explicit extraction impact, validation evidence, risks, and the next measured blocker.
12. Squash merge only after the exact head is green and review threads are resolved.

## Scope rules

Allowed:

- bounded PST parser and extraction changes;
- CLI, output, batch, and diagnostics changes required by a coherent extraction slice;
- synthetic and approved public-fixture tests;
- documentation, issues, branches, pull requests, and CI follow-up;
- revision of the proposed next milestone when repository evidence identifies a higher-value path.

Not allowed without explicit approval:

- unrelated broad refactors;
- direct commits to `main`;
- secret, billing, authentication, production-access, or deployment changes;
- Snowflake, UI, search, analytics, semantic search, or graph implementation;
- heuristic interpretation of unvalidated PST bytes;
- claims that tests, fixtures, or compatibility passed when they were not verified.

## Correctness principles

- Prefer one complete vertical behaviour over several new abstractions.
- Preserve raw evidence and authoritative property identity where needed for validation.
- Keep address kinds, encodings, row order, and source boundaries explicit.
- Never combine values from separate runs and present them as one extraction result.
- Return no partial record when row counts, bounds, types, mappings, or references disagree.
- Keep diagnostic output bounded and exclude private payload data.
- Treat one public fixture as evidence, not proof of general PST compatibility.

## Validation gate

Every extraction pull request must pass:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- version
cargo run -- inspect --help
cargo run -- batch --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

For approved fixtures, also run inspect, extract, batch, and the deterministic public-PST progress workflow. Record the exact result and delta.

## Pull request standard

Every PR must state:

- objective and user-visible extraction value;
- previous evidence and the exact gap addressed;
- implementation scope and out-of-scope items;
- files changed and components reused;
- fail-closed behaviour and safety boundaries;
- tests and validation performed;
- public-PST result and delta;
- operational and data-contract impact;
- remaining blocker and proposed next vertical milestone.

## Documentation model

Current truth belongs in:

- `README.md`;
- `docs/product/project-status.md`;
- `docs/operations/public-pst-progress-log.md`;
- `docs/product/pstd-v1-roadmap.md`;
- the relevant architecture, engineering, data, and operations guides.

Milestone, PQ, vertical, issue-plan, and implementation-plan files are point-in-time records. Do not rewrite their historical conclusions as though they were current; link them from the current-state documents and classify them through `docs/DOCUMENTATION_STATUS.md`.

## Skills

Use `.agents/skills/README.md` as the repository skills index. Skills remain reusable guidance, but this file and the current project-status documents take precedence when an older skill still refers to the completed milestone-planning lane.
