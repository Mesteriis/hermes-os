# ImageGallery

## Description
UI-only image gallery with local selected state and thumbnail buttons.

## When to use
Use for already-loaded image collections that need visual browsing.

## When not to use
Do not trigger provider downloads, pagination or search from this component.

## Accessibility
Gallery thumbnails are buttons with pressed state and labels.

## Keyboard
Thumbnails use native button keyboard behavior.

## Examples
`<ImageGallery :items="images" label="Evidence images" />`

## Anti Patterns
Do not store canonical media state inside the gallery.
