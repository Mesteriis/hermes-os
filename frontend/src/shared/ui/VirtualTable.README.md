# VirtualTable

## Description
Windowed table shell for large row sets where the owning surface controls the visible range.

## When to use
Use when rendering a deterministic slice of a larger local table.

## When not to use
Do not use as a persistence or data-loading layer.

## Accessibility
Keeps native table semantics and exposes visible range text.

## Keyboard
Native table navigation and reading order are preserved.

## Examples
`<VirtualTable :columns="columns" :rows="rows" :visible-count="8" />`

## Anti Patterns
Do not hide filtering, fetching or provider logic inside the component.
