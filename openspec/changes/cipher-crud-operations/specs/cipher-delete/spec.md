## ADDED Requirements

### Requirement: User can delete a cipher
The system SHALL allow authenticated users to permanently delete ciphers from vault.

#### Scenario: Delete cipher by ID
- **WHEN** user confirms deletion of cipher_id
- **THEN** system sends DELETE request to Vaultwarden API, removes from local storage, and emits `CipherDeleted` event

#### Scenario: Delete cipher that does not exist locally
- **WHEN** user attempts to delete cipher_id not in local database
- **THEN** system still sends API request to ensure server-side deletion

#### Scenario: Cipher does not exist on server
- **WHEN** API returns 404 for cipher_id
- **THEN** system removes cipher from local storage if present and emits event

#### Scenario: User lacks delete permission
- **WHEN** user attempts to delete organization cipher without permission
- **THEN** system returns error "You do not have permission to delete this cipher"

### Requirement: System requires explicit confirmation
The system SHALL require user confirmation before deleting cipher.

#### Scenario: Frontend shows confirmation dialog
- **WHEN** user initiates delete action
- **THEN** frontend displays confirmation dialog before calling delete command

#### Scenario: Delete command executes without additional confirmation
- **WHEN** delete command is invoked (confirmation already handled by frontend)
- **THEN** system proceeds with deletion immediately

### Requirement: System emits event after successful deletion
The system SHALL emit `CipherDeleted` event after cipher is removed locally.

#### Scenario: Frontend receives deletion event
- **WHEN** cipher is successfully deleted
- **THEN** system emits event with cipher_id and account_id for frontend to remove from UI
