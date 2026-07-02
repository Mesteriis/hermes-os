# List

## Description
Tokenized list for dense metadata, candidates and local UI records.

## When to use
Use for short homogeneous item sets.

## When not to use
Do not use for tabular comparison.

## Accessibility
Renders native list semantics with optional accessible label.

## Keyboard
The component adds no custom keyboard model.

## Examples
`<List :items="items" label="Review items" />`

## Anti Patterns
Do not make list rows navigate or mutate remote state by themselves.
