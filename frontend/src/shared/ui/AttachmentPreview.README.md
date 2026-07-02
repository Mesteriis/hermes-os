# AttachmentPreview

## Description
Generic attachment summary card with icon, name, MIME type, size and action slot.

## When to use
Use for compact attachment inspection in UI-only contexts.

## When not to use
Do not encode provider-specific attachment lifecycle here.

## Accessibility
The attachment name is the visible heading; slotted actions must be named.

## Keyboard
Only slotted actions are keyboard interactive.

## Examples
`<AttachmentPreview name="report.pdf" mime-type="application/pdf" />`

## Anti Patterns
Do not expose raw local paths, secrets or provider download references.
