# Stack

## Description
Generic one-dimensional layout primitive for vertical or horizontal composition.

## When to use
Use when children need tokenized spacing, alignment and wrapping without owning product behavior.

## When not to use
Do not use for semantic navigation, forms or data structures when a more specific component exists.

## Accessibility
`as` preserves the caller's semantic element. Stack does not add ARIA by itself.

## Keyboard
Stack has no keyboard behavior; focus order follows DOM order.

## Examples
`<Stack gap="lg"><slot /></Stack>`

## Anti Patterns
Do not use inline styles or domain-specific spacing classes inside shared layout.
