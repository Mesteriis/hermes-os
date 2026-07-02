# PropertyGrid

## Description
Grid layout for groups of key-value properties.

## When to use
Use for compact inspection panels.

## When not to use
Do not use for editable forms.

## Accessibility
Uses description-list semantics and tokenized layout.

## Keyboard
Read-only by default; actions belong in owner slots.

## Examples
`<PropertyGrid :items="items" columns="three" />`

## Anti Patterns
Do not make properties editable inside this display primitive.
