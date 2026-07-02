# SyntaxHighlight

## Description
Highlighted code display using `highlight.js` with sanitized output.

## When to use
Use for source snippets where syntax improves scanning.

## When not to use
Do not use for editable code input.

## Accessibility
Caption and language labels provide context; code remains text content.

## Keyboard
No custom keyboard handling.

## Examples
`<SyntaxHighlight :code="json" language="json" />`

## Anti Patterns
Do not bypass the sanitizer around highlighted HTML.
