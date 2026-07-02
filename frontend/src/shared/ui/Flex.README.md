# Flex

## Description
Low-level flex layout primitive with tokenized gaps and alignment.

## When to use
Use when Stack shortcuts are too narrow and the caller needs inline or wrapped flex behavior.

## When not to use
Do not use for application bars, inspectors or scroll areas when a named layout primitive exists.

## Accessibility
Flex is semantic-neutral and keeps the chosen `as` element.

## Keyboard
Flex has no keyboard behavior.

## Examples
`<Flex wrap gap="sm" align="center">...</Flex>`

## Anti Patterns
Do not use Flex to hide structural problems in domain components.
