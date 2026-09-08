# AGENTS.md

## Repository purpose

PSTD is a Rust-first PST email extraction engine. The current priority is end-to-end extraction coverage and correctness: reliably recover folders, messages, metadata, bodies, recipients, threading information, and attachments before starting downstream Snowflake, UI, search, analytics, or graph work.

## Current operating mode

Mode: `vertical-extraction`.

The M1-M25 product-foundation lane is complete. The former PQ sequence established validated parser boundaries through PQ74. New work should implement the smallest coherent vertical slice that exposes new observable extraction behaviour rather than adding parser infrastructure for its own sake.

## Continuation fast path

Long-running implementation must preserve durable state so a new agent turn can resume without reconstructing the project.

When an existing implementation branch or PR has `docs/operations/active-implementation-checkpoint.md`, use this fast path before the fresh-start workflow:

1. Read this file, the active checkpoint, and the active issue.
2. Inspect the current branch/PR diff, latest durable commit, and only the source files named by the checkpoint.
3. Inspect current CI only when a check is failing, a merge decision depends on it, or the checkpoint explicitly requires it.
4. Continue from the checkpoint's `Next exact change` section.
5. Do not reread `README.md`, project status, the public-PST progress log, the full epic, historical milestone/PQ records, or the `docs/readpst-gaps/` corpus unless the checkpoint says one of those inputs changed or the current issue requires a specific cited section.

Historical parity and milestone documents are evidence libraries, not startup context. Never bulk-read them on a continuation. Fetch only the exact file or section needed to answer the active issue.

If the checkpoint is missing or stale, reconstruct it once from the branch/PR and immediately persist the result before doing broader research.

## Active implementation checkpoint

Every non-trivial implementation branch must maintain `docs/operations/active-implementation-checkpoint.md` while work is in progress. Keep it short and operational. It must record:

- active issue and parent epic, if any;
- branch, PR and latest durable implementation commit;
- scope and explicit out-of-scope items;
- established conclusions that must not be re-derived;
- source files/functions currently being changed;
- completed tests and their exact result;
- unresolved failures or blockers;
- `Next exact change`, written so a fresh agent can begin editing immediately.

Update the checkpoint whenever the active issue, implementation boundary, durable commit, test state, or next exact change materially changes. Delete or reset it when the implementation PR is completed so stale state cannot be mistaken for current work.

## Context and scope budget

- Treat one GitHub issue as the active implementation scope. An epic is navigation and dependency context, not permission to keep every child issue in working memory.
- Read sibling issues only when the active issue exposes a concrete API/dependency decision that requires them.
- Do not launch broad specialist-agent fan-out for a bounded implementation. Delegate only a separable question whose result can be persisted in a commit, issue comment, or checkpoint.
- If delegated analysis changes implementation direction, write the conclusion into the checkpoint before further delegation.
- Prefer targeted file reads, exact symbol searches, diffs, and failing log excerpts over whole-directory or whole-history ingestion.

## Durable progress rule

Do not allow meaningful implementation progress to exist only in transient agent context.

- After a bounded source/test/documentation increment reaches a coherent state, commit it to the active branch before broad research, environment work, full validation, or another delegation.
- Keep at most one bounded implementation increment uncommitted.
- Focused formatting/tests should run before the checkpoint commit when readily available, but lack of the full toolchain is not a reason to discard a coherent increment. Mark unverified checkpoint commits explicitly and validate them before merge.
- Full repository validation gates merge readiness, not the first durable commit.
- Infrastructure or temporary CI work must not become a prerequisite to preserving implementation code unless it is technically required to make the edit itself.

## Fresh-start workflow

Use this only when there is no resumable checkpointed implementation.

1. Review `README.md`, `docs/product/project-status.md`, and `docs/operations/public-pst-progress-log.md` once to establish the current baseline.
2. Check open pull requests and active branches to avoid conflicting work. Inspect recent commits and CI only to the extent needed to select or safely create the implementation branch.
3. Select exactly one highest-value issue or coherent vertical extraction gap.
4. Read only the specification, fixture evidence, source areas, and historical references directly relevant to that issue.
5. Create a dedicated branch/PR and immediately create the active implementation checkpoint.
6. Implement one bounded increment, add focused regression coverage, and persist it according to the durable progress rule.
7. Iterate from focused failures; update the checkpoint instead of repeating repository discovery.
8. When implementation is complete, run the full validation gate and relevant approved fixture workflows.
9. Update current-state documentation, point-in-time milestone record when useful, and changelog.
10. Inspect the exact final diff, resolve review threads, and squash merge only after the exact head is green.

## GitHub connector implementation method

Large existing files are not a blocker when work is being performed through ChatGPT and the GitHub connector. Do not stop merely because the connector contents API replaces whole files or a fetched response is truncated.

Use the following preference order:

1. Use direct connector create, update, or delete operations for small files and changes that can be represented safely as complete file contents.
2. Use an authenticated local checkout with `git` and `gh` when one is available.
3. When no usable local checkout exists and a large existing file needs an incremental edit, use a temporary same-repository GitHub Actions checkout-and-patch workflow.

The temporary Actions method is an editing/targeted-validation fallback, not a startup requirement. Preserve the implementation/checkpoint first whenever possible rather than spending a turn building a portable toolchain before any source change is durable.

For the temporary Actions method:

1. Use the existing dedicated branch and draft pull request when one exists; do not create a second implementation branch.
2. Add a narrowly scoped temporary patch script and workflow through the connector.
3. Trigger only for the named same-repository PR branch. Never run write-capable patch automation for fork pull requests or untrusted refs.
4. Grant only the minimum required permission, normally `contents: write`. Do not expose repository secrets.
5. Check out the complete branch on the runner and apply exact deterministic replacements. Every replacement must assert that its expected source block occurs exactly once before modifying the file.
6. Run formatting and the most relevant focused tests for the increment before the runner commits when practical.
7. Commit and push only the intended non-workflow production/test/documentation files from the runner. GitHub may reject `GITHUB_TOKEN` pushes that modify workflow files; edit workflow files separately through the connector when required.
8. Update the active checkpoint after the durable implementation commit is visible.
9. Remove the temporary patch script and workflow as soon as they are no longer needed and always before merge.
10. Run broader required validation only after the implementation has reached merge-ready scope, then inspect the final PR diff and exact-head CI.

Prefer this method over manually reconstructing or replacing a large file from truncated connector output. If the workflow cannot push, inspect the relevant failing log excerpt, preserve already validated patch conclusions in the checkpoint, narrow the pushed paths, and retry rather than restarting discovery.

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

The full gate is required on the cleaned, merge-ready extraction PR head. It is not required before every checkpoint commit.

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

For approved fixtures, also run inspect, extract, batch, and the deterministic public-PST progress workflow. Record the exact result and delta before merge.

During implementation, prefer the smallest focused tests that cover the changed boundary. Do not repeatedly run the full gate after each small increment unless a failure requires it.

## Pull request standard

Every merge-ready PR must state:

- objective and user-visible extraction value;
- previous evidence and the exact gap addressed;
- implementation scope and out-of-scope items;
- files changed and components reused;
- fail-closed behaviour and safety boundaries;
- tests and validation performed;
- public-PST result and delta when applicable;
- operational and data-contract impact;
- remaining blocker and proposed next vertical milestone.

Draft implementation PRs may use the active checkpoint as their working handoff and need not continuously rewrite the full final PR narrative.

## Documentation model

Current truth belongs in:

- `README.md`;
- `docs/product/project-status.md`;
- `docs/operations/public-pst-progress-log.md`;
- `docs/product/pstd-v1-roadmap.md`;
- the relevant architecture, engineering, data, and operations guides.

`docs/operations/active-implementation-checkpoint.md` is temporary branch-local execution state. It is deliberately concise and must not be treated as long-term product truth.

Milestone, PQ, vertical, issue-plan, parity-gap, and implementation-plan files are point-in-time records. Do not rewrite their historical conclusions as though they were current. Do not read the historical corpus by default on continuation.

## Skills

Use `.agents/skills/README.md` as the repository skills index. Skills remain reusable guidance, but this file, the active checkpoint, and the active issue take precedence for continuation work.