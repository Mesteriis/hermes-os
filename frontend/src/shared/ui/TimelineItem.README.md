# TimelineItem

## Description
Single timeline event for chronological UI surfaces.

## When to use
Use for observed UI-only event summaries.

## When not to use
Do not use as an event store or provenance source.

## Accessibility
Uses readable article structure with optional time text.

## Keyboard
No custom keyboard behavior is added.

## Examples
`<TimelineItem title="Observed" time="10:42" />`

## Anti Patterns
Do not create durable business events from this component.
