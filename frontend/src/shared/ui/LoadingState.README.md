# LoadingState

## Description
Accessible local loading state.

## When to use
Use while a panel or component is waiting for data.

## When not to use
Do not use as a global application blocker.

## Accessibility
Uses `role="status"` and polite live region semantics.

## Keyboard
No custom keyboard behavior is added.

## Examples
`<LoadingState title="Refreshing" />`

## Anti Patterns
Do not hide long-running failures behind indefinite loading.
