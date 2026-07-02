# CopyButton

## Description
Small clipboard copy action with copied and error states.

## When to use
Use for copying already-visible text or local identifiers.

## When not to use
Do not copy secrets, tokens or private payloads by default.

## Accessibility
State changes are visible through text and icon changes.

## Keyboard
Uses native button keyboard behavior.

## Examples
`<CopyButton value="local-reference" />`

## Anti Patterns
Do not silently copy hidden sensitive content.
