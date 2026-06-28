# References and Assets Index

## Purpose

This file lists reusable reference material and copyable assets that support PSTD planning and milestone execution.

## Data references

- `roles/data/references/output-contract.md`: v1 output directory and metadata shape.
- `roles/data/references/error-policy.md`: extraction error handling and recovery policy.

## Data assets

- `roles/data/assets/manifest.example.json`: run manifest example.
- `roles/data/assets/error-log.example.jsonl`: error log examples.
- `roles/data/assets/extraction-summary.example.json`: run summary example.
- `roles/data/assets/message-metadata.example.json`: per-message metadata example.

## Full-stack developer references

- `roles/full-stack-developer/references/rust-project-structure.md`: suggested Rust crate and module layout.
- `roles/full-stack-developer/references/cli-design.md`: initial CLI command, options, progress, and exit codes.
- `roles/full-stack-developer/references/module-boundaries.md`: separation between CLI, extractor, output, errors, and progress.

## Execution assets

- `execution/milestone-executor/assets/milestone-pr-template.md`: milestone PR body template.
- `execution/deferred-testing/assets/deferred-testing-block.md`: standard deferred testing wording.

## Usage rule

Skills should prefer these references and assets over inventing new formats. When a milestone changes a contract or template, update the relevant reference or asset in the same PR.
