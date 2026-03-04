# Vanguard

Vanguard is a desktop client for Vaultwarden with a 1Password-like experience.

It is built for people who want a clean, native-feeling Vaultwarden app on desktop without extra setup complexity.

## Platform Support

- macOS only (current release)
- Windows and Linux are not supported yet

## What You Can Do

- Sign in to your Vaultwarden account
- Sync your vault data
- Browse and search vault items and folders
- View login details (such as username, password, URL, and notes)
- Lock and unlock your vault with your master password
- Optionally enable Touch ID and PIN unlock on macOS

## Quick Start

1. Open Vanguard.
2. Enter your Vaultwarden server URL.
3. Sign in with your account credentials.
4. Wait for the initial sync to complete.
5. Unlock your vault and start browsing your items.

## macOS Troubleshooting

If you see this message when opening a release build:

- `"vanguard.app" is damaged and can't be opened. You should move it to the Trash.`

One common cause is the macOS quarantine attribute on the downloaded app bundle.

You can remove it with:

```bash
APP_PATH="/absolute/path/to/vanguard.app"
sudo xattr -d com.apple.quarantine "$APP_PATH"
```

Notes:

- `APP_PATH` is not fixed; set it to wherever your `vanguard.app` is located.
- If needed, use `xattr -dr` to remove quarantine recursively for the entire app bundle.

## Security Notes

- Vanguard is designed to work with Vaultwarden's existing security model.
- Touch ID/PIN unlock is optional and intended for convenience.
- Biometric data is handled by macOS system APIs; raw biometric data is not stored by the app.

## Project Status

- Active development
- User-focused improvements are ongoing

## Disclaimer

This is an independent project and is not affiliated with 1Password, Bitwarden, or Vaultwarden.

## Feedback

Please open an issue in this repository for bug reports or feature requests.
