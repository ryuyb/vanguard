## ADDED Requirements

### Requirement: General settings expose supported language choices
The system SHALL expose Chinese and English as selectable language options in the general settings section.

#### Scenario: User opens language selector
- **WHEN** the user opens the language selector in general settings
- **THEN** the selector shows supported options for Chinese and English and indicates the current selection

### Requirement: Changing language applies immediately
The system SHALL update the active locale immediately after the user selects a supported language in general settings, without requiring a restart.

#### Scenario: User switches from Chinese to English
- **WHEN** the user selects `en` from the general settings language control
- **THEN** the current application window rerenders supported copy in English during the same session

#### Scenario: Unsupported locale value is ignored
- **WHEN** the app encounters a locale value outside the supported set
- **THEN** it keeps or restores a supported locale instead of entering an undefined language state

### Requirement: Selected language persists across launches
The system SHALL persist the last selected language in backend app configuration and restore it when the app starts again.

#### Scenario: Relaunch restores English preference
- **WHEN** the user previously selected `en` and launches the app again
- **THEN** the app initializes with English as the active locale before rendering supported UI copy

#### Scenario: Missing preference falls back to default locale
- **WHEN** there is no stored language preference or the stored value is invalid
- **THEN** the app initializes with `zh` as the default locale

### Requirement: App windows resolve a consistent saved language
The system SHALL keep separately rendered app windows aligned with the same saved language preference resolved from backend app configuration.

#### Scenario: Spotlight opens after locale change
- **WHEN** the user changes the saved language in the main app and later opens Spotlight
- **THEN** Spotlight initializes with the same saved locale

#### Scenario: Background window refreshes locale from saved preference
- **WHEN** an already-open app window regains focus after the saved locale changed elsewhere
- **THEN** it reloads the supported saved locale from backend configuration before rendering new user-visible copy
