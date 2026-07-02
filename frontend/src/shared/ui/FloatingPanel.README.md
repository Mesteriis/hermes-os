# FloatingPanel

## Description
Non-modal floating surface for lightweight contextual layout.

## When to use
Use for local helper panels, previews and non-blocking context.

## When not to use
Do not use for popover behavior that requires focus management; use `Popover`.

## Accessibility
Provide `label` when the panel contains meaningful content.

## Keyboard
FloatingPanel does not close on Escape and does not manage focus.

## Examples
`<FloatingPanel placement="bottom-end">...</FloatingPanel>`

## Anti Patterns
Do not pretend FloatingPanel is a dialog.
