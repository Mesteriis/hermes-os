# ActionBar

## Description
Grouped action strip for local submit, cancel and contextual actions.

## When to use
Use at panel edges or inline surfaces where actions need consistent spacing.

## When not to use
Do not use for persistent navigation.

## Accessibility
Renders `role="group"` and accepts a `label`.

## Keyboard
ActionBar does not alter child control keyboard behavior.

## Examples
`<ActionBar justify="end">...</ActionBar>`

## Anti Patterns
Do not hide destructive actions in unlabeled action groups.
