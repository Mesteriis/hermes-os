# ReactionBadge

## Description
Compact reaction count with optional interactive state.

## When to use
Use for emoji or symbolic reactions in review or conversation surfaces.

## When not to use
Do not mutate reaction state or call APIs from the component.

## Accessibility
Interactive mode uses a button with `aria-pressed`.

## Keyboard
Interactive mode uses native button keyboard behavior.

## Examples
`<ReactionBadge emoji="+" :count="3" interactive />`

## Anti Patterns
Do not rely on the emoji alone as the accessible label.
