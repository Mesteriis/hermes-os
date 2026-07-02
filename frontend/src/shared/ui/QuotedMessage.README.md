# QuotedMessage

## Description
Neutral quoted-message block for source context.

## When to use
Use to show a short cited or replied-to fragment.

## When not to use
Do not use for full document or message body rendering.

## Accessibility
Uses a blockquote with visible author and metadata.

## Keyboard
No custom keyboard behavior.

## Examples
`<QuotedMessage author="Alex" body="Reviewed source fragment." />`

## Anti Patterns
Do not put private provider body parsing inside this component.
