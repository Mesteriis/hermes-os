# AudioPlayer

## Description
Themed wrapper around native audio playback with title and description.

## When to use
Use for local voice notes, recordings or audio attachments already safe to render.

## When not to use
Do not implement transcription, download or provider command logic here.

## Accessibility
Use a clear title so the native control has an accessible name.

## Keyboard
Playback controls use native browser keyboard behavior.

## Examples
`<AudioPlayer :src="audioUrl" title="Voice note" />`

## Anti Patterns
Do not treat this component as canonical attachment state.
