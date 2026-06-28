# PSTD

PST email data extractor. The current direction is to use Rust to process PST files and extract individual emails.

## Project planning

This repo includes a planning-only Codex Delivery Council scaffold designed for a phone-first workflow.

The current workflow is:

```text
Prompt from ChatGPT on mobile
  -> use GitHub connector to read/update repo
  -> apply planning council logic
  -> create docs, issues, or PRs
  -> review from GitHub mobile or ChatGPT
```

Start with:

- [Documentation index](docs/README.md)
- [Planning council overview](docs/product/council-overview.md)
- [PRD intake template](docs/product/prd-intake-template.md)
- [Phone-first operating model](docs/product/phone-first-operating-model.md)
- [Issue template](.github/codex/templates/issue-template.md)
- [Epic template](.github/codex/templates/epic-template.md)

## Repo-scoped skills

Codex-style skills are committed under `.agents/skills/`:

- `planning-council`: turn ideas or PRDs into milestones, epics, issues, and risks.
- `issue-writer`: create developer-ready GitHub issue bodies.
- `docs-writer`: maintain audience-specific docs.
- `github-planning-loop`: operate from ChatGPT mobile through the GitHub connector.

These skills prepare the repo for future Codex runtimes while also documenting the process used from this ChatGPT conversation.

## Current automation mode

Planning only. Execution mode, scheduled GitHub Action automation, API-key Codex usage, and parallel subagents are intentionally disabled or delayed.
