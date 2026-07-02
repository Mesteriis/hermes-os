# MarkdownViewer

## Description
Markdown rendering primitive using `marked` and `DOMPurify`.

## When to use
Use for already-loaded markdown notes, previews and evidence excerpts.

## When not to use
Do not load files, resolve links or mutate documents from this component.

## Accessibility
Rendered markdown keeps semantic headings, lists and links when present.

## Keyboard
Links use native keyboard behavior.

## Examples
`<MarkdownViewer :source="markdown" title="Extract" />`

## Anti Patterns
Do not render unsanitized parser output.
