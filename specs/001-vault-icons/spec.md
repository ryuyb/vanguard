# Feature Specification: Vault Icons

**Feature Branch**: `001-vault-icons`
**Created**: 2026-03-08
**Status**: Draft
**Input**: User description: "/Users/ryuyb/Developer/RustroverProjects/vaultwarden 这是 vaultwarden 的源代码路径，根据源代码及现有实现实现从 icon server获取网站的 icon 并在 vault 页面的 cipher list 和 cipher detail 展示 icon，list 的 icon 需要懒加载"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse vault items with recognizable icons (Priority: P1)

As a vault user, I want each item in the vault list to show the website icon when one is available, so I can identify the correct login or card more quickly while browsing a long list of items.

**Why this priority**: The vault list is the primary browsing surface, and recognizable icons improve scan speed and reduce selection mistakes during the most frequent user workflow.

**Independent Test**: Can be fully tested by opening a vault that contains website-based items, scrolling through the cipher list, and verifying that visible items progressively display the correct website icons without blocking list interaction.

**Acceptance Scenarios**:

1. **Given** a vault item that has a website address associated with it, **When** the item becomes visible in the vault list, **Then** the list shows that website’s icon after it is loaded.
2. **Given** multiple vault items are present in a long list, **When** the user scrolls through the list, **Then** icons are requested only as items become visible or near-visible rather than for the entire list at once.
3. **Given** a vault item does not have a usable website address or no icon can be retrieved, **When** the item appears in the vault list, **Then** a consistent fallback visual is shown instead of a broken image.

---

### User Story 2 - Confirm item identity in details view (Priority: P2)

As a vault user, I want the cipher detail view to show the website icon for the selected item, so I can quickly confirm I opened the intended entry before copying, editing, or using its contents.

**Why this priority**: The detail view is the second most important verification point after list browsing and helps users confirm identity before sensitive actions.

**Independent Test**: Can be fully tested by opening an individual cipher detail view for an item with a website address and verifying that the matching icon is displayed in the detail presentation.

**Acceptance Scenarios**:

1. **Given** a cipher has a website address with an available icon, **When** the user opens that cipher’s detail view, **Then** the detail view shows the corresponding website icon.
2. **Given** a cipher does not have a usable website address or icon retrieval fails, **When** the user opens the detail view, **Then** the detail view shows the same fallback visual pattern used for unsupported or unavailable icons.

---

### User Story 3 - Experience stable icon behavior across sessions (Priority: P3)

As a vault user, I want icon loading behavior to remain predictable and non-disruptive, so that the vault remains usable even when icon retrieval is slow, unavailable, or inconsistent.

**Why this priority**: Reliability matters for a security product; icon enhancement must never reduce usability or trust in the vault experience.

**Independent Test**: Can be fully tested by using the vault under normal conditions and under simulated slow or unavailable icon responses, then confirming that the vault remains readable and interactive with fallbacks.

**Acceptance Scenarios**:

1. **Given** the icon source responds slowly, **When** the user opens the vault list or a cipher detail page, **Then** the primary vault content remains readable and usable while icon loading completes asynchronously.
2. **Given** the icon source is unavailable, **When** the user browses the vault list and detail pages, **Then** users can still complete their tasks without errors, layout breakage, or blocked interactions.

### Edge Cases

- What happens when a cipher contains multiple website values that could map to different sites? The system uses the same source selection rule already established by the existing vault implementation for choosing the display website.
- How does the system handle website values that are malformed, unsupported, local-network-only, or otherwise unsuitable for external icon lookup? The system skips icon retrieval and shows the fallback visual.
- What happens when the icon server returns a response that cannot be displayed as a valid icon? The system treats it as an unavailable icon and shows the fallback visual.
- What happens when users rapidly scroll through the cipher list? The system prioritizes icons for currently visible or near-visible items and must not degrade scrolling responsiveness.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST retrieve a website icon for a cipher when the cipher has a usable website address that can be matched to an icon source.
- **FR-002**: The system MUST display the retrieved website icon in the vault cipher list for eligible items.
- **FR-003**: The system MUST display the retrieved website icon in the cipher detail view for eligible items.
- **FR-004**: The system MUST use lazy loading for icons in the vault cipher list so icon retrieval occurs only for items that are visible or about to become visible.
- **FR-005**: The system MUST keep vault list browsing and cipher detail interactions usable while icon retrieval is in progress.
- **FR-006**: The system MUST show a consistent fallback visual when a cipher has no usable website address, when icon retrieval is unsupported, or when icon retrieval fails.
- **FR-007**: The system MUST derive the icon request target from the website information already associated with the cipher and must not require users to manually enter separate icon data.
- **FR-008**: The system MUST ensure icon presentation does not alter the existing primary vault information hierarchy, meaning item names and core cipher content remain readable and recognizable with or without icons.
- **FR-009**: The system MUST apply the same icon selection outcome for the same cipher in both the vault list and the cipher detail view, unless the underlying cipher website data changes.
- **FR-010**: The system MUST fail gracefully when the icon service is unavailable, slow, or returns unusable data, without producing broken image states that prevent normal vault usage.

### Key Entities *(include if feature involves data)*

- **Cipher**: A vault entry displayed in the vault list and detail view, including its title, type, and associated website information used to determine icon eligibility.
- **Website Address**: The website value already stored with or derived from a cipher and used as the basis for determining which site icon should be requested.
- **Website Icon**: A visual representation of a website associated with a cipher, either retrieved from an icon source or replaced with a fallback visual when unavailable.
- **Fallback Visual**: The default visual shown when no retrievable website icon exists for a cipher.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In a validation set of vault items that contain usable website addresses, at least 95% of items with an available icon source display a recognizable icon in the vault list and cipher detail view.
- **SC-002**: Users can scroll through a vault list of at least 100 items without noticeable blocking or loss of interaction while icons are being loaded.
- **SC-003**: For vault items without an available website icon, 100% of list and detail presentations show a non-broken fallback visual instead of an empty or failed image state.
- **SC-004**: In acceptance testing, users can identify and open the intended vault item from the list on the first attempt in at least 90% of icon-eligible test cases.

## Assumptions

- The existing vault implementation already contains a canonical way to determine the website value associated with a cipher, and this feature should follow that same rule rather than introduce a new selection workflow.
- An existing icon server capability is available for use by the product, and this feature only adds end-user icon display behavior in the vault experience.
- This feature applies to vault items where a website-based identity is meaningful; items without a suitable website continue to use fallback visuals.
- Lazy loading applies specifically to the vault cipher list, while the detail view may load the icon as part of rendering the selected cipher.
