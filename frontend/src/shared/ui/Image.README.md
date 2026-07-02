# Image

## Description
Theme-aware image primitive with fallback, caption and fit controls.

## When to use
Use for safe image previews inside generic UI surfaces.

## When not to use
Do not fetch provider media or resolve blobs here; pass an already-safe `src`.

## Accessibility
Provide meaningful `alt` text unless the image is decorative.

## Keyboard
No custom keyboard handling.

## Examples
`<Image src="/preview.png" alt="Evidence screenshot" ratio="video" />`

## Anti Patterns
Do not pass private provider URLs or raw media secrets.
