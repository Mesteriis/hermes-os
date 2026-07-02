# StatusBar

## Description
Low-height status strip for compact local state summaries.

## When to use
Use for sync, count, mode and local readiness indicators.

## When not to use
Do not use as an alert. Use Banner, Alert or InlineMessage for user-facing feedback.

## Accessibility
Provide `label` when the status bar is not obvious from context.

## Keyboard
StatusBar has no keyboard behavior.

## Examples
Pass `items` with stable ids, labels, values and optional tones.

## Anti Patterns
Do not stream private provider content into status text.
