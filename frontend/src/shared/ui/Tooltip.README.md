# Tooltip

## Description
Short non-interactive text hint attached to a trigger.

## When to use
Use for icon-only controls, abbreviations and compact UI hints.

## When not to use
Do not use for interactive content, errors or critical instructions.

## Accessibility
Built on Reka Tooltip with delayed display and trigger semantics.

## Keyboard
Focus can reveal the tooltip. Tooltip content itself is not interactive.

## Examples
`<Tooltip content="Refresh"><template #trigger>...</template></Tooltip>`

## Anti Patterns
Do not hide validation errors in tooltips.
