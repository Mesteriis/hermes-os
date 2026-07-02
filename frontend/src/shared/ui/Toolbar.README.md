# Toolbar

## Description
ARIA toolbar primitive for grouped local controls.

## When to use
Use for editor controls, filters and component-local command groups.

## When not to use
Do not use as global navigation; use `Dock`, `TopBar` or app composition.

## Accessibility
Renders `role="toolbar"` and `aria-orientation`. Provide `label` for screen readers.

## Keyboard
Toolbar does not impose roving tabindex; child controls keep native keyboard behavior.

## Examples
`<Toolbar label="Editor tools">...</Toolbar>`

## Anti Patterns
Do not put unrelated actions into one toolbar label.
