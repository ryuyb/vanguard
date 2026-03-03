# AGENTS.md

## Project Snapshot
- App type: Tauri 2 desktop app.
- Backend: Rust in `src-tauri/src`.
- Frontend: React/TypeScript in `src/`.
- Core domains: auth, vault sync, vault unlock/biometric.

## DDD Architecture Constraints
- Enforce dependency direction: `interfaces -> application -> domain`.
- `infrastructure` only implements application ports; do not place business rules there.
- `domain` must stay pure: no Tauri, HTTP, SQLite, or OS APIs.
- Tauri commands are interface adapters only:
  - validate/request mapping,
  - call application services/use-cases,
  - map errors/DTOs.
- Do not bypass application layer from interfaces to infrastructure.

## Security Baseline
- Never log secrets: `access_token`, `refresh_token`, `password`, `master_password`, key material.
- Keep redaction on all user-visible and loggable error messages.
- Do not persist plaintext sensitive data; use encrypted persistence/keychain only.
- Treat auth/session initialization as fail-closed on critical errors.
- Keep `allow_invalid_certs = false` by default; only allow in explicit local dev scenarios.
- Validate all external input (Tauri command args, remote payload assumptions).

## Quality Gate (Rust)
- Before merge, run in `src-tauri/`:
  - `cargo check`
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`

## Commit Messages (Conventional Commits)
- Format: `<type>(<scope>): <subject>`
- Types: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `chore`, `build`, `ci`.
- Subject rules:
  - imperative mood,
  - lowercase start,
  - concise, no trailing period.
- Recommended scope examples: `auth`, `sync`, `vault`, `desktop`, `infra`, `bootstrap`, `docs`.
- Examples:
  - `fix(sync): avoid debounce pollution when sync slot acquisition fails`
  - `refactor(vault): move unlock flow from tauri command to application use case`
  - `docs(agents): add ddd and security constraints for codex`

