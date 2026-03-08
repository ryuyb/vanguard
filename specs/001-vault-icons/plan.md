# Implementation Plan: Vault Icons

**Branch**: `001-vault-icons` | **Date**: 2026-03-08 | **Spec**: [/Users/ryuyb/Developer/vanguard/specs/001-vault-icons/spec.md](/Users/ryuyb/Developer/vanguard/specs/001-vault-icons/spec.md)
**Input**: Feature specification from `/specs/001-vault-icons/spec.md`

## Summary

Add website icon presentation to the vault cipher list and cipher detail view by deriving icon targets from existing cipher website data, rendering icons with deterministic fallbacks, and using visibility-based lazy loading for list rows so scrolling remains responsive.

## Technical Context

**Language/Version**: TypeScript 5.x with React 19 frontend, running in a Tauri v2 desktop shell
**Primary Dependencies**: React, TanStack Router, Tailwind CSS, Lucide React, existing Tauri command/bindings layer
**Storage**: Existing vault data retrieved through current Tauri command bindings; no new persistent storage required
**Testing**: Existing repository lint/build validation plus feature-specific manual validation because no active frontend UI test harness was found
**Target Platform**: Tauri desktop application on macOS
**Project Type**: Desktop application with React frontend
**Performance Goals**: Website icons must load without noticeably blocking vault browsing; list scrolling must remain smooth for at least 100 visible-test items while icons resolve lazily
**Constraints**: No broken image states; list icons must lazy load by visibility; avoid adding new third-party dependencies unless implementation proves they are necessary; preserve existing vault visual hierarchy and interaction patterns
**Scale/Scope**: Change is limited to vault cipher list and cipher detail presentation, likely centered on `src/features/vault/components/cipher-row.tsx`, `src/features/vault/components/cipher-detail-panel.tsx`, `src/features/vault/components/vault-page.tsx`, `src/features/vault/hooks/use-vault-page-model.ts`, `src/features/vault/utils.ts`, and any minimal binding/DTO surfaces needed to expose a primary website target for list rows

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Code Quality Gate**: Pass. Smallest viable design is a vault-local icon helper/presentation path rather than a new shared subsystem. Expected files are the vault row/detail components, vault utilities, and only the minimal supporting data surfaces required to provide website-derived icon targets in the list. No new dependency, abstraction, or architectural layer is justified at planning time.
- **Testing Gate**: Pass with documented validation strategy. Automated evidence will use the repository’s existing checks for the touched frontend area. Because research found no established frontend UI test harness, completion also requires manual validation for list icon rendering, detail icon rendering, fallback behavior, degraded network behavior, and list scrolling responsiveness.
- **UX Consistency Gate**: Pass. User-facing work is limited to adding an icon slot to existing cipher rows and the detail presentation while preserving current title/username hierarchy, fallback conventions, loading behavior, and accessibility. Copy changes are not required beyond any accessibility labeling for fallback or icon imagery.
- **Performance Gate**: Pass. The plan explicitly limits lazy loading to the list surface, rejects eager loading of all icons, and rejects per-row full-detail fetch behavior as a default strategy. Validation will confirm that a long list remains responsive while icons resolve.

## Project Structure

### Documentation (this feature)

```text
specs/001-vault-icons/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── vault-icon-ui-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── bindings.ts
├── routes/
│   └── vault.tsx
├── components/
│   └── ui/
└── features/
    └── vault/
        ├── components/
        │   ├── cipher-row.tsx
        │   ├── cipher-detail-panel.tsx
        │   ├── vault-page.tsx
        │   └── index.ts
        ├── hooks/
        │   ├── use-vault-page-model.ts
        │   └── use-cipher-detail-selection.ts
        ├── types.ts
        └── utils.ts
```

**Structure Decision**: Use the existing single-project desktop-app structure and keep all feature behavior inside the current vault frontend modules. Only touch shared files such as `src/bindings.ts` if the list surface truly needs a minimal DTO adjustment to expose the website-derived icon target without introducing expensive per-row detail loading.

## Phase 0: Research Summary

Research resolved the planning unknowns with these conclusions:
- The feature belongs in the current React + TypeScript vault UI inside Tauri.
- There is no existing remote image, icon-server, or virtualized list pattern to reuse.
- The smallest viable design is vault-local UI logic plus deterministic fallback visuals.
- The detail surface already has URI-oriented data available, while the list surface may require a minimal data-shape adjustment if it cannot derive an icon target from existing row fields.
- Validation must rely on existing automated checks and explicit manual acceptance steps.

## Phase 1: Design

### Data Model

See `/Users/ryuyb/Developer/vanguard/specs/001-vault-icons/data-model.md`.

### Contracts

See `/Users/ryuyb/Developer/vanguard/specs/001-vault-icons/contracts/vault-icon-ui-contract.md`.

### Implementation Summary

Implemented in the existing vault frontend with a vault-scoped icon resolution and presentation flow:
- `src/features/vault/utils.ts` derives icon hosts and icon URLs from cipher website URIs.
- `src/features/vault/components/cipher-icon.tsx` centralizes icon rendering, fallback visuals, and load-failure handling.
- `src/features/vault/components/cipher-row.tsx` renders list icons.
- `src/features/vault/components/vault-page.tsx` lazy-loads list icons based on row visibility.
- `src/features/vault/components/cipher-detail-panel.tsx` renders matching detail icons.

### Implementation Approach

1. Add a vault-scoped icon resolution utility that derives a stable icon request target from the cipher’s existing website information and maps unsupported cases to fallback behavior.
2. Add a small vault icon presentation element for rendering either a loaded website icon or a deterministic fallback visual.
3. Integrate that presentation into `cipher-row.tsx` for list rows with visibility-based lazy loading.
4. Integrate the same presentation outcome into `cipher-detail-panel.tsx` so detail and list stay consistent.
5. If list-row data lacks enough website information, introduce the smallest possible DTO/binding change to expose a primary website target for list rows rather than fetching full detail per visible row.
6. Validate that layout, accessibility, and scrolling remain consistent with the current vault experience.

### Validation Plan

- Run the repository’s existing frontend validation commands for formatting/lint/build as applicable to the touched area.
- Manually verify the scenarios documented in `quickstart.md`:
  - eligible list rows show icons
  - list icons lazy load during scrolling
  - detail panel shows matching icon outcome
  - unsupported/failed cases show fallback visuals
  - slow/unavailable icon responses do not block vault usage

## Post-Design Constitution Check

- **Code Quality Gate**: Still passes. The design remains local to the vault feature and only permits a minimal DTO adjustment if list rendering truly requires it.
- **Testing Gate**: Still passes. Validation remains explicit and proportional to repository capabilities.
- **UX Consistency Gate**: Still passes. Design preserves existing vault hierarchy and fallback behavior.
- **Performance Gate**: Still passes. The plan continues to require visibility-based lazy loading and avoids eager network work for the full list.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
