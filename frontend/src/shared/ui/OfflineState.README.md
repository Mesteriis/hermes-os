# OfflineState

## Description
Local offline state for temporarily unavailable remote refresh.

## When to use
Use when local UI remains usable but network refresh is paused.

## When not to use
Do not use for provider authentication failures.

## Accessibility
Uses status semantics and readable copy.

## Keyboard
No custom keyboard behavior is added.

## Examples
`<OfflineState />`

## Anti Patterns
Do not trigger reconnection attempts from this primitive.
