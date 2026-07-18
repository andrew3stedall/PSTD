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

## GitHub connector implementation method

Large existing files are not a blocker when work is being performed through ChatGPT and the GitHub connector. Do not stop merely because the connector contents API replaces whole files or a fetched response is truncated.

Use the following preference order:

1. Use direct connector create, update, or delete operations for small files and changes that can be represented safely as complete file contents.
2. Use an authenticated local checkout with `git` and `gh` when one is available.
3. When no usable local checkout exists and a large existing file needs an incremental edit, use a temporary same-repository GitHub Actions checkout-and-patch workflow.

For the temporary Actions method:

1. Create a dedicated branch and draft pull request from the current `main` head.
2. Add a narrowly scoped temporary patch script and workflow through the connector.
3. Trigger only for the named same-repository PR branch. Never run write-capable patch automation for fork pull requests or untrusted refs.
4. Grant only the minimum required permission, normally `contents: write`. Do not expose repository secrets.
5. Check out the complete branch on the runner and apply exact deterministic replacements. Every replacement must assert that its expected source block occurs exactly once before modifying the file.
6. Run formatting and the most relevant focused tests before committing. Run broader required validation after the tested implementation reaches the branch.
7. Commit and push only the intended non-workflow production files from the runner. GitHub may reject `GITHUB_TOKEN` pushes that modify workflow files; edit workflow files separately through the connector when required.
8. Remove the temporary patch script and workflow through the connector immediately after the tested implementation is pushed.
9. Inspect the final PR diff and exact-head CI. The mergeable PR must contain only intended production, test, fixture, and documentation changes, with no patch scaffolding.

Prefer this method over manually reconstructing or replacing a large file from truncated connector output. If the workflow cannot push, inspect the job logs, preserve the already validated patch logic, narrow the pushed paths, and retry rather than reverting to speculative full-file replacement.

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