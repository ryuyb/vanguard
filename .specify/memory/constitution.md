<!--
Sync Impact Report
- Version change: 0.0.0 -> 1.0.0
- Modified principles:
  - [PRINCIPLE_1_NAME] -> I. Code Quality Is Enforced At Review Time
  - [PRINCIPLE_2_NAME] -> II. Tests Prove Behavior Before Completion
  - [PRINCIPLE_3_NAME] -> III. User Experience Must Stay Cohesive
  - [PRINCIPLE_4_NAME] -> IV. Performance Budgets Are Feature Requirements
  - [PRINCIPLE_5_NAME] -> V. Simplicity And Traceability Over Cleverness
- Added sections:
  - Delivery Standards
  - Workflow & Quality Gates
- Removed sections:
  - None
- Templates requiring updates:
  - ✅ updated /Users/ryuyb/Developer/vanguard/.specify/templates/plan-template.md
  - ✅ updated /Users/ryuyb/Developer/vanguard/.specify/templates/spec-template.md
  - ✅ updated /Users/ryuyb/Developer/vanguard/.specify/templates/tasks-template.md
  - ⚠ pending /Users/ryuyb/Developer/vanguard/docs/release-process.md (no principle references required; reviewed only)
- Follow-up TODOs:
  - TODO(RATIFICATION_DATE): original adoption date unknown; set when historical project governance date is available.
-->
# Vanguard Constitution

## Core Principles

### I. Code Quality Is Enforced At Review Time
All production changes MUST be minimal, intentional, and easy to reason about.
Every change MUST name the concrete files it affects, reuse established project
patterns before adding new abstractions, and justify any new dependency,
indirection, or architectural layer. Code that is difficult to read, duplicates
logic without reason, or broadens scope beyond the requested outcome MUST be
reworked before merge. Rationale: this project values maintainability and
predictable iteration over cleverness.

### II. Tests Prove Behavior Before Completion
Every behavior change MUST include automated validation proportional to the
risk of the change. A task is not complete until the relevant tests, checks, or
reproducible validation steps have been identified and executed. Bug fixes MUST
add or update regression coverage when the affected layer has test support; new
features MUST define the unit, integration, or workflow-level evidence that
proves they work. Rationale: completion is defined by verified behavior, not by
code written.

### III. User Experience Must Stay Cohesive
User-facing changes MUST preserve a consistent experience across navigation,
copy, visual hierarchy, states, and accessibility. New UI MUST match existing
interaction patterns unless a deliberate product-level change is documented.
Loading, empty, error, success, and destructive states MUST be considered for
all new flows. Any deviation in terminology, motion, spacing, or control
behavior MUST be justified in the specification or plan. Rationale: coherent UX
reduces user friction and prevents feature-by-feature drift.

### IV. Performance Budgets Are Feature Requirements
Performance constraints MUST be defined whenever a feature can affect startup
cost, rendering smoothness, bundle size, memory use, I/O latency, or core task
completion time. Plans MUST state the relevant budget or expectation and how it
will be validated. Changes that materially regress responsiveness, animation
smoothness, or release build size without documented approval MUST not ship.
Rationale: performance is a user-facing quality attribute, not a post-launch
cleanup item.

### V. Simplicity And Traceability Over Cleverness
The default implementation MUST be the simplest approach that satisfies current
requirements while remaining observable in code review. Features MUST map
cleanly from specification to plan to tasks to shipped files. Hidden coupling,
one-off frameworks, and speculative abstractions MUST be avoided unless a
specific current requirement demands them. Rationale: straightforward systems
are easier to review, test, and evolve.

## Delivery Standards

- Specifications MUST include measurable functional outcomes, relevant UX
  consistency constraints, and any performance expectations that affect
  acceptance.
- Implementation plans MUST pass explicit constitution gates for code quality,
  testing, UX consistency, and performance before design work proceeds.
- Task breakdowns MUST include the validation work needed to prove each user
  story, including automated tests and any UX or performance verification named
  in the specification.
- Release work MUST preserve existing SemVer practices documented in
  `docs/release-process.md`.

## Workflow & Quality Gates

- During planning, teams MUST identify the smallest viable change set and list
  the concrete modules, views, services, or routes being modified.
- During implementation, contributors MUST keep scope aligned to the approved
  behavior and avoid opportunistic refactors unless they are required to safely
  deliver the change.
- During review, reviewers MUST verify evidence for all four gates: code
  quality, testing, UX consistency, and performance.
- Before merge, all required automated checks MUST pass and any manual
  validation steps MUST be recorded in the work artifact or review notes.

## Governance

This constitution supersedes conflicting local process notes for feature
specification, planning, and task generation. Amendments MUST be documented in
`.specify/memory/constitution.md`, include a Sync Impact Report, and update any
impacted templates before the amendment is considered complete.

Versioning policy for this constitution follows semantic versioning:
- MAJOR: removes or materially redefines a principle or governance rule.
- MINOR: adds a principle, section, or materially expands required practice.
- PATCH: clarifies wording without changing project obligations.

Compliance review is mandatory for every planned feature and every merge review.
Plans MUST explicitly address code quality, testing, UX consistency, and
performance. Reviewers MUST block changes that do not provide sufficient
evidence of compliance or that bypass required template updates.

**Version**: 1.0.0 | **Ratified**: TODO(RATIFICATION_DATE): original adoption date unknown | **Last Amended**: 2026-03-08
