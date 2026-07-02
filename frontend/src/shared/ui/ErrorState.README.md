# ErrorState

## Description
Local error state with optional action slot.

## When to use
Use when a surface cannot render its expected content.

## When not to use
Do not use for warning-only or empty states.

## Accessibility
Uses `role="alert"` for assertive announcement.

## Keyboard
Retry or dismiss controls belong in the action slot.

## Examples
`<ErrorState title="Could not load" />`

## Anti Patterns
Do not expose raw provider or exception payloads.
