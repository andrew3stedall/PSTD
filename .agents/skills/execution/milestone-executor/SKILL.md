---
name: milestone-executor
description: Use when implementing an approved PSTD milestone or epic from ChatGPT/GitHub connector work. Executes the milestone issue order on a milestone branch and documents testing that must be run later.
---

# Milestone Executor

## Purpose

Build an approved PSTD milestone or epic as a coherent body of work, rather than prompting for each issue one by one.

## Inputs required

- Approved milestone or epic definition.
- Ordered issue list.
- Known scope and out-of-scope items.
- Expected output branch name.
- Known validation commands, if available.

## Execution model

1. Work from the milestone or epic definition.
2. Follow the issue order defined by that milestone or epic.
3. Use a milestone branch such as `milestone/<name>` or `epic/<name>`.
4. Implement related issues together when the milestone says they belong together.
5. Keep unrelated work out of the milestone branch.
6. Update docs as behaviour or usage changes.
7. Open a single milestone PR when the body of work is ready for review.

## Testing rule

Local testing may be deferred until the user has Codex running on a laptop. When tests are not run, record that clearly in the PR and list the commands that should be run later.

GitHub Actions CI is the normal validation gate for milestone PRs. After CI is green, inspect the `public-pst-progress` artifact and record the checked-in public PST fixture outcome in `docs/operations/public-pst-progress-log.md` before treating the milestone as complete.

## Stop conditions

Stop and report when:

- The milestone definition is unclear.
- Required files or repo context are missing.
- The work requires secrets or production access.
- The implementation path would exceed the milestone scope.

## Output

Return implementation summary, files changed, issues covered, validation performed, public PST progress result, deferred tests, docs changed, and follow-up work.
