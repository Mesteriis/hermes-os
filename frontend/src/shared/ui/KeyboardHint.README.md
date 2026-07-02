# KeyboardHint

## Description
Inline keyboard shortcut hint with optional label.

## When to use
Use in menus, toolbars and command surfaces.

## When not to use
Do not use as the only way to explain an action.

## Accessibility
Includes screen-reader text for the label and keys.

## Keyboard
No interactive behavior.

## Examples
`<KeyboardHint label="Open" :keys="['Meta', 'K']" />`

## Anti Patterns
Do not show shortcuts that are not actually supported by the parent surface.
