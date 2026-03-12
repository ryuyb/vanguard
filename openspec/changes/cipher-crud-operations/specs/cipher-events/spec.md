## ADDED Requirements

### Requirement: System emits typed events for cipher mutations
The system SHALL emit Tauri events for cipher create, update, and delete operations.

#### Scenario: Emit CipherCreated event
- **WHEN** cipher is successfully created and persisted locally
- **THEN** system emits `cipher:created` event with payload { account_id, cipher_id }

#### Scenario: Emit CipherUpdated event
- **WHEN** cipher is successfully updated locally (via user action or WebSocket)
- **THEN** system emits `cipher:updated` event with payload { account_id, cipher_id }

#### Scenario: Emit CipherDeleted event
- **WHEN** cipher is successfully deleted locally
- **THEN** system emits `cipher:deleted` event with payload { account_id, cipher_id }

### Requirement: Frontend can subscribe to cipher events
The system SHALL provide TypeScript event types for frontend consumption.

#### Scenario: Frontend listens to cipher events
- **WHEN** frontend subscribes to `cipher:created`, `cipher:updated`, or `cipher:deleted`
- **THEN** frontend receives typed event payload and updates UI accordingly

#### Scenario: Multiple windows receive events
- **WHEN** cipher mutation occurs
- **THEN** all open Tauri windows receive the event

### Requirement: Events include account context
The system SHALL include account_id in all cipher events for multi-account support.

#### Scenario: Filter events by account
- **WHEN** frontend receives cipher event
- **THEN** frontend can filter events by account_id to update correct account's UI

#### Scenario: Event for different account
- **WHEN** event account_id does not match active account
- **THEN** frontend ignores event or updates background cache
