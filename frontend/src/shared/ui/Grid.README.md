# Grid

## Description
Tokenized responsive grid for UI-only component composition.

## When to use
Use for repeated panels, metric groups and responsive component galleries.

## When not to use
Do not use for tabular data. Use `Table` or `VirtualTable`.

## Accessibility
Grid keeps native semantics unless the caller chooses a semantic `as` element.

## Keyboard
Grid does not trap focus or change traversal order.

## Examples
`<Grid columns="auto" gap="lg">...</Grid>`

## Anti Patterns
Do not encode domain-specific breakpoint rules in Grid consumers.
