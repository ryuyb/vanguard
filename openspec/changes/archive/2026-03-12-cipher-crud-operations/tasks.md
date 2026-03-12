## 1. Domain & DTOs

- [x] 1.1 Add cipher mutation DTOs in `application/dto/sync.rs` (CreateCipherCommand, UpdateCipherCommand, DeleteCipherCommand)
- [x] 1.2 Add cipher mutation result types (CipherMutationResult with cipher_id, revision_date)

## 2. Application Ports

- [x] 2.1 Extend `RemoteVaultPort` with `create_cipher()`, `update_cipher()`, `delete_cipher()` methods
- [x] 2.2 Add cipher event types to `SyncEventPort` (CipherCreated, CipherUpdated, CipherDeleted)

## 3. Use Cases

- [x] 3.1 Implement `CreateCipherUseCase` (validate unlocked, encrypt data, call port, persist, emit event)
- [x] 3.2 Implement `UpdateCipherUseCase` (validate permissions, encrypt data, call port, persist, emit event)
- [x] 3.3 Implement `DeleteCipherUseCase` (validate permissions, call port, remove from DB, emit event)
- [x] 3.4 Implement `FetchCipherUseCase` (call port, decrypt data, upsert to DB, emit event)

## 4. Infrastructure - Vaultwarden Client

- [x] 4.1 Add cipher mutation endpoints to `vaultwarden/endpoints.rs` (POST/PUT/DELETE /api/ciphers)
- [x] 4.2 Add request/response models to `vaultwarden/models.rs` (CipherRequest, CipherResponse)
- [x] 4.3 Implement `create_cipher()` in `VaultwardenClient` (POST with encrypted payload)
- [x] 4.4 Implement `update_cipher()` in `VaultwardenClient` (PUT with cipher_id and encrypted payload)
- [x] 4.5 Implement `delete_cipher()` in `VaultwardenClient` (DELETE with cipher_id)
- [x] 4.6 Update `RemoteVaultPortAdapter` to implement new port methods

## 5. Infrastructure - Persistence

- [x] 5.1 Add `upsert_cipher()` method to `SqliteVaultRepository` (insert or replace cipher)
- [x] 5.2 Add `delete_cipher()` method to `SqliteVaultRepository` (remove cipher by ID)

## 6. Tauri Interface

- [x] 6.1 Add cipher event types in `interfaces/tauri/events/` (CipherCreated, CipherUpdated, CipherDeleted)
- [x] 6.2 Implement `create_cipher` Tauri command in `commands/vault.rs` (validate input, call use case, map errors)
- [x] 6.3 Implement `update_cipher` Tauri command in `commands/vault.rs`
- [x] 6.4 Implement `delete_cipher` Tauri command in `commands/vault.rs`
- [x] 6.5 Add request/response DTOs in `interfaces/tauri/dto/vault.rs`

## 7. WebSocket Integration

- [x] 7.1 Update `RealtimeSyncService` to handle cipher mutation notifications (SyncCipherUpdate, SyncCipherDelete)
- [x] 7.2 Call `FetchCipherUseCase` when receiving cipher update notification
- [x] 7.3 Call `DeleteCipherUseCase` (local only) when receiving cipher delete notification

## 8. Frontend Integration

- [x] 8.1 Generate TypeScript bindings for new commands and events (`pnpm tauri-specta`)
- [x] 8.2 Create mutation hooks (useCreateCipher, useUpdateCipher, useDeleteCipher)
- [x] 8.3 Add event listeners for cipher events in vault context
- [x] 8.4 Update cipher list UI to handle real-time events

## 9. Testing & Validation

- [ ] 9.1 Test create cipher with various field types (login, note, card, identity)
- [ ] 9.2 Test update cipher with permission validation
- [ ] 9.3 Test delete cipher with confirmation flow
- [ ] 9.4 Test WebSocket-triggered updates
- [x] 9.5 Run `cargo clippy` and fix warnings
- [ ] 9.6 Run `cargo test` and ensure all tests pass
