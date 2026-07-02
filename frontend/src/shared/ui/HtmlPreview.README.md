# HtmlPreview

## Description
Generic HTML/text preview primitive with an explicit sanitized rendering boundary.

## When to use
Use for already-prepared HTML or plain-text body fragments.

## When not to use
Do not use as a Mail UI component or provider body parser.

## Accessibility
Rendered HTML is sanitized and preserves semantic structure when allowed.

## Keyboard
Links inside sanitized HTML use native keyboard behavior.

## Examples
`<HtmlPreview format="html" sanitized :content="safeHtml" />`

## Anti Patterns
Do not pass raw provider HTML with `sanitized` set to true.
