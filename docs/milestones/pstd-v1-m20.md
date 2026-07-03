# PSTD v1 M20

## Goal

Add one focused attachment table decoder path with tests.

## Selected candidate

- Category: attachment table parser path.
- Target: compact attachment rows with UTF-16LE text fields.
- Magic: `CATW`.

## Scope

M20 adds decoding for compact rows where filename and content-type fields are UTF-16LE byte strings.

## Acceptance criteria

- Existing compact table tests still pass.
- New UTF-16 compact table tests pass.
- Unknown table shapes continue through the existing status path.

## Validation

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
python -m pstd --help
docker build -t pstd:local -f docker/Dockerfile .
```
