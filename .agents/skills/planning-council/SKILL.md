---
name: planning-council
description: Use when turning PSTD product ideas, PRDs, rough notes, or GitHub issue drafts into epics, milestones, developer-ready issues, acceptance criteria, dependencies, risks, and documentation requirements. This skill is planning-only and must not implement code.
---

# Planning Council Skill

## Purpose

Use this skill to plan PSTD work from a phone-first ChatGPT workflow or from a future Codex runtime. It converts vague product intent into structured delivery artefacts.

## Non-negotiable mode

Planning only. Do not implement application code, change runtime behaviour, create migrations, change credentials, or automate merges.

## Required role loop

Run these roles in order:

1. Executive Sponsor: confirm the work aligns with PSTD's core goal.
2. Product Owner: define user value, MVP relevance, and priority.
3. Business Analyst: decompose work into epics, milestones, issues, dependencies, and success criteria.
4. UX Designer: define user, CLI, API, or developer flows.
5. Data Scientist: define metrics, analysis, anomaly, inference, or evaluation requirements where relevant.
6. Data Engineer: define data contracts, quality checks, volume assumptions, and batch or streaming needs.
7. Systems Engineer: define operational, CI/CD, environment, and delivery constraints.
8. Developer: provide implementation feasibility notes only.
9. Docs Writer: define documentation updates required for each issue.

## Output

Produce a planning report with:

- Summary.
- Alignment decision.
- Proposed milestones.
- Proposed epics.
- Developer-ready issue list.
- Dependency order.
- Risks and blocked decisions.
- Documentation updates.
- Suggested labels.
- Next human decisions.

## Ready issue standard

Each proposed issue must include:

- Goal.
- Background.
- In scope.
- Out of scope.
- User flow or system flow.
- Acceptance criteria.
- Technical notes.
- Data considerations.
- UX considerations.
- Infrastructure and operations considerations.
- Dependencies.
- Test expectations.
- Documentation required.
- Risk rating.
- Open questions.
