# Dock

## Description
Generic edge dock for dense navigation or tool clusters.

## When to use
Use for UI-only side or top/bottom rails.

## When not to use
Do not use for domain navigation that needs routing state; keep routing in app composition.

## Accessibility
Defaults to `nav`; provide `label` when multiple docks exist.

## Keyboard
Dock does not implement roving focus. Buttons and links inside own focus behavior.

## Examples
`<Dock label="Workspace dock" position="left">...</Dock>`

## Anti Patterns
Do not import router or store state into Dock.
