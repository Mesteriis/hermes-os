# DescriptionList

## Description
Definition-list surface for labeled metadata.

## When to use
Use for evidence, ownership or object metadata.

## When not to use
Do not use for metrics that need visual comparison.

## Accessibility
Uses native `dl`, `dt` and `dd` semantics through KeyValue children.

## Keyboard
No custom keyboard behavior is required.

## Examples
`<DescriptionList :items="items" title="Evidence" />`

## Anti Patterns
Do not flatten complex domain objects without explicit labels.
