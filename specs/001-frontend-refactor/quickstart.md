# Quickstart: Frontend Maintainability Refactor

## Goal

Refactor the frontend for maintainability without changing user-visible behavior,
while making file ownership, reuse boundaries, and future edits easier to manage.

## Preconditions

- Start from branch `001-frontend-refactor`.
- Keep the current route paths and startup behavior unchanged.
- Treat generated files and framework-required entry points as fixed unless a move is clearly safe.

## Behavior Inventory

### Startup Routing
- Launch still resolves the initial route through `resolveSessionRoute()`.
- Window refocus still re-checks session state and redirects only when the target route changes.
- Route entrypoints remain `/`, `/unlock`, and `/vault`.

### Login
- Restored base URL and email hints continue to prefill the login surface when available.
- Field validation still blocks submit for missing server URL, invalid URL protocol, invalid email, and missing master password.
- Two-factor challenge selection, token entry, and email-code sending behavior remain unchanged.
- Successful login still follows the same unlock-or-sync path before navigating to the vault.

### Unlock
- Unlock still restores session state before rendering the locked surface.
- `needsLogin` still sends the user back to the login route.
- PIN, biometric, and master-password unlock branches remain available based on restored capability state.
- Unlock-surface logout still clears session state and returns to `/`.

### Vault
- Vault still redirects away when the session is not eligible for `/vault`.
- Loading, error, and ready states remain intact.
- Header search, inline list search, type filtering, sorting, folder selection, detail loading, settings, lock, and logout behaviors remain intact.
- Stale cipher-detail responses continue to be ignored when a newer selection supersedes an older request.

### Spotlight
- Spotlight item shaping, search matching, result visibility, detail selection, actions, and dismiss behavior remain unchanged.
- Shared helper cleanup must not change spotlight logging or error text behavior.

## Implementation Sequence

1. Review current route entry files and feature hooks to define the smallest safe vertical slice.
2. Extract route-local composition into feature-owned page or section modules while keeping route
   guards and navigation outcomes unchanged.
3. Split oversized feature hooks by responsibility, preserving the existing route-facing contract
   until each slice is stable.
4. Consolidate duplicated feature logic into shared modules only when reuse is proven.
5. Tighten shared boundaries so feature-specific helpers do not depend on unrelated features.
6. Repeat for the next slice only after validation evidence is updated.

## Validation Checklist

For each slice, verify:
- Startup route resolution still lands on the same route for the same session state.
- Login, unlock, vault, and spotlight flows still complete with the same visible outcomes.
- Loading, empty, error, locked, and destructive states remain intact where affected.
- No additional startup or navigation regressions are introduced.
- Any extracted pure logic has targeted automated coverage when the surrounding layer supports it.

## Manual Parity Checklist

### Startup + Routing
- [ ] Launch the app with no session and confirm startup lands on `/`.
- [ ] Launch the app with a locked restorable session and confirm startup lands on `/unlock`.
- [ ] Launch the app with an unlocked session and confirm startup lands on `/vault`.
- [ ] Refocus the window from each route and confirm session-sync navigation remains unchanged.

### Login
- [ ] Confirm restored service address and email hints still appear when available.
- [ ] Confirm invalid or incomplete login fields show the same validation outcomes before submit.
- [ ] Confirm a two-factor-required account still shows provider selection and token entry.
- [ ] Confirm sending an email code still works from the email 2FA provider.
- [ ] Confirm successful login still reaches the vault through the existing unlock/sync flow.

### Unlock
- [ ] Confirm `needsLogin` still shows the login redirect path.
- [ ] Confirm PIN unlock still appears and succeeds when enabled.
- [ ] Confirm biometric unlock still appears and succeeds when supported and enabled.
- [ ] Confirm master-password unlock still succeeds when required.
- [ ] Confirm unlock-page logout still returns to `/`.

### Vault
- [ ] Confirm vault loading and error states still render correctly.
- [ ] Confirm folder selection, type filtering, sort changes, and both search inputs still work.
- [ ] Confirm cipher detail selection still ignores stale requests when rapidly switching items.
- [ ] Confirm settings dialog, lock action, and logout action still work.

### Spotlight
- [ ] Confirm spotlight search still matches the same vault items.
- [ ] Confirm spotlight result selection and detail actions still work.
- [ ] Confirm spotlight dismissal and outside-click behavior remain unchanged.

## Suggested Slice Order

1. Thin route files:
   - `/`
   - `/unlock`
   - `/vault`
2. Split feature orchestration hooks:
   - login flow
   - unlock flow
   - vault page model
3. Normalize shared helper boundaries and remove cross-feature leakage.
4. Revisit barrel exports only after module boundaries are stable.

## Completion Criteria

The refactor is ready for completion when:
- Existing user-visible flows behave the same as before.
- The frontend structure clearly separates feature-owned and shared code.
- Targeted duplication in scoped areas is reduced.
- Large mixed-responsibility files selected for refactor are replaced by smaller focused units.
- Validation evidence is recorded for all scoped high-risk flows.

## Validation Notes

- Automated checks completed successfully:
  - `npm run build`
  - `npm run biome:check`
- Manual parity still needs to be executed in the running app using the checklist above for startup,
  login, unlock, vault, and spotlight flows.
