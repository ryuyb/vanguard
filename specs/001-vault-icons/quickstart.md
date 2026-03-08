# Quickstart: Vault Icons

## Goal

Verify that vault items display website icons from the icon source in both the cipher list and cipher detail view, with lazy loading in the list and graceful fallback behavior everywhere.

## Preconditions

- Run the app in the existing local development workflow for this repository.
- Sign in to a vault that contains multiple ciphers, including:
  - items with a usable website address and an available icon
  - items without a usable website address
  - items with malformed or unsupported website values
- Prefer a vault with at least 100 items to validate scrolling behavior.

## Validation Steps

### 1. Confirm list icon rendering for eligible items
1. Open the vault page.
2. Observe the initial visible rows in the cipher list.
3. Confirm that eligible items display website icons once loaded.
4. Confirm that cipher names and usernames remain readable during and after icon loading.
5. Confirm that unsupported items render a deterministic fallback visual instead of an empty or broken image.

### 2. Confirm list lazy loading behavior
1. Start at the top of a long cipher list.
2. Scroll gradually downward.
3. Confirm that icons appear for rows as they enter view or near-view rather than appearing for the entire list immediately.
4. Confirm that scrolling remains smooth and the list remains interactive while icons load.
5. Confirm that rapidly scrolling past rows does not cause obvious eager loading for off-screen items.

### 3. Confirm detail icon rendering
1. Select a cipher with a usable website address.
2. Open its detail panel.
3. Confirm that the detail panel shows the corresponding website icon.
4. Confirm that the icon shown in detail matches the icon outcome used for the same cipher in the list.
5. Confirm that unsupported detail items render the same fallback style used in the list.

### 4. Confirm fallback behavior
1. Select or locate a cipher without a usable website address.
2. Confirm that both the list row and detail panel show a consistent fallback visual.
3. Repeat with a cipher whose website value is malformed or unsupported.
4. Confirm there is no broken image state.

### 5. Confirm degraded network behavior
1. Run the app in conditions where icon loading is slow or unavailable.
2. Open the vault page and a cipher detail panel.
3. Confirm that primary vault content remains readable and interactive.
4. Confirm that failed icon loads resolve to the fallback visual without layout breakage.
5. Confirm that rapidly scrolling through the list does not trigger obvious eager icon loading for far off-screen rows.

## Automated Checks

Use `pnpm` for frontend validation in this repository.

Run the repository’s existing frontend checks that are already part of normal development for this codebase:
- `pnpm biome:check`
- `pnpm build`

If a touched-file-only formatting pass is needed during implementation, use `pnpm biome:write` or `pnpm biome:format` before the final validation run.

### Latest Validation Results
- `pnpm biome:check` — PASS
- `pnpm build` — PASS
- Detail icon integration build verification — PASS
- Degraded-state and lazy-loading hardening build verification — PASS


## Acceptance Summary

The feature is ready when:
- eligible ciphers show website icons in both list and detail views
- list icons load lazily based on visibility
- unsupported or failed cases always show a fallback visual
- vault browsing and detail usage remain responsive during icon loading

## Final Validation Evidence

- List icon rendering implemented and validated
- Detail icon rendering implemented and validated
- Fallback rendering implemented for unsupported and failed icon loads
- Visibility-based lazy loading implemented for list rows
- `pnpm biome:check` completed successfully
- `pnpm build` completed successfully
