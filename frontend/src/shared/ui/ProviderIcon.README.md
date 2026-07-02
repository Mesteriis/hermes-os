# ProviderIcon

## Description
Provider/channel icon primitive.

## When to use
Use for generic provider or channel identification in UI.

## When not to use
Do not couple it to provider runtime state or credentials.

## Accessibility
The wrapper exposes a readable image label.

## Keyboard
No keyboard behavior.

## Examples
`<ProviderIcon provider="mail" label="Mail channel" />`

## Anti Patterns
Do not use the icon as authority that a provider is connected.
