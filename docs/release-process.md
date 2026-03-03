# Release Process (GitHub Actions)

This project uses a tag-driven release flow. GitHub Actions will automatically build and publish macOS artifacts to GitHub Releases.

## Versioning Rules

- Use SemVer tags with a `v` prefix.
- Stable release: `vX.Y.Z` (example: `v0.2.0`)
- Pre-release: `vX.Y.Z-<channel>.<n>` (example: `v0.2.0-rc.1`, `v0.2.0-beta.2`)

## Version Source of Truth

Before tagging, keep these three versions identical:

- `package.json` -> `version`
- `src-tauri/Cargo.toml` -> `package.version`
- `src-tauri/tauri.conf.json` -> `version`

The workflow will fail if tag version and file versions do not match.

## Workflow Overview

Workflow file: `.github/workflows/publish-macos.yml`

Trigger:

- `push` on tags matching `v*`
- `workflow_dispatch` (manual run from GitHub Actions UI)

Pipeline stages:

1. Resolve metadata from tag:
   - Parses `github.ref_name`
   - Detects `release` vs `pre-release` by checking whether version contains `-`
2. Version alignment check:
   - Validates tag version against `package.json`, `Cargo.toml`, and `tauri.conf.json`
3. Rust quality gate (macOS runner):
   - `cargo check`
   - `cargo test`
   - `cargo clippy --all-targets --all-features -- -D warnings`
4. Create or update GitHub Release:
   - Stable tag -> `release`
   - Tag with suffix (for example `-rc.1`) -> `pre-release`
5. Build macOS artifacts (matrix):
   - `aarch64-apple-darwin`
   - `x86_64-apple-darwin`
   - Upload artifacts to the same GitHub Release

## How To Publish

1. Update versions in:
   - `package.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`
2. Commit to `main`.
3. Create and push tag.

Stable release:

```bash
git tag v0.2.0
git push origin v0.2.0
```

Pre-release:

```bash
git tag v0.2.0-rc.1
git push origin v0.2.0-rc.1
```

After the workflow completes, open GitHub Releases to verify generated macOS assets.

## Manual Publish (Workflow Dispatch)

You can also publish manually from GitHub Actions without pushing a tag first.

1. Open `Actions` -> `publish-macos` -> `Run workflow`.
2. Fill inputs:
   - `tag`: target version tag (for example `v0.2.0` or `v0.2.0-rc.1`)
   - `ref`: git ref to build from (default: `main`)
3. Run the workflow.

Behavior:

- `tag` without suffix (for example `v0.2.0`) -> stable release
- `tag` with suffix (for example `v0.2.0-rc.1`) -> pre-release
- Version alignment checks still apply and can fail the run if versions are inconsistent.
