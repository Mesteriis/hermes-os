# ComposerToolbar

## Description
Generic toolbar for composer actions.

## When to use
Use for formatting, insert and local draft actions around a composer.

## When not to use
Do not place provider send or remote mutation logic here.

## Accessibility
Renders a labeled toolbar with native buttons.

## Keyboard
Toolbar actions use native button keyboard behavior.

## Examples
`<ComposerToolbar :actions="actions" @select="handleAction" />`

## Anti Patterns
Do not create provider-specific toolbars in `shared/ui`.
