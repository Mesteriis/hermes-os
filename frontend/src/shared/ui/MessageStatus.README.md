# MessageStatus

## Description
Small delivery-state label for message-like surfaces.

## When to use
Use to show queued, sent, delivered, read or failed states.

## When not to use
Do not treat this as provider command truth.

## Accessibility
The state has an accessible label and optional visible text.

## Keyboard
No keyboard behavior.

## Examples
`<MessageStatus status="delivered" />`

## Anti Patterns
Do not infer business completion from this visual state alone.
