# Split

## Description
Two-pane layout primitive for primary and secondary content.

## When to use
Use for editor/inspector, list/detail and overview/context pairings.

## When not to use
Do not use for user-resizable panes; wrap content in `Resizable` when native resizing is required.

## Accessibility
Split preserves DOM order and does not add landmark roles.

## Keyboard
Split has no keyboard behavior.

## Examples
Use `#primary` and `#secondary` slots for explicit pane ownership.

## Anti Patterns
Do not put unrelated business flows into the secondary pane.
