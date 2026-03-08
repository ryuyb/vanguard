# Research: Frontend Maintainability Refactor

## Decision 1: Keep the current top-level frontend shape and enforce stricter feature ownership

**Decision**: Keep `src/routes`, `src/features`, `src/components/ui`, and `src/lib` as the
main top-level structure. Strengthen the rule that feature-specific code stays inside its
feature, shared UI stays in `src/components/ui`, and truly cross-feature runtime helpers stay
in `src/lib`.

**Rationale**: The repository already follows a partly feature-based structure, especially in
`src/features/auth`, `src/features/vault`, and `src/features/spotlight`. The maintainability
problem is not a missing top-level pattern; it is uneven enforcement of boundaries and a few
large orchestration files. Preserving the current top-level layout avoids unnecessary churn
while improving discoverability.

**Alternatives considered**:
- Introduce many new global folders by type (`hooks`, `utils`, `services`) at the top level.
  Rejected because that weakens ownership and increases cross-feature sprawl.
- Move route files into each feature. Rejected because current file-based routing works well
  with stable ownership in `src/routes`.
- Introduce a more elaborate layered taxonomy. Rejected because it adds ceremony without clear
  benefit for the current project size.

## Decision 2: Refactor in vertical slices that preserve public seams

**Decision**: Refactor incrementally in the following order:
1. Preserve current public seams and route paths.
2. Thin route files so they mostly declare routing, redirects, and page composition.
3. Split large orchestration hooks into smaller feature-local modules.
4. Normalize feature-local command access and shared helper boundaries.
5. Revisit barrel exports only after the structure settles.

**Rationale**: The highest regression risk is in boot routing, unlock/login branching, and vault
screen behavior. Incremental vertical slices make review easier and preserve user-visible
behavior while code moves underneath stable interfaces.

**Alternatives considered**:
- Big-bang folder rewrite first. Rejected because it produces noisy diffs and raises regression
  risk.
- Rename files and directories before improving boundaries. Rejected because naming churn alone
  gives little safety or maintainability benefit.
- Start by rewriting shared UI primitives. Rejected because shared UI is already centralized and
  is not the main maintainability bottleneck.

## Decision 3: Use characterization-based validation rather than waiting for a full test suite

**Decision**: Validate this refactor with a layered strategy:
- a behavior inventory for critical flows,
- route and flow smoke checks,
- targeted automated tests around extracted pure logic,
- manual verification for Tauri-specific startup and window behavior.

**Rationale**: The current project does not expose a large existing frontend test suite. The most
practical safety net is to protect the seams that are being reorganized: route resolution,
login/unlock branching, vault filtering/detail loading, and spotlight interactions. This matches
constitution requirements without blocking the refactor on a full testing overhaul.

**Alternatives considered**:
- Rely on manual QA only. Rejected because it is too fragile during repeated file moves.
- Delay the refactor until comprehensive tests exist. Rejected because it prevents incremental
  maintainability work.
- Add broad snapshot coverage first. Rejected because it is likely to be noisy during active
  reorganization.

## Decision 4: Preserve startup and route behavior as the primary performance requirement

**Decision**: Treat startup routing, navigation correctness, and interactive vault responsiveness
as the key performance-sensitive areas. Do not change route paths, boot sequencing, or startup
command count unless explicitly justified.

**Rationale**: For this desktop app, the most user-visible performance regressions would come from
slower initial route resolution, duplicated startup calls, or less responsive vault interactions.
The refactor should prioritize no-regression behavior rather than introducing new code-splitting or
state architecture.

**Alternatives considered**:
- Add route-level lazy loading during the refactor. Rejected because it changes startup behavior
  and adds complexity during a behavior-preserving change.
- Centralize all routing/session logic into a new global store. Rejected because it broadens scope
  and increases risk around boot behavior.
- Optimize render performance before structural cleanup. Rejected because there is no evidence that
  performance is currently limited by rendering architecture.

## Repository-specific findings

### Current frontend seams worth preserving
- `src/main.tsx` owns app startup and session-to-route synchronization.
- `src/lib/route-session.ts` owns the route decision contract between `/`, `/unlock`, and `/vault`.
- `src/routes/index.tsx`, `src/routes/unlock.tsx`, and `src/routes/vault.tsx` own route guards and
  current page composition.

### Current maintainability hotspots
- `src/features/vault/hooks/use-vault-page-model.ts` mixes loading, navigation fallback, filtering,
  sorting, folder tree state, detail loading, and header actions.
- `src/features/auth/login/hooks/use-login-flow.ts` mixes session restore, input validation, login,
  two-factor branching, unlock flow, sync flow, and user feedback.
- `src/features/auth/unlock/hooks/use-unlock-flow.ts` mixes restore probing, unlock capability
  discovery, unlock actions, logout, and feedback state.
- `src/routes/vault.tsx`, `src/routes/index.tsx`, and `src/routes/unlock.tsx` still contain large
  page composition blocks that can be made thinner without changing behavior.

### Validation focus areas
- Boot route resolution from `src/main.tsx` and `src/lib/route-session.ts`.
- Login flow including restore hint, validation, two-factor branch, unlock-after-login, and sync
  outcomes.
- Unlock flow including needs-login redirect, biometric/PIN/password options, and unlocked redirect.
- Vault flow including locked redirect, ready/error states, filtering, sorting, folder selection,
  and stale detail request handling.
- Spotlight flow parity after any shared helper extraction in nearby modules.
