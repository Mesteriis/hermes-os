# InspectorPanel

## Description
Structured inspector panel with header, body and footer slots.

## When to use
Use for evidence, properties or settings-like side inspection surfaces.

## When not to use
Do not embed provider-specific business behavior in the shared inspector.

## Accessibility
Defaults to `aside` and uses `title` or `label` for accessible naming.

## Keyboard
InspectorPanel does not trap focus or own shortcuts.

## Examples
`<InspectorPanel title="Evidence">...</InspectorPanel>`

## Anti Patterns
Do not mutate domain state directly from InspectorPanel.
