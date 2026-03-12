## Why

Vanguard's frontend is currently hard-coded in Chinese, while the general settings dialog already exposes a language selector that does not affect the application. Adding bilingual localization now closes that UX gap and establishes a consistent way to ship future user-facing copy in both Chinese and English.

## What Changes

- Add application-wide localization support for Simplified Chinese and English
- Introduce a global locale state and provider that can be consumed by React routes, dialogs, toasts, and utility formatters
- Persist the selected language so the app restores it on the next launch
- Make the existing language selector in general settings switch the UI immediately after selection
- Route locale-sensitive formatting and sorting through the active locale instead of hard-coded Chinese collation rules
- Externalize user-facing copy from feature components, feedback alerts, and shared error message maps into translation resources

## Capabilities

### New Capabilities
- `app-localization`: Render user-facing UI copy, feedback text, and locale-sensitive formatting in either Chinese or English based on the active locale
- `language-preference`: Allow the user to change the application language from general settings, apply it immediately, and restore the saved choice on startup

### Modified Capabilities

## Impact

- **Frontend**: `src/main.tsx`, settings dialog flow, shared error/toast helpers, locale-sensitive vault utilities, auth/vault/spotlight UI copy
- **Dependencies**: likely add a frontend i18n library plus translation resource files for `zh` and `en`
- **Persistence**: add a lightweight client-side preference store for locale selection
- **Product behavior**: no backend or Tauri API contract changes required for the initial bilingual rollout
