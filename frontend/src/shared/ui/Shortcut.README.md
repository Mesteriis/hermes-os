# Shortcut

## Description
Compact key sequence primitive.

## When to use
Use where only the key chord needs to be shown.

## When not to use
Do not use for clickable controls.

## Accessibility
The sequence has an accessible label.

## Keyboard
No interactive behavior.

## Examples
`<Shortcut :keys="['Meta', 'Enter']" />`

## Anti Patterns
Do not encode platform-specific behavior without parent context.
