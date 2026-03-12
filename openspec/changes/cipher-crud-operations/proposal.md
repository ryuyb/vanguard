## Why

Currently, Vanguard only supports viewing and syncing ciphers from Vaultwarden. Users cannot create, edit, or delete ciphers within the app, forcing them to use the web interface for these operations. This breaks the native desktop experience and limits Vanguard's usefulness as a standalone password manager.

## What Changes

- Add cipher creation capability with support for all field types (credentials, notes, URIs, custom fields)
- Add cipher editing capability to modify existing vault items
- Add cipher deletion capability with proper confirmation
- Integrate Vaultwarden's individual cipher fetch API (`GET /api/ciphers/{id}`) for real-time updates
- Implement event emission system for cipher changes (create/update/delete) to trigger frontend updates
- Support WebSocket-triggered updates to keep UI in sync with remote changes
- Maintain DDD architecture: domain models, application use cases, infrastructure adapters, and Tauri command interfaces

## Capabilities

### New Capabilities
- `cipher-create`: Create new ciphers with validation and encryption
- `cipher-update`: Edit existing ciphers with conflict detection
- `cipher-delete`: Delete ciphers with confirmation and cleanup
- `cipher-fetch`: Fetch individual cipher details from Vaultwarden API
- `cipher-events`: Event emission system for real-time UI updates

### Modified Capabilities

## Impact

- **Backend**: New use cases in `application/use_cases/`, new port methods in `application/ports/remote_vault_port.rs`, new API client methods in `infrastructure/vaultwarden/client.rs`, new Tauri commands in `interfaces/tauri/commands/vault.rs`
- **Frontend**: New mutation hooks, event listeners for cipher changes, UI components for create/edit/delete forms
- **Domain**: Potential new validation rules in domain layer for cipher mutations
- **Events**: New event types in `interfaces/tauri/events/` for cipher change notifications
- **Security**: Encryption/decryption of cipher data using existing vault crypto, validation of user permissions
