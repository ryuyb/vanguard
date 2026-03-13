## 1. Backend app-config unification

- [x] 1.1 Extend backend app config model to include `locale` with supported-value validation and default fallback behavior
- [x] 1.2 Extend backend app config schema to include Vault Settings preferences (launch-on-login, show-website-icon, shortcuts, require-master-password interval, lock-on-sleep, idle auto-lock delay, clipboard clear delay)
- [x] 1.3 Expose backend commands/services to read and update full global app config as the single source of truth
- [x] 1.4 Ensure all app-config persistence paths keep atomic write and consistent error logging behavior

## 2. Frontend settings persistence switch

- [ ] 2.1 Replace frontend locale load/save implementation to use backend config APIs instead of localStorage
- [ ] 2.2 Update app and Spotlight locale bootstrap flow to initialize from backend-provided locale before rendering user-facing copy
- [ ] 2.3 Refactor Vault Settings dialog to initialize general/security preference controls from backend app-config values
- [ ] 2.4 Refactor Vault Settings preference updates to write through backend app-config APIs and reflect saved values on reopen
- [ ] 2.5 Remove or retire obsolete localStorage-based language persistence and component-state-only settings persistence code paths

## 3. Multi-account auth-state storage refactor

- [ ] 3.1 Refactor persisted auth-state storage from single `auth-state.json` to per-account files under `auth-states/{account_id}.json`
- [ ] 3.2 Add active-account index file `auth-states/active.json` and update login/account-switch flows to maintain it
- [ ] 3.3 Update restore-auth-state flow to resolve persisted context based on active account index and handle missing/invalid target file as needs-login
- [ ] 3.4 Update logout flow to delete current account auth-state and refresh/remove active index according to remaining account states

## 4. Validation and regression checks

- [ ] 4.1 Add or update backend tests for app-config validation/defaulting (locale + bounded settings options) and per-account auth-state read/write behavior
- [ ] 4.2 Add or update frontend/integration checks for settings persistence (Vault Settings reopen consistency, locale bootstrap) and multi-account login/logout/restore scenarios
- [ ] 4.3 Run required project checks for touched surfaces (Rust and frontend) and resolve all issues
