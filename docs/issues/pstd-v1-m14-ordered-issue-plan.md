# PSTD v1 M14 Ordered Issue Plan

## Branch

`pstd-v1-m14-recursive-subnode-layouts`

## Issues

| Order | Issue | Title | Notes |
|---:|---:|---|---|
| 1 | #102 | Add subnode layout classification reports | Classify loaded subnode blocks and record compatibility status. |
| 2 | #103 | Add bounded recursive child loading | Follow known child-reference layouts within parser limits. |
| 3 | #104 | Wire recursive subnode loading into extraction | Use recursive bounded loading for attachment subnode extraction. |
| 4 | #105 | Add M14 validation and handoff notes | Finish docs and validation record. |

## Execution rule

Build on the existing M1-M13 foundation. Keep CI green. Do not add Snowflake, search, web UI, distributed execution, real mailbox data, or broad parser rewrites in this slice.

## Validation suite

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --help
cargo run -- batch --help
cargo run -- inspect --help
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```
