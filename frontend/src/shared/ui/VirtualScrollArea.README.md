# VirtualScrollArea

## Description
Generic windowed scroll shell that exposes visible range metadata.

## When to use
Use for UI-only virtualized previews where the owner supplies rendered rows.

## When not to use
Do not use for tabular data; use `VirtualTable`.

## Accessibility
Provide `label` and render accessible content inside the slot.

## Keyboard
VirtualScrollArea keeps native scroll behavior.

## Examples
Use the slot props `visible-start` and `visible-end` to render a range.

## Anti Patterns
Do not put data fetching or provider paging into this component.
