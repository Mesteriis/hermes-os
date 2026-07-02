# ThemeSwitcher

## Description
Controlled theme selector for Hermes UI themes.

## When to use
Use in Storybook, settings or shells that own theme state.

## When not to use
Do not persist theme settings from this component.

## Accessibility
Uses a radiogroup with one radio-like button per theme.

## Keyboard
Buttons use native focus and activation behavior.

## Examples
`<ThemeSwitcher v-model="theme" />`

## Anti Patterns
Do not mutate global document theme directly inside this component.
