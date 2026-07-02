# VStack

## Description
Vertical stack shortcut for dense desktop surfaces.

## When to use
Use for grouped controls, form rows, panel content and local card internals.

## When not to use
Do not use as a replacement for list, table or description-list semantics.

## Accessibility
VStack does not change roles. Pick the correct `as` element for the content.

## Keyboard
Keyboard order remains document order.

## Examples
`<VStack gap="md"><slot /></VStack>`

## Anti Patterns
Do not nest stacks just to compensate for missing component API.
