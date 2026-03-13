## Context

Vanguard currently renders most user-facing frontend copy directly in Chinese across auth flows, vault pages, settings, shared toasts, and Spotlight. The general settings dialog already exposes a `zh` / `en` language selector, but it is only local component state and does not persist or drive any application-wide behavior.

The change is cross-cutting because it touches two React entry points (`src/main.tsx` and `src/spotlight/main.tsx`), shared helper modules such as `src/lib/error-messages.ts` and `src/lib/toast.tsx`, and locale-sensitive utilities such as folder sorting and date rendering in the vault feature. There is no existing frontend preference store or locale provider, so this change must establish both the localization infrastructure and the first preference built on top of it.

Constraints:
- Keep the change in the frontend layer; no new Rust, Tauri command, or infrastructure dependency is required for the initial rollout
- Preserve the existing DDD boundary by treating localization as UI behavior, not domain logic
- Avoid logging or persisting sensitive data beyond a benign locale preference
- Support only Simplified Chinese and English for the initial release

## Goals / Non-Goals

**Goals:**
- Add a shared localization foundation for the main app window and Spotlight window
- Allow the user to switch between Chinese and English from general settings and see the result immediately
- Persist the selected locale and restore it on the next app launch
- Route shared copy, error messages, toasts, and locale-sensitive formatting through the active locale
- Replace hard-coded Chinese collation in vault utilities with locale-aware sorting based on the current language

**Non-Goals:**
- Translating Rust-side log messages or domain errors that never reach the UI
- Building a full app preferences subsystem beyond what is needed for locale
- Supporting additional locales, remote translation loading, or runtime translation editing
- Redesigning existing UI layouts beyond the minimum label and copy changes required for bilingual support

## Decisions

### 1. Use `i18next` + `react-i18next` for frontend localization

**Decision**: Introduce `i18next` and `react-i18next` as the localization layer and keep translation resources in versioned frontend modules.

**Rationale**:
- The codebase needs translation access both inside React components and in imperative helpers such as toast/error utilities
- The library provides a proven `useTranslation` hook plus a shared `i18n.t(...)` escape hatch for non-component code
- It scales better than a hand-rolled dictionary context as more features and interpolation cases are added

**Alternatives considered**:
- Custom React context with plain objects: smaller initial footprint, but weaker ergonomics for non-React usage and fallback handling
- `react-intl`: strong formatting support, but less convenient for the many imperative string call sites already present in the repo

### 2. Centralize locale bootstrap in `src/i18n/` and reuse it across entry points

**Decision**: Add a shared `src/i18n/` module that owns locale metadata, translation resources, initialization, persistence helpers, and a provider consumed by both `src/main.tsx` and `src/spotlight/main.tsx`.

**Rationale**:
- The project has two separate React roots, and both must resolve the same locale behavior
- `src/main.tsx` is the correct composition point for the primary app provider tree
- Reusing the same bootstrap for Spotlight avoids a second, divergent localization setup

**Implementation shape**:
- Stable app locale keys remain `zh` and `en`
- Each key maps to display metadata plus formatting tags such as `zh-Hans-CN` and `en-US`
- Shared helper functions expose the active locale tag for `Intl.Collator` and `Intl.DateTimeFormat`

**Alternatives considered**:
- Initializing locale separately in each feature tree: duplicates logic and makes cross-window consistency fragile
- Wiring locale through route context only: does not cover Spotlight or imperative shared utilities cleanly

### 3. Persist locale in frontend storage and default to Chinese when unset

**Decision**: Store the chosen locale in client-side storage under a dedicated key and restore it at boot; if the stored value is missing or invalid, default to `zh`.

**Rationale**:
- There is no existing settings repository or Tauri preference API for app-level UI preferences
- Locale is a benign frontend concern, so browser storage is sufficient for the first rollout
- Defaulting to Chinese preserves current product behavior for existing users

**Cross-window behavior**:
- The current window updates immediately through provider state
- Other windows read the same persisted preference on boot and can refresh from storage on focus or storage change events

**Alternatives considered**:
- Tauri-side settings persistence: heavier than necessary and would add backend scope without clear value for the first release
- System-locale auto-detection on first boot: useful later, but changes current default behavior and is not required for the initial user request

### 4. Translate from shared keys, not from inline fallback strings

**Decision**: Move user-facing copy into namespaced translation resources and convert shared utilities to return translation keys or call the shared translator directly.

**Rationale**:
- Inline Chinese strings are currently spread across components, hooks, constants, and error maps
- Shared modules such as `src/lib/error-messages.ts` and `src/lib/toast.tsx` need the same locale source as React components
- A resource-key approach makes missing-copy audits and future additions tractable

**Examples in scope**:
- Settings labels and placeholders
- Auth feedback and progress text
- Vault success/error toasts and empty states
- Spotlight search UI and copy actions
- Shared error-message maps and fallback text passed into `toErrorText(...)`

**Alternatives considered**:
- Leave shared utilities in Chinese and translate only component JSX: faster initially, but would leave mixed-language UX
- Translate backend responses directly: breaks the frontend-only boundary and complicates DDD separation

### 5. Localize formatting and sorting through shared locale helpers

**Decision**: Replace hard-coded locale usage in utility functions with shared formatting helpers driven by the active locale metadata.

**Rationale**:
- The codebase already hard-codes `zh-Hans-CN` for folder sorting, which would produce incorrect ordering for English users
- Date/time rendering via `toLocaleString()` should be explicit and consistent across features
- Central wrappers reduce duplication and make test cases deterministic

**Alternatives considered**:
- Only translate text and leave formatting unchanged: produces partially localized behavior
- Pass raw locale strings through many call sites: increases drift and makes future locale expansion harder

## Risks / Trade-offs

**[Risk]** Missed hard-coded strings leave parts of the UI in Chinese after switching to English
**Mitigation**: Audit affected directories (`auth`, `vault`, `spotlight`, `lib`) and move strings into translation resources as part of the rollout tasks

**[Risk]** Missing translation keys surface raw keys or inconsistent fallback text
**Mitigation**: Configure fallback locale to `zh` and verify both resource files stay structurally aligned

**[Risk]** Multi-window behavior becomes inconsistent if a background window does not observe locale changes promptly
**Mitigation**: Rehydrate locale from persisted storage on window boot/focus and keep the bootstrap shared between main and Spotlight entry points

**[Trade-off]** Adding an i18n dependency increases bundle size slightly
**Mitigation**: The cost is small relative to the reduction in bespoke localization code and the need for imperative translation support

**[Trade-off]** Using local storage is not a full settings platform
**Mitigation**: The locale key is intentionally isolated so it can later be migrated behind a broader preference service without changing feature code

## Migration Plan

**Phase 1: Localization foundation**
1. Add the i18n dependencies and create shared locale bootstrap modules under `src/i18n/`
2. Wrap `src/main.tsx` and `src/spotlight/main.tsx` with the shared provider
3. Add locale persistence helpers and fallback behavior

**Phase 2: Shared copy migration**
1. Move common labels, error copy, and toast text into translation resources
2. Convert shared helpers and constants to read translated values from the active locale

**Phase 3: Feature migration**
1. Migrate settings, auth, vault, and Spotlight UI strings to translation keys
2. Replace hard-coded sort and date locale usage with shared formatting helpers

**Phase 4: Validation**
1. Verify the settings selector updates the current window immediately
2. Verify persisted locale restoration on relaunch
3. Verify main app and Spotlight use the same saved locale

**Rollback strategy**:
- Revert the frontend localization scaffold and translation resource usage
- Remove the saved locale key from client storage
- The app falls back to the current Chinese-only behavior because no backend schema or persisted data migration is involved

## Open Questions

- None required for proposal readiness; this design intentionally defaults to `zh` on first boot to preserve current behavior and avoid blocking implementation on locale-detection policy.
