# DeliveryStatus

## Description
Composite delivery summary with status, description and time.

## When to use
Use below a message or command preview to explain delivery state.

## When not to use
Do not execute retries or provider commands from this component.

## Accessibility
Combines visible text with the accessible `MessageStatus` primitive.

## Keyboard
No keyboard behavior.

## Examples
`<DeliveryStatus status="failed" description="Retry required" />`

## Anti Patterns
Do not hide failed delivery behind color-only styling.
