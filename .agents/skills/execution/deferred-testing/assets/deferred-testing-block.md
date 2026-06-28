# Deferred Testing Block

Use this block in PRs when work was done from ChatGPT mobile or the GitHub connector and local validation was not available.

```md
## Tests deferred

Local tests were not run from the phone/GitHub connector workflow.

Reason:

- Local Codex/laptop environment is not available yet.

Commands to run later:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

Risk areas:

- Build errors not detected locally.
- Formatting or lint issues not detected locally.
- Behaviour not validated against fixtures yet.

Release note:

- Do not treat this milestone as release-verified until local or CI validation has run.
```
