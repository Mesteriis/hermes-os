# HStack

## Description
Horizontal stack shortcut with center alignment by default.

## When to use
Use for compact rows, metadata lines, toolbar groups and inline action clusters.

## When not to use
Do not use when the row is a semantic toolbar; use `Toolbar` instead.

## Accessibility
The component keeps native semantics through the `as` prop.

## Keyboard
HStack does not manage roving focus; interactive children keep their own keyboard contract.

## Examples
`<HStack gap="sm" justify="between">...</HStack>`

## Anti Patterns
Do not rely on HStack to reorder content visually away from DOM reading order.
