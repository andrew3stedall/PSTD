# Patch application workflow

PSTD can apply connector-generated source patches through a guarded GitHub Actions checkout. This supports phone-only development without replacing complete large Rust files through the GitHub contents API.

## Workflow

The workflow is defined in `.github/workflows/apply-repository-patch.yml` and is started manually from the Actions tab after it has been merged to the default branch.

It accepts three inputs:

- `branch`: an existing non-default branch beginning with `extract-`, `emit-`, `fix-`, or `add-`;
- `patch_path`: a `.patch` file below `changes/` on that branch;
- `commit_message`: the commit message used after validation succeeds.

## Connector-driven process

1. Create a milestone branch from current `main`.
2. Generate a unified diff against that exact branch state.
3. Write the diff to `changes/<milestone>.patch` on the milestone branch.
4. Run **Apply repository patch** from GitHub Actions with the branch and patch path.
5. Review the Action result and resulting commit.
6. Open or update the pull request only after the workflow succeeds.
7. Run the normal fixture and CI workflows before merge.

The patch file is deleted by the workflow and is not retained in the resulting source commit.

## Safety boundaries

The workflow:

- never accepts `main` or `master` as a target;
- restricts target branch prefixes;
- restricts patch files to `changes/*.patch`;
- rejects empty patches and patches over 250,000 bytes;
- checks the patch with `git apply --check` before applying it;
- rejects changes to `.github/workflows/`;
- serializes executions per target branch;
- runs `cargo fmt --all`, `cargo test --all-targets`, Clippy with warnings denied, and `git diff --check`;
- commits and pushes only after every validation step succeeds.

Normal branch protection and pull-request review remain the final merge boundary.

## Failure handling

A failed run leaves the branch unchanged because the commit and push occur only in the final step. Correct the patch file on the branch and rerun the workflow. Do not bypass a failed validation by applying source changes directly to the default branch.
