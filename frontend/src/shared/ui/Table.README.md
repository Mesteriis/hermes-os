# Table

## Description
Structured data table for compact desktop review surfaces.

## When to use
Use for comparable rows with stable columns.

## When not to use
Do not use for free-form cards or activity feeds.

## Accessibility
Renders a native table with column headers and optional caption.

## Keyboard
Native table reading order is preserved.

## Examples
`<Table :columns="columns" :rows="rows" />`

## Anti Patterns
Do not fetch data or encode business status rules inside the table.
