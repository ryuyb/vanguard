## ADDED Requirements

### Requirement: User can create a new cipher
The system SHALL allow authenticated users to create a new cipher with encrypted data.

#### Scenario: Create login cipher with credentials
- **WHEN** user provides cipher name, username, password, and optional folder
- **THEN** system encrypts the data, sends to Vaultwarden API, stores locally, and emits `CipherCreated` event

#### Scenario: Create cipher without folder
- **WHEN** user creates cipher without specifying folder_id
- **THEN** system creates cipher in root vault (folder_id = null)

#### Scenario: Create cipher with custom fields
- **WHEN** user provides custom fields (text, hidden, boolean, linked)
- **THEN** system encrypts custom field values and includes in cipher payload

#### Scenario: Vault is locked
- **WHEN** user attempts to create cipher while vault is locked
- **THEN** system returns error "Vault must be unlocked to create ciphers"

#### Scenario: Network failure during creation
- **WHEN** API call fails due to network error
- **THEN** system does not persist locally and returns error to user

### Requirement: Cipher data must be encrypted before transmission
The system SHALL encrypt all sensitive cipher fields using vault crypto before sending to API.

#### Scenario: Encrypt login credentials
- **WHEN** creating login cipher with password
- **THEN** system encrypts name, username, password, notes, URIs, and custom fields using vault encryption key

#### Scenario: Encryption key unavailable
- **WHEN** vault encryption key is not available in memory
- **THEN** system returns error and prevents cipher creation

### Requirement: System emits event after successful creation
The system SHALL emit `CipherCreated` event after cipher is persisted locally.

#### Scenario: Frontend receives creation event
- **WHEN** cipher is successfully created and stored
- **THEN** system emits event with cipher_id and account_id for frontend to update UI
