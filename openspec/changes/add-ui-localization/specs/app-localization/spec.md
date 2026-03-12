## ADDED Requirements

### Requirement: Application renders supported copy in the active locale
The system SHALL render supported user-facing copy in Simplified Chinese or English based on the active locale across the main window, dialogs, auth flows, vault flows, shared toasts, and Spotlight.

#### Scenario: Main app boots in English
- **WHEN** the active locale is `en`
- **THEN** the application renders supported user-facing copy from the English resource set instead of the Chinese defaults

#### Scenario: Missing translation falls back safely
- **WHEN** a translation key is unavailable in the active locale
- **THEN** the system falls back to the default locale resource instead of rendering a blank string or raw implementation error

### Requirement: Shared helpers can resolve translated copy
The system SHALL expose the active translation catalog to imperative frontend code paths that do not render inside React components.

#### Scenario: Toast or error helper resolves copy
- **WHEN** a shared helper constructs a user-visible toast or error message
- **THEN** it resolves titles, descriptions, and action labels from the active locale resources

#### Scenario: Spotlight uses the shared localization bootstrap
- **WHEN** the Spotlight window initializes
- **THEN** it uses the same translation resources and locale bootstrap as the main application window

### Requirement: Locale-sensitive formatting honors the active locale
The system SHALL apply the active locale to sorting and display formatting for user-visible data.

#### Scenario: Folder sorting uses English collation
- **WHEN** the active locale is `en` and folders or labels are sorted
- **THEN** the system uses the English locale mapping instead of hard-coded Chinese collation

#### Scenario: Timestamps render with locale-specific formatting
- **WHEN** the UI renders a date or date-time value
- **THEN** the displayed text follows the active locale's formatting rules
