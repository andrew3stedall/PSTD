---
name: issue-writer
description: Use when creating or refining PSTD GitHub issues so they are scoped, dependency-aware, testable, and documented.
---

# Issue Writer Skill

## Purpose

Turn a planned work item into a GitHub issue that a developer can start without extra briefing.

## Required sections

Each issue should include:

- Goal.
- Background.
- In scope.
- Out of scope.
- User flow or system flow.
- Acceptance criteria.
- Technical notes.
- Data considerations.
- UX considerations.
- Operations considerations.
- Dependencies.
- Test expectations.
- Documentation required.
- Risk rating.
- Open questions.

## Quality rules

- Keep each issue small enough for one focused PR.
- Use observable and testable acceptance criteria.
- State dependencies clearly.
- Identify whether the issue is implementation, docs, data, UX, or operations focused.
- Mark missing decisions as open questions.
- Do not turn vague goals into implementation tasks until scope is clear.

## Final output

Return issue bodies in copy-ready Markdown, grouped by milestone and dependency order.
