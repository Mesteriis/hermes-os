# Drawer

## Description
Modal drawer surface for temporary dense context.

## When to use
Use for mobile-like bottom panels, compact local settings, or temporary inspection.

## When not to use
Do not use for persistent application columns. Use `SidePanel` or `InspectorPanel`.

## Accessibility
Built on Reka Dialog so focus is trapped while open and the surface is labelled.

## Keyboard
Escape closes the drawer. Tab moves within drawer content.

## Examples
`<Drawer side="bottom" title="Review context">...</Drawer>`

## Anti Patterns
Do not store provider or domain state inside Drawer.
