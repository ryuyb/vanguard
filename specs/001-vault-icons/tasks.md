# Tasks: Vault Icons

**Input**: Design documents from `/specs/001-vault-icons/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Include tasks for the automated tests and validation work required by the constitution. Every user story MUST include the tasks needed to prove correctness, and any performance or UX validation tasks required by the spec.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm the implementation surface and validation commands before feature work begins

- [x] T001 Inspect and document the existing frontend validation commands in `package.json` and `pnpm` workflow for this feature
- [x] T002 Create the feature task breakdown artifact in `specs/001-vault-icons/tasks.md`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core icon-resolution and shared presentation primitives that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T003 Update `src/bindings.ts` with the smallest DTO change needed to expose website-derived icon input for cipher list rows if current row data is insufficient
- [x] T004 [P] Add website-to-icon target normalization helpers and fallback selection utilities in `src/features/vault/utils.ts`
- [x] T005 [P] Create a reusable vault icon presentation component for loaded, loading, and fallback states in `src/features/vault/components/cipher-icon.tsx`
- [x] T006 Export the shared vault icon presentation component from `src/features/vault/components/index.ts`

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Browse vault items with recognizable icons (Priority: P1) 🎯 MVP

**Goal**: Show website icons in cipher list rows with visibility-based lazy loading and deterministic fallback visuals

**Independent Test**: Open a vault with website-based entries, scroll through the cipher list, and confirm visible rows progressively show icons without blocking list interaction or showing broken image states.

### Validation for User Story 1

- [x] T007 [US1] Add manual validation steps for list icon rendering and lazy loading to `specs/001-vault-icons/quickstart.md`

### Implementation for User Story 1

- [x] T008 [US1] Thread any new icon-related list fields through vault list types and selectors in `src/features/vault/types.ts` and `src/features/vault/hooks/use-vault-page-model.ts`
- [x] T009 [US1] Integrate the shared vault icon component into cipher list rows in `src/features/vault/components/cipher-row.tsx`
- [x] T010 [US1] Add visibility-based lazy loading state for cipher row icons in `src/features/vault/components/vault-page.tsx`
- [x] T011 [US1] Refine row spacing and fallback presentation to preserve existing list hierarchy in `src/features/vault/components/cipher-row.tsx`
- [x] T012 [US1] Run the frontend validation commands for the touched list-row files and record the results in `specs/001-vault-icons/quickstart.md`

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Confirm item identity in details view (Priority: P2)

**Goal**: Show a matching website icon in the cipher detail view using the same selection and fallback rules as the list

**Independent Test**: Open a cipher detail view for an item with a usable website address and confirm the detail panel shows the corresponding icon, while unsupported items show the same fallback outcome used in the list.

### Validation for User Story 2

- [x] T013 [US2] Add manual validation steps for detail icon rendering and cross-surface consistency to `specs/001-vault-icons/quickstart.md`

### Implementation for User Story 2

- [x] T014 [US2] Derive detail-view icon input from selected cipher data in `src/features/vault/hooks/use-cipher-detail-selection.ts` and `src/features/vault/utils.ts`
- [x] T015 [US2] Integrate the shared vault icon component into the detail header or summary presentation in `src/features/vault/components/cipher-detail-panel.tsx`
- [x] T016 [US2] Align detail fallback visuals and accessibility labeling with the list behavior in `src/features/vault/components/cipher-detail-panel.tsx` and `src/features/vault/components/cipher-icon.tsx`
- [x] T017 [US2] Run the frontend validation commands for the touched detail-view files and record the results in `specs/001-vault-icons/quickstart.md`

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Experience stable icon behavior across sessions (Priority: P3)

**Goal**: Ensure slow, missing, or failed icon responses never degrade vault usability and that list scrolling remains responsive while icons load

**Independent Test**: Browse the vault under slow or unavailable icon conditions and confirm the list and detail views remain readable, interactive, and free of broken image states while fallback visuals appear consistently.

### Validation for User Story 3

- [x] T018 [US3] Add degraded-network and scrolling validation steps to `specs/001-vault-icons/quickstart.md`

### Implementation for User Story 3

- [x] T019 [US3] Harden the shared vault icon component against invalid image responses and loading failures in `src/features/vault/components/cipher-icon.tsx`
- [x] T020 [US3] Ensure list lazy-loading state does not trigger eager icon requests during rapid scrolling in `src/features/vault/components/vault-page.tsx`
- [x] T021 [US3] Ensure detail and list surfaces resolve to fallback visuals without layout breakage under failure conditions in `src/features/vault/components/cipher-row.tsx` and `src/features/vault/components/cipher-detail-panel.tsx`
- [x] T022 [US3] Run the frontend validation commands and execute the degraded-behavior manual checks from `specs/001-vault-icons/quickstart.md`, then record the results in `specs/001-vault-icons/quickstart.md`

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final consistency, documentation, and release-readiness checks across all stories

- [x] T023 [P] Update the implementation summary and final validation evidence in `specs/001-vault-icons/plan.md` and `specs/001-vault-icons/quickstart.md`
- [x] T024 Review `src/features/vault/components/cipher-row.tsx`, `src/features/vault/components/cipher-detail-panel.tsx`, `src/features/vault/components/cipher-icon.tsx`, and `src/features/vault/utils.ts` for duplicate logic and remove unnecessary complexity

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational completion
- **User Story 2 (Phase 4)**: Depends on Foundational completion and may reuse the shared icon component from US1 while remaining independently testable
- **User Story 3 (Phase 5)**: Depends on Foundational completion and builds on the icon behavior delivered in US1 and US2
- **Polish (Phase 6)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Starts after Foundational and delivers the MVP list-icon experience
- **User Story 2 (P2)**: Starts after Foundational and depends on the shared icon resolution/presentation primitives established earlier
- **User Story 3 (P3)**: Starts after US1 and US2 icon rendering exists, because it hardens degraded and performance-sensitive behavior across both surfaces

### Within Each User Story

- Validation steps must be updated before the story is considered complete
- Shared data flow must be wired before rendering integration
- Rendering integration must be complete before final validation recording
- Manual and automated validation evidence must be recorded in `specs/001-vault-icons/quickstart.md`

### Parallel Opportunities

- T004 and T005 can run in parallel after T003 if the DTO decision is already known
- T023 can run in parallel with final cleanup once all story work is complete
- If staffed, one developer can refine list integration while another prepares detail integration after Foundational phase completion, but US3 should remain last because it validates and hardens shared behavior

---

## Parallel Example: User Story 1

```bash
Task: "Thread any new icon-related list fields through vault list types and selectors in src/features/vault/types.ts and src/features/vault/hooks/use-vault-page-model.ts"
Task: "Integrate the shared vault icon component into cipher list rows in src/features/vault/components/cipher-row.tsx"
```

---

## Parallel Example: Foundational Phase

```bash
Task: "Add website-to-icon target normalization helpers and fallback selection utilities in src/features/vault/utils.ts"
Task: "Create a reusable vault icon presentation component for loaded, loading, and fallback states in src/features/vault/components/cipher-icon.tsx"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test list icon rendering and lazy loading independently
5. Demo the list experience before expanding to detail and degraded-state hardening

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Validate independently → MVP available
3. Add User Story 2 → Validate independently → Detail consistency available
4. Add User Story 3 → Validate independently → Resilience and performance hardening complete
5. Finish with Phase 6 polish and final validation evidence

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. After Foundational:
   - Developer A: User Story 1 list integration
   - Developer B: User Story 2 detail integration
3. Team rejoins for User Story 3 degraded-state hardening and final polish

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is scoped to remain independently testable
- No new third-party dependency should be added unless implementation proves it is necessary
- Avoid per-row detail fetching as a shortcut for list icons because it threatens scrolling performance
- All tasks above follow the required checklist format with checkbox, task ID, story label where required, and file paths
