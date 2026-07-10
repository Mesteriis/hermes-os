# Mention

## Description
Inline mention primitive for Personas, entities or references.

## When to use
Use inside text-like UI where an entity needs compact emphasis.

## When not to use
Do not resolve or fetch mentioned entities from this component.

## Accessibility
Text remains visible and does not depend on color alone.

## Keyboard
No keyboard behavior unless wrapped by an interactive parent.

## Examples
`<Mention label="@Owner" icon="tabler:user" />`

## Anti Patterns
Do not make mentions domain stores in disguise.
