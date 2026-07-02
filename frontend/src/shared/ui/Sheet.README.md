# Sheet

## Description
Modal edge panel for temporary context and settings.

## When to use
Use for side inspection, settings and review panels that should block background interaction.

## When not to use
Do not use for persistent layout columns; use `SidePanel` or `InspectorPanel`.

## Accessibility
Built on Reka Dialog so focus is managed while the sheet is open.

## Keyboard
Escape closes the sheet. Tab stays inside the sheet.

## Examples
`<Sheet side="right" title="Inspector">...</Sheet>`

## Anti Patterns
Do not use Sheet for route-level navigation.
