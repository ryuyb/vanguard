## 1. Localization Foundation

- [x] 1.1 Add the frontend i18n dependencies and create the shared `src/i18n/` module structure
- [x] 1.2 Define supported app locales (`zh`, `en`), locale metadata, storage key, and fallback behavior
- [x] 1.3 Create translation resource files for Chinese and English with an initial namespace layout for common, auth, vault, spotlight, and errors

## 2. App Bootstrapping

- [ ] 2.1 Initialize the shared i18n instance and wrap `src/main.tsx` with the locale provider
- [ ] 2.2 Reuse the same i18n bootstrap in `src/spotlight/main.tsx`
- [ ] 2.3 Add locale persistence and rehydration so the current window updates immediately and other windows can restore the saved locale

## 3. Shared Utilities

- [ ] 3.1 Refactor shared error-message and toast helpers to resolve translated copy from the active locale
- [ ] 3.2 Add shared locale-aware formatting helpers for sorting and date/time rendering
- [ ] 3.3 Replace hard-coded locale usage in shared vault utilities with the new locale helpers

## 4. Feature Copy Migration

- [ ] 4.1 Update the vault general settings language selector to read and write the global locale preference
- [ ] 4.2 Migrate auth login and unlock user-facing copy to translation keys
- [ ] 4.3 Migrate vault page, dialogs, detail panels, and success/error toasts to translation keys
- [ ] 4.4 Migrate Spotlight labels, hints, search states, and copy-action text to translation keys

## 5. Validation

- [ ] 5.1 Verify Chinese and English switching works immediately from general settings without restarting the current window
- [ ] 5.2 Verify the saved language is restored on relaunch for the main app and Spotlight entry point
- [ ] 5.3 Run `pnpm build` and `pnpm biome:check` after the migration and fix any localization-related regressions
