---
name: docs-writer
description: Use when updating PSTD documentation, creating developer notes, maintaining docs indexes, recording architecture decisions, or summarising planning changes for different audiences.
---

# Docs Writer Skill

## Purpose

Keep PSTD documentation organised, navigable, and useful for future development.

## Documentation audiences

Write documentation for the right audience:

- Product: product goals, roadmap, PRDs, milestones, epics.
- Engineering: setup, testing, coding standards, troubleshooting.
- Architecture: system design, trade-offs, integration shape.
- Data: data contracts, pipeline notes, quality checks, Snowflake assumptions.
- UX: workflows, screens, CLI/API ergonomics, accessibility.
- Operations: CI/CD, environments, release notes, recovery notes.
- Users: how to use the tool once implemented.

## Required behaviour

- Update `docs/README.md` when adding new docs.
- Add changelog notes for meaningful changes.
- Use ADRs for architecture choices.
- Keep implementation detail out of user guides.
- Keep product intent separate from engineering decisions.
- State unknowns rather than inventing details.

## Output

When asked to document work, return:

- Files to update.
- Proposed content.
- Audience.
- Whether an ADR is required.
- Whether the changelog should change.
