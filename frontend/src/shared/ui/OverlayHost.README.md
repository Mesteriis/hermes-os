# OverlayHost

## Description
Generic host layer for custom overlay composition.

## When to use
Use when an owner needs a tokenized overlay placement surface without modal behavior.

## When not to use
Do not use instead of Dialog, Popover, Toast or Tooltip when those semantics are required.

## Accessibility
OverlayHost is semantic-neutral. Provide `label` when the host itself is meaningful.

## Keyboard
OverlayHost has no keyboard behavior.

## Examples
`<OverlayHost layer="toast">...</OverlayHost>`

## Anti Patterns
Do not use OverlayHost to bypass focus management for modal content.
