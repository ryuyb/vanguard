## ADDED Requirements

### Requirement: Backend stores and serves global app configuration as the single source of truth
The system SHALL persist global app configuration in backend storage and SHALL expose read/write interfaces for frontend callers without requiring frontend local persistence.

#### Scenario: Frontend reads configuration on startup
- **WHEN** the application frontend initializes
- **THEN** it obtains locale and other global settings from backend configuration APIs as the authoritative source

#### Scenario: Frontend updates global configuration
- **WHEN** the user updates a supported global setting
- **THEN** backend persists the new value and subsequent reads return the updated value

### Requirement: Global configuration includes locale as a first-class field
The system SHALL include locale in global app configuration alongside existing global settings.

#### Scenario: Locale key exists in global app config
- **WHEN** backend initializes app config without a stored locale value
- **THEN** it writes and returns a default supported locale value

#### Scenario: Locale value is unsupported
- **WHEN** backend reads an unsupported locale value from storage
- **THEN** backend returns a supported default locale and persists the corrected value

### Requirement: Global configuration includes Vault Settings general preferences
The system SHALL include Vault Settings general preferences in global app configuration and SHALL return them to frontend as part of unified app-config reads.

#### Scenario: Settings dialog loads persisted general preferences
- **WHEN** user opens Vault Settings
- **THEN** frontend gets launch-on-login, show-website-icon, and shortcut preferences from backend app-config instead of only in-memory defaults

#### Scenario: Settings update is persisted through backend
- **WHEN** user updates a supported general preference in Vault Settings
- **THEN** frontend writes through backend app-config APIs and reopening the dialog returns the updated value

### Requirement: Global configuration includes Vault Settings security preferences
The system SHALL include Vault Settings security behavior preferences in global app configuration while keeping credential materials in existing secure storage.

#### Scenario: Security behavior settings are persisted
- **WHEN** user changes require-master-password interval, lock-on-sleep, idle auto-lock delay, or clipboard clear delay
- **THEN** backend persists these values in app-config and subsequent reads return the same values

#### Scenario: Security secret storage remains unchanged
- **WHEN** user enables/disables biometric or PIN unlock
- **THEN** secret-related data handling continues to use current secure mechanisms and is not moved into app-config

### Requirement: Global configuration enforces supported value domains
The system SHALL validate enum-like app-config fields and SHALL fallback to documented defaults when persisted values are unsupported.

#### Scenario: Invalid persisted option is detected
- **WHEN** backend reads an unsupported option for a bounded settings field (for example locale, auto-lock interval, clipboard interval, or master-password interval)
- **THEN** backend returns a supported default value and persists the corrected value

### Requirement: Frontend no longer persists language preference directly
The system SHALL prevent frontend language persistence from bypassing backend configuration storage.

#### Scenario: Language selection is saved
- **WHEN** the user selects a new language in settings
- **THEN** frontend writes through backend configuration APIs instead of localStorage
