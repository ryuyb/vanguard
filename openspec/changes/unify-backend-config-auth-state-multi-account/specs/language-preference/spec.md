## MODIFIED Requirements

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
