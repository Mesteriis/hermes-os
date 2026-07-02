# ReadReceipt

## Description
Avatar cluster and label for generic read receipts.

## When to use
Use when a parent surface already knows receipt metadata.

## When not to use
Do not query provider receipt state from this component.

## Accessibility
The cluster is grouped with a readable label.

## Keyboard
No keyboard behavior.

## Examples
`<ReadReceipt :items="receipts">Read by 3</ReadReceipt>`

## Anti Patterns
Do not display raw personal data without parent-level sanitization.
