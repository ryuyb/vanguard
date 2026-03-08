# UI Contract: Vault Icon Presentation

## Purpose

Define the user-facing contract for how website icons appear in the vault cipher list and cipher detail view.

## Surfaces

### Cipher List
- Each eligible cipher row displays a website icon area alongside the existing row content.
- Icon loading is deferred until the row is visible or near-visible.
- The row remains readable and interactive while the icon is loading.
- If no icon can be displayed, the row shows a deterministic fallback visual.

### Cipher Detail
- The selected cipher detail view displays the website icon area as part of the detail presentation.
- The detail surface may request the icon when the selected cipher renders.
- If no icon can be displayed, the detail view shows the same fallback outcome used by the list.

## Input Contract

A cipher is icon-eligible when it has a usable website value that can be converted into an icon lookup target.

A cipher is not icon-eligible when:
- it has no website value
- the website value is malformed
- the website value is unsupported for icon retrieval
- icon retrieval returns unusable data

## Output Contract

For each cipher and surface, the UI must resolve to exactly one of the following outcomes:
- remote website icon rendered successfully
- fallback visual rendered successfully

Broken image output is not an allowed outcome.

## Consistency Rules

- The same cipher website selection rule must be used in both the list and detail view.
- The icon outcome for a cipher must remain consistent across both surfaces unless the underlying cipher website data changes.
- Icon presentation must not obscure or replace the cipher title or other primary vault information.

## Performance Rules

- List icon loading must be visibility-driven rather than eager for the entire list.
- Icon loading must not block list scrolling or detail interaction.
