---
name: subagent-model-routing
description: Route Codex delegated work to the cheapest sufficiently capable model, preferring Luna for bounded leaf tasks and escalating only when complexity or risk requires it.
---

# Subagent Model Routing

## Purpose

Reduce subagent cost and context consumption without lowering correctness by matching the delegated task to the smallest sufficiently capable Codex model.

## Default routing

Use the model per delegated task rather than inheriting the parent orchestrator's model automatically when Codex exposes explicit subagent model selection.

### GPT-5.6 Luna

Prefer Luna for bounded, low-complexity leaf tasks such as:

- targeted repository/code searches;
- locating symbols, usages, tests, or fixtures;
- small mechanical edits with precise instructions;
- straightforward unit-test additions or updates;
- inspecting a small fixture or log excerpt;
- concise documentation edits;
- summarising a small, already identified source boundary.

Keep Luna tasks narrow. Supply only the relevant context/fork and do not ask a Luna worker to orchestrate further subagents.

### GPT-5.6 Terra

Use Terra for ordinary implementation work that requires moderate reasoning across several files, non-trivial test design, or integration of a few established components but does not require consequential architectural judgment.

### GPT-5.6 Sol or GPT-6 Astra

Use Sol or Astra when the task involves one or more of:

- ambiguous PST/MAPI semantics;
- consequential architecture or data-contract decisions;
- difficult debugging with multiple plausible causes;
- broad cross-component integration judgment;
- security/correctness-sensitive review;
- merge-critical review where weaker-model uncertainty would materially increase risk.

## Escalation

Escalate when the actual task becomes more complex than expected, the worker stalls, evidence conflicts, or correctness risk becomes material. Do not keep retrying the same task on Luna merely to save cost.

## Context discipline

- Prefer a fresh, bounded child context/fork over hydrating the full parent history when the runtime permits it.
- Provide the active issue/checkpoint conclusion and only the source/test boundary needed for the delegated task.
- Do not send the full epic, historical parity corpus, or unrelated source areas to a leaf worker.

## Durable output

Every delegation must produce reusable output before overlapping work is launched: a code/test/docs commit, durable issue/PR comment, or concise checkpoint conclusion. If it does not, narrow the task or return it to the parent rather than repeating broad delegation.

## Correctness rule

Model economy is subordinate to correctness. The parent orchestrator owns integration, validation, and escalation decisions regardless of which model produced the delegated result.
