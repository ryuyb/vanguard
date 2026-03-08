# Data Model: Vault Icons

## Overview

This feature does not introduce a new persisted domain model. It derives display behavior from existing vault data and adds a transient presentation model for website icons in the vault list and cipher detail surfaces.

## Entities

### Cipher
- **Description**: A vault entry shown in the cipher list and detail panel.
- **Existing fields used by this feature**:
  - `id`: Stable identifier used to associate icon loading state with the cipher.
  - `name`: Primary display label that must remain readable whether or not an icon is available.
  - `type`: Existing cipher category used to choose a fallback visual when no website icon is available.
  - `username` or equivalent summary fields: Secondary information that must remain visually subordinate to the title.
- **Relationships**:
  - Has zero or more website values used to determine icon eligibility.
  - Has exactly one displayed icon state per surface at render time.

### Website Address
- **Description**: The website value already associated with a cipher and used to determine the icon lookup target.
- **Attributes**:
  - `rawValue`: Stored website value as already available in current vault data.
  - `normalizedDisplayTarget`: Canonical target derived from the website value for icon lookup and cross-surface consistency.
  - `eligibility`: Whether the value is suitable for icon retrieval.
- **Validation rules**:
  - Must be present and usable to attempt icon retrieval.
  - Malformed, unsupported, local-only, or otherwise unsuitable values are treated as ineligible.
- **Relationships**:
  - Belongs to one cipher.
  - Produces zero or one icon lookup target for display.

### Website Icon Presentation
- **Description**: The transient display state for a cipher’s icon in the vault list or detail panel.
- **Attributes**:
  - `cipherId`: Associates the icon state with a specific cipher.
  - `surface`: Either `list` or `detail`.
  - `requestTarget`: Derived lookup target used to request the icon.
  - `loadState`: `idle`, `loading`, `loaded`, or `fallback`.
  - `resolvedVisual`: Either the remote website icon or the fallback visual.
- **Validation rules**:
  - A cipher can never render a broken image state.
  - If icon retrieval fails or is skipped, `loadState` becomes `fallback`.
  - The same cipher website selection rule must produce the same request target across list and detail surfaces unless underlying cipher data changes.
- **Relationships**:
  - References one cipher.
  - Uses one website address selection outcome.

### Fallback Visual
- **Description**: The deterministic non-broken visual shown when a website icon cannot be displayed.
- **Attributes**:
  - `styleVariant`: Existing UI-consistent fallback style selected for the cipher.
  - `accessibilityLabel`: User-facing description that preserves recognizability.
- **Validation rules**:
  - Must be renderable without remote network access.
  - Must preserve the existing visual hierarchy and not obscure cipher name or core data.

## State Transitions

### List Icon State
1. **idle**: Cipher row is not yet visible or near-visible, so no icon request has started.
2. **loading**: Cipher row becomes eligible for lazy loading and icon retrieval begins.
3. **loaded**: A valid website icon is retrieved and displayed.
4. **fallback**: No eligible website exists or icon retrieval fails; fallback visual is displayed.

### Detail Icon State
1. **loading**: Cipher detail renders and icon retrieval begins immediately if a usable website target exists.
2. **loaded**: A valid website icon is retrieved and displayed.
3. **fallback**: No eligible website exists or icon retrieval fails; fallback visual is displayed.

## Derived Rules

- List icons must not begin loading for all ciphers at once; they transition from `idle` only when the item becomes visible or near-visible.
- Detail icons may load at detail render time because only one selected cipher is shown at once.
- Fallback visuals are required for every ineligible, unavailable, or failed icon state.
- This feature adds presentation state only; it does not require users to manage or edit icon data.
