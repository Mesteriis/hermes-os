# VirtualList

## Description
Windowed list surface for deterministic slices of large local lists.

## When to use
Use when an owner already knows the visible range.

## When not to use
Do not use as a scroll engine or query cache.

## Accessibility
Exposes `aria-posinset` and `aria-setsize` for visible items.

## Keyboard
No custom keyboard behavior is added.

## Examples
`<VirtualList :items="items" :visible-start="10" :visible-count="6" />`

## Anti Patterns
Do not fetch or page provider data from this UI primitive.
