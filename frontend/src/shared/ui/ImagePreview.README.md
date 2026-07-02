# ImagePreview

## Description
Composed image card with title, description, metadata and optional actions slot.

## When to use
Use when an image needs surrounding inspection context.

## When not to use
Do not use as a gallery or provider-specific media browser.

## Accessibility
The nested `Image` owns image alternative text; actions must be named controls.

## Keyboard
Only slotted actions are keyboard interactive.

## Examples
`<ImagePreview :src="src" alt="Attachment preview" title="Receipt" />`

## Anti Patterns
Do not put domain mutations inside the component.
