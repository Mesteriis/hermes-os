# AttachmentChip

## Description
Compact attachment metadata chip.

## When to use
Use for draft attachments, evidence files or generic local artifacts.

## When not to use
Do not load bytes, open files or parse provider attachments here.

## Accessibility
The optional remove action is an icon button with a label.

## Keyboard
Remove uses native button keyboard behavior.

## Examples
`<AttachmentChip name="context.pdf" meta="284 KB" removable />`

## Anti Patterns
Do not store provider blobs or secret paths in the component.
