<p align="center">
  <img src="docs/img/logo.png" alt="Vanguard Logo" width="120">
</p>

<h1 align="center">Vanguard</h1>

<p align="center">A macOS desktop client for Vaultwarden with a 1Password-like experience.</p>

> [!CAUTION]
> ⚠️ Vanguard is still under active development and incomplete.
> It may contain bugs and could potentially corrupt your vault data.
> Use with caution and don't rely on it as the only copy of important passwords.

## Platform Support

- **macOS** (current release)
- Windows and Linux are not supported yet

## Features

### Available Now

| Feature | Description |
|---------|-------------|
| 🔐 Account Login | Connect to your Vaultwarden server, including self-hosted instances |
| 🔄 Data Sync | Automatically sync your vault data to local storage |
| 🔍 Search & Browse | Quickly search and browse passwords, folders, and items |
| 👁️ View Details | See usernames, passwords, URLs, notes, and other login information |
| 🔢 TOTP Codes | View time-based one-time passwords for 2FA-enabled accounts |
| 🔒 Lock & Unlock | Lock and unlock your vault with your master password |
| 👆 Touch ID | Quick unlock with fingerprint on macOS (requires code signing) |
| 🔢 PIN Unlock | Set a PIN for faster access (optional) |
| 🌍 Multi-language | Interface available in multiple languages |

### Spotlight Search

Vanguard includes a powerful Spotlight-style quick search that lets you access your passwords without opening the main app:

**How to use:**
- Press `Cmd + Shift + L` to open the Spotlight search window from anywhere
- Type to search your vault instantly
- Navigate results with arrow keys

**What you can do:**
- **Quick Copy**: Press `Enter` to copy the password, `Cmd + Enter` to copy the username
- **Auto-fill**: Select an item and press `Tab` to auto-fill credentials into the active app (requires accessibility permissions)
- **Open URL**: Press `Cmd + O` to open the website in your browser

**Setup:**
Grant accessibility permissions in **System Settings > Privacy & Security > Accessibility** to enable auto-fill functionality.

### Coming Soon

- Password generator

## Quick Start

### Installation

1. Download the latest `.dmg` from [Releases](https://github.com/yourusername/vanguard/releases)
2. Open the `.dmg` and drag Vanguard to your Applications folder
3. If you see a "damaged" warning on first launch, see [Troubleshooting](#troubleshooting)

### First-Time Setup

1. Open Vanguard
2. Enter your Vaultwarden server URL (e.g., `https://vault.example.com`)
3. Sign in with your email and master password
4. Wait for the initial sync to complete
5. Start using your vault

### Daily Usage

- **Unlock**: Use your master password or PIN for quick access (Touch ID requires a signed build)
- **Spotlight Search**: Press `Cmd + Shift + L` anywhere to quickly find and copy passwords
- **Search**: Use the search bar in the main app to browse your vault
- **Copy**: Click on any item to copy username or password
- **Lock**: Manually lock when away, or set auto-lock timer

## Troubleshooting

### "App is damaged" Warning

If you see this message when opening Vanguard:

> "vanguard.app" is damaged and can't be opened. You should move it to the Trash.

This is caused by macOS quarantine attributes. Fix it with:

```bash
sudo xattr -d com.apple.quarantine "/Applications/Vanguard.app"
```

If the issue persists, try recursive removal:

```bash
sudo xattr -dr com.apple.quarantine "/Applications/Vanguard.app"
```

## Security Notes

- Vanguard follows Vaultwarden's security model—all data is encrypted
- Touch ID and PIN are for convenience only; biometric data is handled by macOS, not stored by the app
- **Note**: Touch ID requires a code-signed build. Unsigned builds will not show the Touch ID option
- Keep the Vaultwarden web interface or official clients as backup

## Feedback

Found a bug or have a feature request? Open an [issue](https://github.com/ryuyb/vanguard/issues).

## Disclaimer

This is an independent project and is not affiliated with 1Password, Bitwarden, or Vaultwarden.
