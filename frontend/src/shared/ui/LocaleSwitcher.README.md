# LocaleSwitcher

## Description
Controlled locale selector.

## When to use
Use where a parent owns locale state.

## When not to use
Do not load dictionaries or persist locale here.

## Accessibility
Uses a radiogroup with clear locale labels.

## Keyboard
Buttons use native focus and activation behavior.

## Examples
`<LocaleSwitcher v-model="locale" :options="locales" />`

## Anti Patterns
Do not hardcode application i18n behavior in the UI primitive.
