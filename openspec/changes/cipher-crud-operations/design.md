## Context

Vanguard currently supports read-only vault operations: syncing from Vaultwarden and viewing cipher details. Users must switch to the web interface to create, edit, or delete ciphers, breaking the native desktop experience.

The backend follows DDD architecture with clear layer separation:
- **Domain**: Pure business logic (no external dependencies)
- **Application**: Use cases, ports (interfaces), DTOs, services
- **Infrastructure**: Port implementations (Vaultwarden API client, SQLite repository, security adapters)
- **Interfaces**: Tauri commands (thin adapters that validate input, call use cases, emit events)

Current cipher flow:
1. Sync fetches all ciphers via `/api/sync` endpoint
2. Ciphers stored encrypted in SQLite (`sqlite_vault_repository.rs`)
3. Frontend queries local vault data via Tauri commands
4. WebSocket notifications trigger incremental syncs

Constraints:
- All cipher data must be encrypted before storage (using `vault_crypto.rs`)
- Master password/key material never leaves application layer
- Tauri commands must not contain business logic
- Events must be emitted for all state changes to keep UI reactive

## Goals / Non-Goals

**Goals:**
- Add cipher create/update/delete operations with full DDD compliance
- Integrate individual cipher fetch API (`GET /api/ciphers/{id}`) for real-time updates
- Emit events on cipher mutations for immediate frontend updates
- Support WebSocket-triggered cipher updates (already partially implemented via realtime sync)
- Maintain encryption/decryption flow consistency with existing sync logic

**Non-Goals:**
- Attachment upload/download (separate feature)
- Offline-first conflict resolution (rely on Vaultwarden's server-side conflict handling)
- Bulk operations (batch create/delete)
- Cipher history/versioning (Vaultwarden feature, not client-side)

## Decisions

### 1. API Integration Strategy

**Decision**: Use Vaultwarden's REST API for cipher mutations, not WebSocket commands.

**Rationale**:
- Vaultwarden's WebSocket is notification-only (push events), not bidirectional RPC
- REST API provides clear request/response semantics with proper error handling
- Existing `VaultwardenClient` already handles auth token injection and error mapping

**Alternatives considered**:
- WebSocket commands: Not supported by Vaultwarden's protocol
- Optimistic UI updates: Risky without server confirmation; prefer request-response flow

**Implementation**:
- Add methods to `RemoteVaultPort`: `create_cipher`, `update_cipher`, `delete_cipher`
- Implement in `VaultwardenClient` using endpoints from reference:
  - `POST /api/ciphers` (create)
  - `PUT /api/ciphers/{id}` (update)
  - `DELETE /api/ciphers/{id}` (delete)
  - `GET /api/ciphers/{id}` (fetch individual)

### 2. Event Emission Architecture

**Decision**: Emit domain events from use cases, not from Tauri commands.

**Rationale**:
- Events represent business outcomes, not UI actions
- Use cases are the single source of truth for state changes
- Allows future non-Tauri interfaces (CLI, tests) to trigger same events

**Event types** (new):
- `cipher:created` - Emitted after successful cipher creation
- `cipher:updated` - Emitted after successful cipher update
- `cipher:deleted` - Emitted after successful cipher deletion
- `cipher:fetched` - Emitted when individual cipher is fetched (for cache invalidation)

**Implementation**:
- Add `CipherEventPort` trait in `application/ports/`
- Implement adapter in `interfaces/tauri/events/cipher_event_adapter.rs`
- Inject port into use cases via `AppState`

### 3. Encryption Flow

**Decision**: Reuse existing `VaultCrypto` for cipher encryption/decryption.

**Rationale**:
- Cipher encryption keys are derived from master password (already implemented)
- Sync flow already handles encryption/decryption of cipher fields
- No need to duplicate crypto logic

**Flow**:
1. Use case receives plaintext cipher data from Tauri command
2. Use case calls `VaultCrypto::encrypt_cipher()` before sending to API
3. API returns encrypted cipher
4. Use case decrypts for local storage (SQLite stores encrypted data)
5. Frontend receives decrypted data via events

### 4. Conflict Handling

**Decision**: Rely on Vaultwarden's server-side conflict detection (revision date checks).

**Rationale**:
- Vaultwarden uses `revisionDate` field to detect stale updates
- Client sends current `revisionDate` with update requests
- Server returns 409 Conflict if revision is outdated
- Simpler than client-side CRDT or OT

**Error handling**:
- 409 Conflict → Fetch latest cipher, show merge UI (future enhancement)
- For MVP: Show error message, require user to refresh and retry

### 5. Local Cache Update Strategy

**Decision**: Update SQLite immediately after successful API call, then emit event.

**Rationale**:
- Ensures local cache is consistent before UI updates
- Avoids race conditions between event emission and cache queries
- Follows existing sync flow pattern

**Flow**:
1. Use case calls API (create/update/delete)
2. API returns success
3. Use case updates SQLite via `VaultRepositoryPort`
4. Use case emits event via `CipherEventPort`
5. Frontend receives event, queries updated cache

### 6. WebSocket Integration

**Decision**: Extend existing `RealtimeSyncService` to handle cipher-specific push notifications.

**Rationale**:
- WebSocket already connected and handling sync notifications
- Vaultwarden sends `SyncCipherUpdate` / `SyncCipherDelete` push messages
- Current implementation triggers full sync; optimize to fetch individual cipher

**Enhancement**:
- On `SyncCipherUpdate`: Call `get_cipher()` API, update SQLite, emit `cipher:updated`
- On `SyncCipherDelete`: Remove from SQLite, emit `cipher:deleted`
- Avoids full sync overhead for single cipher changes

## Risks / Trade-offs

**[Risk]** Concurrent modifications from multiple devices → **Mitigation**: Rely on server-side conflict detection (409 response), show error to user

**[Risk]** Network failure during create/update leaves inconsistent state → **Mitigation**: Wrap API call + SQLite update in transactional use case; rollback on failure

**[Risk]** Large cipher data (e.g., long notes) may slow encryption → **Mitigation**: Encryption is fast for typical cipher sizes (<10KB); monitor performance in testing

**[Trade-off]** Immediate SQLite update vs. eventual consistency: Chose immediate update for simpler reasoning, but increases use case complexity

**[Trade-off]** Event granularity: Emit per-cipher events (not batch) for fine-grained UI updates, but increases event traffic

## Migration Plan

**Phase 1: Backend Implementation**
1. Add port methods to `RemoteVaultPort`
2. Implement API client methods in `VaultwardenClient`
3. Create use cases: `CreateCipherUseCase`, `UpdateCipherUseCase`, `DeleteCipherUseCase`, `FetchCipherUseCase`
4. Add `CipherEventPort` and Tauri adapter
5. Wire dependencies in `AppState`

**Phase 2: Tauri Commands**
1. Add commands: `create_cipher`, `update_cipher`, `delete_cipher`, `fetch_cipher`
2. Define request/response DTOs in `interfaces/tauri/dto/vault.rs`
3. Map domain errors to user-friendly messages

**Phase 3: WebSocket Enhancement**
1. Extend `RealtimeSyncService` to handle `SyncCipherUpdate` / `SyncCipherDelete`
2. Call `FetchCipherUseCase` on push notification
3. Test multi-device sync scenarios

**Phase 4: Frontend Integration**
1. Create mutation hooks: `useCreateCipher`, `useUpdateCipher`, `useDeleteCipher`
2. Add event listeners for cipher events
3. Build create/edit/delete UI components

**Rollback Strategy**:
- Backend changes are additive (no breaking changes to existing sync flow)
- If critical bug found, disable new Tauri commands via feature flag
- Frontend can fall back to read-only mode

## Open Questions

1. **Cipher validation rules**: Should client validate cipher fields (e.g., URI format) before sending to API, or rely on server validation?
   - **Recommendation**: Basic client-side validation (non-empty name, valid URI format) for better UX, but trust server as source of truth

2. **Optimistic UI updates**: Should frontend show cipher immediately before API confirmation?
   - **Recommendation**: No for MVP (wait for API response); consider for future enhancement with rollback logic

3. **Cipher templates**: Should we provide templates for common cipher types (login, card, identity)?
   - **Recommendation**: Out of scope for this change; can be added as frontend-only feature later
