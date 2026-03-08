# UI Contract: Frontend Maintainability Refactor

## Purpose

This contract defines the externally visible frontend behaviors that MUST remain unchanged while
internal code is reorganized.

## Stable Route Contract

- `/` remains the login route.
- `/unlock` remains the unlock route.
- `/vault` remains the vault route.
- `resolveSessionRoute()` remains the central session-to-route decision seam.
- `src/main.tsx` remains the startup/session sync entrypoint.
- Route guards MUST continue redirecting users to the same destination for the same session state.

## Stable Flow Contract

### Login Flow
- Restored session hints for service address and email remain available.
- Validation messages appear before submission when required fields are incomplete or invalid.
- Two-factor challenges continue to appear when required.
- Successful login continues into unlock-or-sync behavior that ends at the vault view.

### Unlock Flow
- Users without a restorable session are directed back to login.
- Available unlock methods continue to be presented based on the restored session state.
- Successful unlock continues to the vault view.
- Logout from the unlock surface returns the user to login.

### Vault Flow
- Locked sessions continue to redirect away from the vault surface.
- Ready, loading, and error states remain available.
- Search, filtering, sorting, folder selection, detail loading, lock, logout, and settings access
  continue to work.
- Detail loading continues to ignore stale selections when a newer selection supersedes an older
  request.

### Spotlight Flow
- Search input, result selection, action triggers, and dismiss behavior remain consistent where
  affected by shared helper extraction.

## Structural Contract

- Route entry modules may become thinner, but route paths and route-facing outcomes MUST not change.
- Feature modules may split internally, but existing feature behavior and reachable actions MUST not
  change.
- Shared modules may only contain logic with demonstrated cross-feature value.

## Validation Evidence Log

| Phase | Scope | Automated Evidence | Manual Evidence | Notes |
| --- | --- | --- | --- | --- |
| Planned | Startup routing + `/` + `/unlock` + `/vault` + spotlight parity | `npm run build`, `npm run biome:check` | Run the quickstart parity checklist | Update with final implementation results |
| Final | Full `001-frontend-refactor` scope | `npm run build` ✅, `npm run biome:check` ✅ | Quickstart checklist recorded for manual execution | Route files thinned, startup seam preserved, hook/helper splits completed |
