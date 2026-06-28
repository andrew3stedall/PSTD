# PSTD

PSTD is a PST email data tool. The v1 command is `pstd`.

## M1 status

M1 adds the local foundation:

- Rust CLI shell.
- TAR shard writer.
- JSONL writer.
- Stable ID and path helpers.
- Status and progress records.
- Python wrapper boundary.
- Local Docker scaffold.

PST binary parsing is planned for a later milestone.

## Local validation commands

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
pstd --help
pstd extract --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```

## Planning docs

- [Documentation index](docs/README.md)
- [PSTD v1 MVP PRD](docs/product/pstd-v1-mvp-prd.md)
- [PSTD v1 Roadmap](docs/product/pstd-v1-roadmap.md)
- [M1 milestone](docs/milestones/pstd-v1-m1-extraction-foundation.md)
- [M1 issue plan](docs/issues/pstd-v1-m1-ordered-issue-plan.md)
