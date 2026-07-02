# VideoPlayer

## Description
Themed wrapper around native video playback with optional tracks and fallback.

## When to use
Use for local or already-authorized video preview URLs.

## When not to use
Do not negotiate provider media transfer here.

## Accessibility
Provide a title and caption tracks when available.

## Keyboard
Playback controls use native browser keyboard behavior.

## Examples
`<VideoPlayer :src="videoUrl" title="Clip" />`

## Anti Patterns
Do not log or embed private remote media URLs in stories.
