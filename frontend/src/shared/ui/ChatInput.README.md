# ChatInput

## Description
Controlled composer input with attach and submit actions.

## When to use
Use for generic draft entry where the owner supplies text.

## When not to use
Do not send remote messages or persist drafts from this component.

## Accessibility
Supports a visible label, disabled state, helper text and native textarea semantics.

## Keyboard
Ctrl+Enter and Meta+Enter submit when text is present.

## Examples
`<ChatInput v-model="draft" @submit="handleSubmit" />`

## Anti Patterns
Do not wire provider command execution directly into the component.
