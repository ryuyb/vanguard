## ADDED Requirements

### Requirement: System can fetch individual cipher from API
The system SHALL retrieve single cipher details from Vaultwarden API by cipher_id.

#### Scenario: Fetch cipher after WebSocket notification
- **WHEN** WebSocket notification indicates cipher update
- **THEN** system calls GET /api/ciphers/{id}, decrypts data, updates local storage, and emits `CipherUpdated` event

#### Scenario: Fetch cipher for manual refresh
- **WHEN** user explicitly requests cipher refresh
- **THEN** system fetches latest cipher data from API and updates local cache

#### Scenario: Cipher deleted on server
- **WHEN** API returns 404 for cipher_id
- **THEN** system removes cipher from local storage and emits `CipherDeleted` event

#### Scenario: Cipher access revoked
- **WHEN** API returns 403 forbidden
- **THEN** system removes cipher from local storage and logs access revocation

### Requirement: System decrypts fetched cipher data
The system SHALL decrypt cipher fields using vault crypto after fetching from API.

#### Scenario: Decrypt login cipher
- **WHEN** cipher is fetched from API
- **THEN** system decrypts name, username, password, notes, URIs, and custom fields before storing locally

#### Scenario: Decryption fails
- **WHEN** cipher decryption fails due to key mismatch
- **THEN** system logs error and does not update local storage

### Requirement: Fetch operation updates local storage
The system SHALL update SQLite vault repository with fetched cipher data.

#### Scenario: Update existing cipher
- **WHEN** fetched cipher_id exists locally
- **THEN** system replaces local cipher with fetched data

#### Scenario: Insert new cipher
- **WHEN** fetched cipher_id does not exist locally
- **THEN** system inserts cipher into local storage
