# MessageBubble

## Description
Provider-neutral message surface for conversation-like UI.

## When to use
Use for rendered message excerpts, review comments, or communication previews.

## When not to use
Do not parse provider payloads or fetch message content inside this component.

## Accessibility
Uses an article landmark with visible author, time, meta, body and footer slots.

## Keyboard
No custom keyboard behavior; interactive children keep their native behavior.

## Examples
`<MessageBubble author="Owner" direction="outbound">Text</MessageBubble>`

## Anti Patterns
Do not put Telegram, Mail or WhatsApp business logic in props or slots.
