#   / Tasks: Frontend Maintainability Refactor

**Input**: Design documents from `/specs/001-frontend-refactor/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Include tasks for the automated tests and validation work required by the constitution. Every user story MUST include the tasks needed to prove correctness, and any performance or UX validation tasks required by the spec.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish the refactor safety rails and documentation artifacts used by all slices.

- [ ] T001 Create a behavior inventory checklist for login, unlock, vault, and spotlight flows in `specs/001-frontend-refactor/quickstart.md`
- [ ] T002 Capture the current route/feature refactor scope and target file ownership notes in `specs/001-frontend-refactor/plan.md`
- [ ] T003 [P] Review shared helper candidates and cross-feature utility leakage in `src/lib/utils.ts`, `src/features/auth/shared/utils.ts`, `src/features/vault/utils.ts`, and `src/features/spotlight/utils.ts`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Put shared validation and route/session safety seams in place before user-story refactors begin.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Add targeted route-session coverage for stable route outcomes in `src/lib/route-session.ts`
- [ ] T005 [P] Extract or document shared no-regression validation steps for startup route resolution in `src/main.tsx` and `src/lib/route-session.ts`
- [ ] T006 [P] Define a stable route composition contract for `src/routes/index.tsx`, `src/routes/unlock.tsx`, and `src/routes/vault.tsx`
- [ ] T007 Create a validation evidence log template for slice-by-slice parity checks in `specs/001-frontend-refactor/contracts/ui-contract.md`

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Preserve Existing Behavior While Reorganizing Code (Priority: P1) 🎯 MVP

**Goal**: Keep current frontend behavior intact while moving route composition and feature orchestration into safer internal boundaries.

**Independent Test**: Verify startup routing plus the login, unlock, vault, and spotlight flows still complete with the same visible outcomes after refactoring the route-facing seams.

### Tests for User Story 1

- [ ] T008 [P] [US1] Add parity coverage for startup route resolution in `src/lib/route-session.ts`
- [ ] T009 [P] [US1] Add targeted coverage for extracted route-facing pure logic in `src/features/auth/login/utils.ts`, `src/features/auth/unlock/hooks/index.ts`, and `src/features/vault/utils.ts`
- [ ] T010 [US1] Record manual smoke validation steps and expected outcomes for login, unlock, vault, and spotlight flows in `specs/001-frontend-refactor/quickstart.md`

### Implementation for User Story 1

- [ ] T011 [US1] Extract a login page composition component from `src/routes/index.tsx` into `src/features/auth/login/components/`
- [ ] T012 [US1] Extract an unlock page composition component from `src/routes/unlock.tsx` into `src/features/auth/unlock/components/`
- [ ] T013 [US1] Extract vault page sections from `src/routes/vault.tsx` into `src/features/vault/components/`
- [ ] T014 [US1] Reduce `src/routes/index.tsx` to route guard and thin page composition using `src/features/auth/login/components/`
- [ ] T015 [US1] Reduce `src/routes/unlock.tsx` to route guard and thin page composition using `src/features/auth/unlock/components/`
- [ ] T016 [US1] Reduce `src/routes/vault.tsx` to route guard and thin page composition using `src/features/vault/components/`
- [ ] T017 [US1] Preserve startup/session synchronization behavior while aligning imports in `src/main.tsx` and `src/lib/route-session.ts`
- [ ] T018 [US1] Update feature barrel exports in `src/features/auth/login/index.ts`, `src/features/auth/unlock/index.ts`, and `src/features/vault/index.ts` to support the thinner route entrypoints

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Find Frontend Code In Predictable Locations (Priority: P2)

**Goal**: Make file ownership predictable by moving feature-specific code into clearer module locations and tightening shared boundaries.

**Independent Test**: Confirm a developer can locate representative route-facing components, hooks, shared helpers, and feature entry points from the new structure without prior history.

### Tests for User Story 2

- [ ] T019 [P] [US2] Add focused coverage for any extracted feature-local selectors or pure helpers in `src/features/auth/login/utils.ts`, `src/features/vault/utils.ts`, and `src/features/spotlight/utils.ts`
- [ ] T020 [US2] Record a structure validation checklist for locating components, hooks, and utilities in `specs/001-frontend-refactor/quickstart.md`

### Implementation for User Story 2

- [ ] T021 [US2] Reorganize login feature internals into clearer component, hook, and helper boundaries under `src/features/auth/login/`
- [ ] T022 [US2] Reorganize unlock feature internals into clearer component, hook, and helper boundaries under `src/features/auth/unlock/`
- [ ] T023 [US2] Reorganize vault feature internals into clearer component, hook, and helper boundaries under `src/features/vault/`
- [ ] T024 [US2] Reorganize spotlight feature internals into clearer component, hook, and helper boundaries under `src/features/spotlight/`
- [ ] T025 [US2] Move or normalize truly shared non-feature helpers between `src/lib/utils.ts`, `src/features/auth/shared/utils.ts`, `src/features/vault/utils.ts`, and `src/features/spotlight/utils.ts`
- [ ] T026 [US2] Tighten public entry points and import paths in `src/features/auth/index.ts`, `src/features/auth/login/index.ts`, `src/features/auth/unlock/index.ts`, `src/features/spotlight/index.ts`, and `src/features/vault/index.ts`

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Reduce Duplication And Oversized Files (Priority: P3)

**Goal**: Split large orchestration files and consolidate repeated logic so future changes are easier to review and maintain.

**Independent Test**: Confirm the selected high-risk files are broken into focused units, repeated logic is centralized, and the affected flows still pass the parity checks from prior phases.

### Tests for User Story 3

- [ ] T027 [P] [US3] Add targeted coverage for extracted vault filtering, sorting, and folder-tree helpers in `src/features/vault/utils.ts` and related feature-local modules under `src/features/vault/`
- [ ] T028 [P] [US3] Add targeted coverage for extracted login and unlock branching helpers under `src/features/auth/login/` and `src/features/auth/unlock/`
- [ ] T029 [US3] Extend manual parity validation notes for stale detail loading, unlock method selection, and login branching in `specs/001-frontend-refactor/quickstart.md`

### Implementation for User Story 3

- [ ] T030 [US3] Split `src/features/vault/hooks/use-vault-page-model.ts` into focused feature-local modules under `src/features/vault/hooks/` and `src/features/vault/`
- [ ] T031 [US3] Split `src/features/auth/login/hooks/use-login-flow.ts` into focused feature-local modules under `src/features/auth/login/hooks/` and `src/features/auth/login/`
- [ ] T032 [US3] Split `src/features/auth/unlock/hooks/use-unlock-flow.ts` into focused feature-local modules under `src/features/auth/unlock/hooks/` and `src/features/auth/unlock/`
- [ ] T033 [US3] Consolidate duplicated cross-feature error and normalization helpers between `src/features/auth/shared/utils.ts`, `src/features/vault/utils.ts`, `src/features/auth/login/utils.ts`, and `src/lib/utils.ts`
- [ ] T034 [US3] Reduce accidental coupling and narrow exports after the splits in `src/features/auth/login/index.ts`, `src/features/auth/unlock/index.ts`, `src/features/vault/index.ts`, and `src/features/spotlight/index.ts`

**Checkpoint**: All user stories should now be independently functional

---

## Final Phase: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, cleanup, and documentation across all stories

- [ ] T035 [P] Run the full frontend validation pass for behavior parity using `specs/001-frontend-refactor/quickstart.md`
- [ ] T036 [P] Review startup, navigation, and interactive performance for regressions in `src/main.tsx`, `src/lib/route-session.ts`, and `src/routes/vault.tsx`
- [ ] T037 Clean up stale imports, dead barrels, and obsolete helper paths across `src/routes/`, `src/features/`, and `src/lib/`
- [ ] T038 Update implementation notes and final validation evidence in `specs/001-frontend-refactor/contracts/ui-contract.md` and `specs/001-frontend-refactor/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel if enough contributors are available
  - Recommended execution order is P1 → P2 → P3 because each story reduces risk for the next
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Starts after Foundational and establishes the safe route-entry seams used by later work
- **User Story 2 (P2)**: Starts after Foundational, but is easier after User Story 1 because route ownership is clearer
- **User Story 3 (P3)**: Starts after Foundational, but should follow User Stories 1 and 2 because large-hook splits depend on the stabilized structure

### Within Each User Story

- Validation tasks MUST be completed before the corresponding refactor is considered done
- Route-facing seams before deeper hook/module splits
- Feature-local reorganization before cross-feature helper consolidation
- Core refactor before barrel export tightening
- Story complete before moving to the next highest-risk slice

### Parallel Opportunities

- T003 can run in parallel with T001-T002
- T005-T006 can run in parallel after T004 starts
- T008-T010 can run in parallel inside User Story 1
- T011-T013 can run in parallel by route/feature area once the validation baseline exists
- T019-T020 can run in parallel inside User Story 2
- T021-T024 can run in parallel by feature area if ownership boundaries stay independent
- T027-T029 can run in parallel inside User Story 3
- T030-T032 can run in parallel by feature area after the structure is stabilized
- T035-T036 can run in parallel during polish

---

## Parallel Example: User Story 1

```bash
# Launch User Story 1 validation tasks together:
Task: "Add parity coverage for startup route resolution in src/lib/route-session.ts"
Task: "Add targeted coverage for extracted route-facing pure logic in src/features/auth/login/utils.ts, src/features/auth/unlock/hooks/index.ts, and src/features/vault/utils.ts"
Task: "Record manual smoke validation steps in specs/001-frontend-refactor/quickstart.md"

# Launch User Story 1 route extraction tasks together:
Task: "Extract a login page composition component from src/routes/index.tsx into src/features/auth/login/components/"
Task: "Extract an unlock page composition component from src/routes/unlock.tsx into src/features/auth/unlock/components/"
Task: "Extract vault page sections from src/routes/vault.tsx into src/features/vault/components/"
```

## Parallel Example: User Story 2

```bash
# Launch User Story 2 feature reorganization tasks together:
Task: "Reorganize login feature internals under src/features/auth/login/"
Task: "Reorganize unlock feature internals under src/features/auth/unlock/"
Task: "Reorganize vault feature internals under src/features/vault/"
Task: "Reorganize spotlight feature internals under src/features/spotlight/"
```

## Parallel Example: User Story 3

```bash
# Launch User Story 3 large-hook split tasks together:
Task: "Split src/features/vault/hooks/use-vault-page-model.ts into focused feature-local modules"
Task: "Split src/features/auth/login/hooks/use-login-flow.ts into focused feature-local modules"
Task: "Split src/features/auth/unlock/hooks/use-unlock-flow.ts into focused feature-local modules"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Confirm startup routing, login, unlock, vault, and spotlight still behave the same
5. Proceed only after parity is demonstrated

### Incremental Delivery

1. Complete Setup + Foundational → foundation ready
2. Deliver User Story 1 → validate route and behavior parity
3. Deliver User Story 2 → validate discoverable file ownership and unchanged flows
4. Deliver User Story 3 → validate reduced duplication and split large files without regressions
5. Finish with polish and full parity validation

### Parallel Team Strategy

With multiple developers:

1. One developer owns foundational validation seams
2. After foundation is ready:
   - Developer A: route thinning and route-owned composition
   - Developer B: feature directory and ownership cleanup
   - Developer C: large-hook decomposition and helper consolidation
3. Rejoin for shared validation and export cleanup

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] labels map each task to a single user story for traceability
- Every user story includes validation work plus implementation work
- File paths are intentionally specific enough for immediate execution
- Suggested MVP scope: Phase 3 / User Story 1 only
- All tasks follow the required checklist format
