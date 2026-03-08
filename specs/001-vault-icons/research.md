# Research: Vault Icons

## Decision 1: Implement the feature in the existing React + TypeScript vault frontend inside the Tauri desktop app
- **Decision**: Plan the feature as a frontend change in the existing React 19 + TypeScript + Vite vault feature, running inside the current Tauri desktop shell.
- **Rationale**: The vault list and detail experiences are already implemented in the frontend, and the feature is purely user-facing display behavior. This matches the current architecture and avoids introducing a new service or cross-layer abstraction without evidence it is needed.
- **Alternatives considered**:
  - Add a new native-side icon retrieval layer in Tauri: rejected for the initial plan because it increases scope and adds an unnecessary cross-layer dependency before proving the UI need.
  - Add a separate shared icon subsystem for the entire app: rejected because only the vault surfaces currently need this behavior.

## Decision 2: Use vault-local UI changes as the smallest viable design
- **Decision**: Limit the planned code changes to the vault feature and its immediate helpers, with the smallest likely set centered on `src/features/vault/components/cipher-row.tsx`, `src/features/vault/components/cipher-detail-panel.tsx`, and `src/features/vault/utils.ts`.
- **Rationale**: These files already own the list row, detail panel, and small formatting logic. The constitution requires minimal, intentional changes and reuse of established patterns before introducing abstraction.
- **Alternatives considered**:
  - Build a repository-wide image framework: rejected as over-engineering for a single feature.
  - Add a new shared domain service layer just for icon resolution: rejected unless implementation later proves the logic is reused broadly enough to justify it.

## Decision 3: Treat cipher website data as the source of truth for icon selection
- **Decision**: Derive icon lookup targets from the existing website information already associated with each cipher, following the same primary-website selection rule the product already uses for display.
- **Rationale**: The specification requires icons to come from existing cipher website data and not from a new user-managed field. Reusing the existing website source keeps list and detail behavior consistent and traceable.
- **Alternatives considered**:
  - Let users enter a custom icon URL: rejected because it expands scope and violates the requirement that icons come from existing cipher website data.
  - Maintain a separate icon mapping store: rejected because it adds data maintenance and synchronization complexity with no user-facing need.

## Decision 4: Use remote icon loading with deterministic fallback visuals
- **Decision**: Plan for a simple icon presentation model: load the remote website icon when possible, and otherwise display a deterministic fallback visual that matches the current vault UI style.
- **Rationale**: The codebase already uses straightforward fallbacks such as initials and Lucide icons rather than heavy placeholder systems. This satisfies the UX consistency requirement and keeps failure states simple and readable.
- **Alternatives considered**:
  - Use broken-image browser fallback behavior: rejected because the specification explicitly requires graceful failure and no broken image state.
  - Add animated placeholders or progressive image effects: rejected because there is no existing pattern requiring that level of visual treatment.

## Decision 5: Use visibility-based lazy loading only for the cipher list
- **Decision**: Implement lazy loading for list icons based on row visibility or near-visibility, while allowing the detail icon to load when the selected cipher detail renders.
- **Rationale**: The specification explicitly limits lazy loading to the vault list. The current list is rendered normally without virtualization, so a lightweight visibility-based approach fits the existing rendering model and avoids introducing a new windowing dependency.
- **Alternatives considered**:
  - Add list virtualization: rejected because the repo does not currently use a virtualization library and this feature does not justify a large structural change.
  - Eagerly load all list icons at once: rejected because it conflicts with the requirement for lazy loading and creates unnecessary network cost during scrolling.

## Decision 6: Avoid adding new third-party frontend dependencies in the initial implementation
- **Decision**: Do not plan on adding an image-loading library, virtualization package, or broad icon management dependency for this feature.
- **Rationale**: The current repository does not show an established pattern for remote images or virtualized lists, and the constitution requires justification for every new dependency. The feature can be implemented with platform primitives and existing project libraries.
- **Alternatives considered**:
  - Add a lazy-image package: rejected because browser and React primitives should be enough.
  - Add a virtualization package: rejected because it solves a larger problem than the requested feature.

## Decision 7: Treat automated validation as lint plus focused manual acceptance for now
- **Decision**: Plan validation around existing repository checks and feature-specific manual verification, with manual acceptance covering list icon rendering, detail icon rendering, fallback behavior, and scrolling responsiveness.
- **Rationale**: Research found no active frontend test harness such as Vitest, Jest, Testing Library, or Playwright in this repository. The constitution still requires proof of behavior, so the plan must define explicit manual validation steps and run the available automated checks.
- **Alternatives considered**:
  - Introduce a brand-new UI test framework as part of this feature: rejected because it would significantly expand scope beyond the requested behavior.
  - Ship without validation evidence: rejected because it would violate the testing gate.

## Decision 8: Treat list icon data availability as a design constraint to resolve during implementation planning
- **Decision**: Record that list icon rendering depends on whether the list payload already contains enough website information to derive icon targets directly. If it does not, the implementation may need a targeted data-contract change rather than per-row detail fetches.
- **Rationale**: Research found that detail data includes URI information, while the current list DTO appears to expose only limited row fields. The plan must keep the smallest viable design while avoiding a performance-hostile strategy such as fetching full detail data for every visible row.
- **Alternatives considered**:
  - Fetch full cipher detail for each list row on demand: rejected as likely too expensive and too complex for scrolling performance.
  - Show icons only in detail and not in list: rejected because the approved specification requires both list and detail icons.
