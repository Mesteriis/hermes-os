# TypingIndicator

## Description
Small status primitive for pending text activity.

## When to use
Use for generic typing, composing or processing activity.

## When not to use
Do not use as proof of real provider presence or remote activity.

## Accessibility
Renders as `role="status"` with an accessible label.

## Keyboard
No keyboard behavior.

## Examples
`<TypingIndicator label="Assistant is composing" />`

## Anti Patterns
Do not bind this directly to provider runtime internals.
