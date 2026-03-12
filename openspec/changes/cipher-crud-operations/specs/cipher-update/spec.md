## ADDED Requirements

### Requirement: User can update an existing cipher
The system SHALL allow authenticated users to modify cipher data and sync changes to server.

#### Scenario: Update cipher name and credentials
- **WHEN** user modifies cipher name, username, or password
- **THEN** system encrypts updated data, sends to Vaultwarden API with cipher_id, updates local storage, and emits `CipherUpdated` event

#### Scenario: Move cipher to different folder
- **WHEN** user changes cipher's folder_id
- **THEN** system updates folder association and syncs to server

#### Scenario: Update cipher custom fields
- **WHEN** user adds, modifies, or removes custom fields
- **THEN** system encrypts new field values and updates cipher payload

#### Scenario: Cipher does not exist
- **WHEN** user attempts to update non-existent cipher_id
- **THEN** system returns error "Cipher not found"

#### Scenario: Concurrent modification conflict
- **WHEN** server returns 409 conflict due to outdated revision_date
- **THEN** system returns error "Cipher was modified by another device, please refresh"

### Requirement: System validates user has edit permission
The system SHALL verify user has edit permission before allowing updates.

#### Scenario: User lacks edit permission
- **WHEN** cipher.edit is false for the user
- **THEN** system returns error "You do not have permission to edit this cipher"

#### Scenario: Organization cipher with edit access
- **WHEN** user has edit permission on organization cipher
- **THEN** system allows update

### Requirement: System emits event after successful update
The system SHALL emit `CipherUpdated` event after cipher is updated locally.

#### Scenario: Frontend receives update event
- **WHEN** cipher is successfully updated and stored
- **THEN** system emits event with cipher_id, account_id, and updated_fields for frontend to refresh UI
