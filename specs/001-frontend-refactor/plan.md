# Implementation Plan: Frontend Maintainability Refactor

**Branch**: `001-frontend-refactor` | **Date**: 2026-03-08 | **Spec**: `/Users/ryuyb/Developer/vanguard/specs/001-frontend-refactor/spec.md`
**Input**: Feature specification from `/Users/ryuyb/Developer/vanguard/specs/001-frontend-refactor/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Refactor the React/Tauri frontend in behavior-preserving vertical slices so that route files become
thin entry points, feature modules own their internal components/hooks/utilities more clearly, and
duplicated or mixed-responsibility logic is reduced without changing current user-visible flows.

## Technical Context

**Language/Version**: TypeScript 5.x, React 19.x
**Primary Dependencies**: React, TanStack Router, Tauri frontend bindings, Vite, shared UI
component primitives
**Storage**: N/A for this frontend refactor; existing app state is provided through current Tauri
commands and route/session helpers
**Testing**: Existing project checks plus targeted automated coverage for extracted pure logic and
manual smoke validation for critical frontend flows
**Target Platform**: Desktop application frontend running in Tauri on macOS
**Project Type**: Desktop application frontend
**Performance Goals**: Preserve current startup route behavior, keep vault interactions responsive,
and avoid introducing noticeable regressions in search, selection, and navigation flows
**Constraints**: Must not change route paths, route guard outcomes, startup session sequencing,
or user-visible flow behavior; generated files and framework-required entry points should remain
stable unless a move is clearly safe
**Scale/Scope**: Refactor scoped frontend surfaces under `src/`, especially routes and feature
modules for auth, unlock, vault, and spotlight

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Code Quality Gate**: PASS. The smallest viable design is an incremental refactor of existing
  frontend seams rather than a top-level architecture rewrite. The primary files and modules in
  scope are `src/routes/index.tsx`, `src/routes/unlock.tsx`, `src/routes/vault.tsx`,
  `src/main.tsx`, `src/lib/route-session.ts`, `src/features/auth/login/**/*`,
  `src/features/auth/unlock/**/*`, `src/features/vault/**/*`, and any narrowly justified shared
  helpers in `src/lib` or `src/components/ui`. No new dependency, framework, or global layer is
  required.
- **Testing Gate**: PASS. Validation will combine targeted automated checks for extracted pure
  logic and route/session helpers with manual smoke coverage for login, unlock, vault, and
  spotlight behavior. Each implementation slice must define the evidence used to prove parity.
- **UX Consistency Gate**: PASS. Route paths, messages, interaction order, loading/error/locked
  states, and settings access remain unchanged. Route files may become thinner, but existing user
  flows and visible states must remain cohesive.
- **Performance Gate**: PASS. The plan preserves startup route resolution, avoids adding new boot
  work, keeps command invocation patterns stable where possible, and protects interactive vault
  behavior such as filtering, detail loading, and navigation responsiveness.

## Project Structure

### Documentation (this feature)

```text
specs/001-frontend-refactor/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── ui-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── bindings.ts
├── components/
│   └── ui/
├── features/
│   ├── auth/
│   │   ├── login/
│   │   │   ├── components/
│   │   │   ├── hooks/
│   │   │   ├── constants.ts
│   │   │   ├── index.ts
│   │   │   ├── types.ts
│   │   │   └── utils.ts
│   │   ├── shared/
│   │   └── unlock/
│   │       ├── components/
│   │       ├── hooks/
│   │       ├── index.ts
│   │       └── types.ts
│   ├── spotlight/
│   │   ├── components/
│   │   ├── hooks/
│   │   ├── constants.ts
│   │   ├── index.ts
│   │   ├── logging.ts
│   │   ├── types.ts
│   │   └── utils.ts
│   └── vault/
│       ├── components/
│       ├── hooks/
│       ├── constants.ts
│       ├── index.ts
│       ├── types.ts
│       └── utils.ts
├── lib/
│   ├── route-session.ts
│   └── utils.ts
├── main.tsx
├── routes/
│   ├── __root.tsx
│   ├── index.tsx
│   ├── unlock.tsx
│   └── vault.tsx
└── spotlight/
    └── main.tsx
```

**Structure Decision**: Keep the existing route-driven and feature-based structure rather than
introducing a new top-level taxonomy. Refactor by strengthening feature ownership within
`src/features/*`, keeping shared UI in `src/components/ui`, keeping app-wide runtime helpers in
`src/lib`, and reducing route-file responsibilities in `src/routes`.

## Phase 0: Research Summary

Research confirms the safest path is an incremental maintainability refactor:
- preserve the current top-level structure,
- thin route files first,
- split large feature orchestration hooks by responsibility,
- normalize shared helper boundaries only when reuse is demonstrated,
- validate through characterization-based checks around startup, auth, unlock, vault, and
  spotlight flows.

See `/Users/ryuyb/Developer/vanguard/specs/001-frontend-refactor/research.md`.

## Phase 1: Design Summary

### Data Model

The refactor centers on five planning entities:
- Frontend Feature Module
- Shared UI Module
- Shared Logic Module
- Route Entry Module
- Validation Evidence

These entities define ownership boundaries, acceptable reuse, and the evidence required to prove
behavior parity.

See `/Users/ryuyb/Developer/vanguard/specs/001-frontend-refactor/data-model.md`.

### Contracts

The main external contract for this work is the UI behavior contract. It preserves:
- route paths (`/`, `/unlock`, `/vault`),
- route guard outcomes,
- the stable route-decision seam in `resolveSessionRoute()`,
- the startup/session sync entrypoint in `src/main.tsx`,
- login/unlock/vault/spotlight flow behavior,
- detail loading semantics for superseded vault detail requests.

See `/Users/ryuyb/Developer/vanguard/specs/001-frontend-refactor/contracts/ui-contract.md`.

### Quickstart

The quickstart defines the intended slice order:
1. thin route files,
2. split orchestration hooks,
3. normalize helper boundaries,
4. revisit barrels last.

Execution ownership notes:
- `src/routes/index.tsx`, `src/routes/unlock.tsx`, and `src/routes/vault.tsx` should be reduced to
  route guards plus feature-owned page composition.
- `src/main.tsx` should remain the startup route-sync entrypoint.
- `src/lib/route-session.ts` should remain the canonical session-routing seam.
- Shared helper consolidation should prefer `src/lib` for app-wide primitives and keep
  feature-specific normalization close to the owning feature unless reuse is demonstrated.

See `/Users/ryuyb/Developer/vanguard/specs/001-frontend-refactor/quickstart.md`.

## Post-Design Constitution Check

- **Code Quality Gate**: PASS. The design stays within the smallest viable change set and names the
  exact route, feature, and helper modules to refactor without introducing a new architectural
  layer.
- **Testing Gate**: PASS. The design defines both automated and manual parity evidence for the
  highest-risk flows touched by the refactor.
- **UX Consistency Gate**: PASS. The UI contract explicitly preserves route behavior, messages,
  flow states, and interaction parity.
- **Performance Gate**: PASS. The design preserves startup routing behavior and avoids route or
  boot changes that would create new latency or command churn.

## Complexity Tracking

No constitution violations currently require justification.
