# CodeBlock

## Description
Plain code display primitive with optional language label, wrapping and line numbers.

## When to use
Use when exact text preservation matters more than syntax color.

## When not to use
Use `SyntaxHighlight` for highlighted code.

## Accessibility
Caption identifies the code block when provided.

## Keyboard
No custom keyboard handling.

## Examples
`<CodeBlock code="const ok = true" language="ts" />`

## Anti Patterns
Do not put executable code evaluation in this component.
