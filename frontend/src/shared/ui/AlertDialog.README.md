# AlertDialog

## Description
Modal confirmation overlay for high-impact actions.

## When to use
Use before destructive, irreversible or security-sensitive UI actions.

## When not to use
Do not use for ordinary information. Use `Dialog`, `Banner` or `InlineMessage`.

## Accessibility
Built on Reka AlertDialog with modal focus management, title and description slots.

## Keyboard
Escape closes the dialog. Tab stays within the modal surface. Enter/Space activate focused buttons.

## Examples
`<AlertDialog title="Discard changes" action-label="Discard" />`

## Anti Patterns
Do not use AlertDialog for domain mutation by itself; the owner component must handle the action.
