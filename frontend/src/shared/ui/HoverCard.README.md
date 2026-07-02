# HoverCard

## Description
Non-modal preview surface opened from hover or focus.

## When to use
Use for lightweight contextual previews that do not require a command.

## When not to use
Do not use for critical information or actions that keyboard users must discover.

## Accessibility
Built on Reka HoverCard. Trigger content remains the accessible entry point.

## Keyboard
Focus can reveal the card. Interactive content should remain simple and reachable elsewhere.

## Examples
`<HoverCard><template #trigger>...</template>...</HoverCard>`

## Anti Patterns
Do not hide required instructions only in a hover card.
