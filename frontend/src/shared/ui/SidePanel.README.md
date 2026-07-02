# SidePanel

## Description
Side-attached panel for generic layout composition.

## When to use
Use for local navigation, filters or context panels.

## When not to use
Do not use for modal or temporary overlay behavior; use Sheet or Drawer-style overlays.

## Accessibility
Defaults to `aside` and uses `title` or `label` for accessible naming.

## Keyboard
SidePanel does not trap focus.

## Examples
`<SidePanel title="Filters" side="left">...</SidePanel>`

## Anti Patterns
Do not import domain stores into SidePanel.
