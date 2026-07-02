# EmptyState

## Description
Generic empty state for UI surfaces with no visible content.

## When to use
Use when a surface is valid but has no entries yet.

## When not to use
Do not use for errors, offline states or search misses.

## Accessibility
Provides heading, description and optional action slot.

## Keyboard
Keyboard behavior belongs to slotted actions.

## Examples
`<EmptyState title="No candidates" />`

## Anti Patterns
Do not imply data was deleted unless the owner proves it.
