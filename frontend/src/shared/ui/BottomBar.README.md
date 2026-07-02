# BottomBar

## Description
Bottom shell or panel bar with start, main and end slots.

## When to use
Use for local footer actions, state summaries or persistent low-height controls.

## When not to use
Do not use for toast or alert feedback.

## Accessibility
Defaults to `footer`; provide `label` when there are multiple footer-like bars.

## Keyboard
BottomBar does not manage focus.

## Examples
`<BottomBar label="Panel footer">...</BottomBar>`

## Anti Patterns
Do not duplicate StatusBar content in BottomBar without a clear reason.
