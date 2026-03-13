## ADDED Requirements

### Requirement: Auth state is persisted per account in separate files
The system SHALL store persisted auth state in account-scoped files under `auth-states/{account_id}.json`.

#### Scenario: Login persists account-specific auth state
- **WHEN** a user authenticates successfully for account A
- **THEN** backend writes encrypted auth state to `auth-states/{account_id_of_A}.json`

#### Scenario: Multiple accounts coexist
- **WHEN** account A and account B both authenticate on the same device
- **THEN** backend keeps independent auth-state files for both accounts without overwrite

### Requirement: Active account index is maintained separately
The system SHALL persist the current active account identifier in `auth-states/active.json`.

#### Scenario: Active account updates after login
- **WHEN** login succeeds for an account
- **THEN** backend updates `auth-states/active.json` to that account id

#### Scenario: Active account updates after explicit account switch
- **WHEN** user switches to another account with existing auth-state
- **THEN** backend updates `auth-states/active.json` to the switched account id

### Requirement: Restore flow resolves state from active account index
The system SHALL resolve persisted auth restore context based on the account id referenced by `auth-states/active.json`.

#### Scenario: Restore finds active account auth-state
- **WHEN** `active.json` points to an account with a valid auth-state file
- **THEN** backend returns locked/authenticated restore status using that account context

#### Scenario: Active index points to missing auth-state
- **WHEN** `active.json` exists but the referenced account auth-state file is absent or invalid
- **THEN** backend treats persisted auth state as unavailable and returns needs-login status

### Requirement: Logout clears only target account auth-state in default flow
The system SHALL remove persisted auth-state for the current account during standard logout and SHALL update active account index accordingly.

#### Scenario: Logout current account with other accounts remaining
- **WHEN** user logs out of currently active account and other account auth-states remain
- **THEN** backend deletes current account auth-state and re-points active account to an existing remaining account

#### Scenario: Logout current account as last account
- **WHEN** user logs out of currently active account and no other auth-states remain
- **THEN** backend deletes current account auth-state and removes `active.json`
